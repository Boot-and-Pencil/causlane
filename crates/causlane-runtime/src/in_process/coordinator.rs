//! Admission-time partition route coordinator.

use std::{collections::HashMap, sync::Arc};

use causlane_core::PartitionRoute;
use tokio::sync::{broadcast, OwnedSemaphorePermit, Semaphore};

use super::{publish, InProcessRuntimeError, InProcessRuntimeEvent};
use crate::partitions::PartitionKey;

pub(super) type AdmissionGuards = Vec<OwnedSemaphorePermit>;

#[derive(Debug)]
pub(super) struct AdmissionCoordinator {
    permits: HashMap<PartitionKey, Arc<Semaphore>>,
}

impl AdmissionCoordinator {
    pub(super) fn new(partitions: &[PartitionKey]) -> Self {
        let permits = partitions
            .iter()
            .map(|partition| (partition.clone(), Arc::new(Semaphore::new(1))))
            .collect();
        Self { permits }
    }

    pub(super) async fn lock_route(
        &self,
        route: &PartitionRoute,
        task_id: &str,
    ) -> Result<AdmissionGuards, InProcessRuntimeError> {
        let permits = self.route_permits(route, task_id)?;
        let mut guards = Vec::with_capacity(permits.len());
        for (partition, permit) in permits {
            guards.push(permit.acquire_owned().await.map_err(|_error| {
                InProcessRuntimeError::Closed {
                    partition: partition.clone(),
                    task_id: task_id.to_owned(),
                }
            })?);
        }
        Ok(guards)
    }

    pub(super) fn validate_route(
        &self,
        route: &PartitionRoute,
        task_id: &str,
    ) -> Result<(), InProcessRuntimeError> {
        self.route_permits(route, task_id).map(|_permits| ())
    }

    pub(super) fn try_lock_route(
        &self,
        route: &PartitionRoute,
        task_id: &str,
        events: &broadcast::Sender<InProcessRuntimeEvent>,
    ) -> Result<AdmissionGuards, InProcessRuntimeError> {
        let permits = self.route_permits(route, task_id)?;
        let mut guards = Vec::with_capacity(permits.len());
        for (partition, permit) in permits {
            match permit.try_acquire_owned() {
                Ok(guard) => guards.push(guard),
                Err(_error) => {
                    publish(
                        events,
                        InProcessRuntimeEvent::RouteBusy {
                            partition: partition.clone(),
                            task_id: task_id.to_owned(),
                        },
                    );
                    return Err(InProcessRuntimeError::RouteBusy {
                        partition,
                        task_id: task_id.to_owned(),
                    });
                }
            }
        }
        Ok(guards)
    }

    fn route_permits(
        &self,
        route: &PartitionRoute,
        task_id: &str,
    ) -> Result<Vec<(PartitionKey, Arc<Semaphore>)>, InProcessRuntimeError> {
        route
            .acquisition_order()
            .into_iter()
            .map(|partition| {
                self.permit_for(&partition, task_id)
                    .map(|permit| (partition, permit))
            })
            .collect()
    }

    fn permit_for(
        &self,
        partition: &PartitionKey,
        task_id: &str,
    ) -> Result<Arc<Semaphore>, InProcessRuntimeError> {
        self.permits.get(partition).cloned().ok_or_else(|| {
            InProcessRuntimeError::UnknownPartition {
                partition: partition.clone(),
                task_id: task_id.to_owned(),
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use causlane_core::PartitionRoute;
    use tokio::sync::broadcast;

    use super::{AdmissionCoordinator, InProcessRuntimeError, InProcessRuntimeEvent};
    use crate::partitions::PartitionKey;

    fn partition(id: &str) -> PartitionKey {
        PartitionKey(id.to_owned())
    }

    #[tokio::test(flavor = "current_thread")]
    async fn try_lock_route_reports_busy_partition() -> Result<(), InProcessRuntimeError> {
        let primary = partition("primary");
        let route = PartitionRoute::for_primary(primary.clone());
        let coordinator = AdmissionCoordinator::new(std::slice::from_ref(&primary));
        let (events, _initial_receiver) = broadcast::channel(4);
        let mut receiver = events.subscribe();
        let held = coordinator.lock_route(&route, "held").await?;

        assert!(matches!(
            coordinator.try_lock_route(&route, "blocked", &events),
            Err(InProcessRuntimeError::RouteBusy {
                partition,
                task_id,
            }) if partition == primary && task_id == "blocked"
        ));
        assert!(matches!(
            receiver.try_recv(),
            Ok(InProcessRuntimeEvent::RouteBusy {
                partition,
                task_id
            }) if partition == primary && task_id == "blocked"
        ));
        drop(held);
        Ok(())
    }
}
