namespace CauslaneFormal

inductive EventKind where
  | actionAdmitted
  | actionPlanned
  | dispatchLogged
  | executionBarrierLogged
  | executionStarted
  | executionCompleted
  | observedTruthCommitted
  | projectionEmitted
  | lifecycleClosed
  | gateApproved
  | gateDenied
  | authzDecisionRecorded
  | constraintLeaseGranted
  | constraintLeaseReleased
  | overlayAttached
  | constraintUpdated
  | drainFenceRequested
  | drainFenceAcquired
deriving DecidableEq, Repr

structure Event where
  eventId : String
  kind : EventKind
  actionId : Option String
  planHash : Option String
  opIndex : Option Nat
  factKind : Option String
  scope : Option String
  anchorEventId : Option String
  anchorFactKind : Option String
  anchorScope : Option String
  barrierEventId : Option String
  barrierRef : Option String
  impactSetHash : Option String
  witnessBindAction : Option String
  witnessBindPlan : Option String
  witnessBindImpact : Option String
deriving Repr

def eventAt (trace : List Event) (index : Nat) : Option Event :=
  trace[index]?

def eventKindAt (trace : List Event) (index : Nat) : Option EventKind :=
  match eventAt trace index with
  | some event => some event.kind
  | none => none

def eventActionAt (trace : List Event) (index : Nat) : Option String :=
  match eventAt trace index with
  | some event => event.actionId
  | none => none

def eventPlanAt (trace : List Event) (index : Nat) : Option String :=
  match eventAt trace index with
  | some event => event.planHash
  | none => none

def eventFactAt (trace : List Event) (index : Nat) : Option String :=
  match eventAt trace index with
  | some event => event.factKind
  | none => none

def eventScopeAt (trace : List Event) (index : Nat) : Option String :=
  match eventAt trace index with
  | some event => event.scope
  | none => none

def eventAnchorEventIdAt (trace : List Event) (index : Nat) : Option String :=
  match eventAt trace index with
  | some event => event.anchorEventId
  | none => none

def eventAnchorFactAt (trace : List Event) (index : Nat) : Option String :=
  match eventAt trace index with
  | some event => event.anchorFactKind
  | none => none

def eventAnchorScopeAt (trace : List Event) (index : Nat) : Option String :=
  match eventAt trace index with
  | some event => event.anchorScope
  | none => none

def eventBarrierRefAt (trace : List Event) (index : Nat) : Option String :=
  match eventAt trace index with
  | some event => event.barrierRef
  | none => none

def eventImpactAt (trace : List Event) (index : Nat) : Option String :=
  match eventAt trace index with
  | some event => event.impactSetHash
  | none => none

def eventWitnessBindActionAt (trace : List Event) (index : Nat) : Option String :=
  match eventAt trace index with
  | some event => event.witnessBindAction
  | none => none

def eventWitnessBindPlanAt (trace : List Event) (index : Nat) : Option String :=
  match eventAt trace index with
  | some event => event.witnessBindPlan
  | none => none

def eventWitnessBindImpactAt (trace : List Event) (index : Nat) : Option String :=
  match eventAt trace index with
  | some event => event.witnessBindImpact
  | none => none

def isLifecycleMutation (event : Event) : Bool :=
  match event.kind with
  | EventKind.executionStarted => true
  | EventKind.executionCompleted => true
  | EventKind.observedTruthCommitted => true
  | EventKind.projectionEmitted => true
  | EventKind.constraintLeaseGranted => true
  | EventKind.constraintLeaseReleased => true
  | EventKind.constraintUpdated => true
  | EventKind.drainFenceRequested => true
  | EventKind.drainFenceAcquired => true
  | _ => false

def noLifecycleMutationsAfter (trace : List Event) (index : Nat) : Bool :=
  (trace.drop (index + 1)).all fun event => !(isLifecycleMutation event)

inductive ConsequenceProfile where
  | runtimeExecution
  | projectionRead
  | oversightMeta
  | topologyMeta
  | evidenceMeta
  | outsideKernel
deriving DecidableEq, Repr

inductive LifecycleClass where
  | executionBearing
  | projectionOnly
  | metaOnly
deriving DecidableEq, Repr

structure PredicateRoute where
  predicateId : String
  routeId : String
  consequenceProfile : ConsequenceProfile
  lifecycleClass : LifecycleClass
deriving Repr

def lifecycleClassForProfile (profile : ConsequenceProfile) : LifecycleClass :=
  match profile with
  | ConsequenceProfile.runtimeExecution => LifecycleClass.executionBearing
  | ConsequenceProfile.projectionRead => LifecycleClass.projectionOnly
  | ConsequenceProfile.oversightMeta
  | ConsequenceProfile.topologyMeta
  | ConsequenceProfile.evidenceMeta
  | ConsequenceProfile.outsideKernel => LifecycleClass.metaOnly

def routeConsistentWithProfile (route : PredicateRoute) : Bool :=
  if route.lifecycleClass = lifecycleClassForProfile route.consequenceProfile then
    true
  else
    false

def allRoutesConsistentWithProfiles (routes : List PredicateRoute) : Bool :=
  routes.all routeConsistentWithProfile

inductive ClaimMode where
  | sharedRead
  | exclusiveWrite
  | token
deriving DecidableEq, Repr

def isExclusive (mode : ClaimMode) : Bool :=
  match mode with
  | ClaimMode.exclusiveWrite => true
  | ClaimMode.sharedRead
  | ClaimMode.token => false

def claimModesConflict
    (left : ClaimMode)
    (right : ClaimMode)
    (sameResource : Bool)
    (sameScope : Bool)
    (verifiedMerge : Bool) : Bool :=
  sameResource && sameScope && (isExclusive left || isExclusive right) && !verifiedMerge

def boolValues : List Bool := [false, true]

def allClaimModes : List ClaimMode := [
  ClaimMode.sharedRead,
  ClaimMode.exclusiveWrite,
  ClaimMode.token
]

def allClaimModePairs : List (ClaimMode × ClaimMode) :=
  List.flatten (allClaimModes.map fun left =>
  allClaimModes.map fun right => (left, right))

def allBoolPairs : List (Bool × Bool) :=
  List.flatten (boolValues.map fun left =>
  boolValues.map fun right => (left, right))

def allBoolTriples : List (Bool × Bool × Bool) :=
  List.flatten (boolValues.map fun first =>
  List.flatten (boolValues.map fun second =>
  boolValues.map fun third => (first, second, third)))

def leaseConflictFailClosedCase
    (left : ClaimMode)
    (right : ClaimMode)
    (sameResource : Bool)
    (sameScope : Bool)
    (verifiedMerge : Bool) : Bool :=
  if sameResource && sameScope && !verifiedMerge && (isExclusive left || isExclusive right) then
    claimModesConflict left right sameResource sameScope verifiedMerge
  else
    true

def leaseConflictFailClosedHolds : Bool :=
  allClaimModePairs.all fun (left, right) =>
    allBoolTriples.all fun (sameResource, sameScope, verifiedMerge) =>
      leaseConflictFailClosedCase left right sameResource sameScope verifiedMerge

def verifiedMergeClearsConflictHolds : Bool :=
  allClaimModePairs.all fun (left, right) =>
    allBoolPairs.all fun (sameResource, sameScope) =>
      !claimModesConflict left right sameResource sameScope true

structure DrainLeaseSlot where
  overlaps : Bool
  active : Bool
  expired : Bool
deriving Repr

structure DrainFenceCheck where
  left : DrainLeaseSlot
  right : DrainLeaseSlot
deriving Repr

def activeUnexpiredOverlap (slot : DrainLeaseSlot) : Bool :=
  slot.overlaps && slot.active && !slot.expired

def drainSlotClearSpec (slot : DrainLeaseSlot) : Bool :=
  !activeUnexpiredOverlap slot

def drainFenceClearSpec (check : DrainFenceCheck) : Bool :=
  drainSlotClearSpec check.left && drainSlotClearSpec check.right

def drainFenceAcquirable (check : DrainFenceCheck) : Bool :=
  drainFenceClearSpec check

def allDrainLeaseSlots : List DrainLeaseSlot :=
  allBoolTriples.map fun (overlaps, active, expired) => {
    overlaps := overlaps,
    active := active,
    expired := expired
  }

def allDrainFenceChecks : List DrainFenceCheck :=
  List.flatten (allDrainLeaseSlots.map fun left =>
  allDrainLeaseSlots.map fun right => { left := left, right := right })

def expiredOverlapDoesNotBlock (slot : DrainLeaseSlot) : Bool :=
  if slot.overlaps && slot.active && slot.expired then
    drainSlotClearSpec slot
  else
    true

def drainAfterOverlapClearCase (check : DrainFenceCheck) : Bool :=
  (drainFenceAcquirable check == drainFenceClearSpec check) &&
  (if drainFenceAcquirable check then drainFenceClearSpec check else true) &&
  expiredOverlapDoesNotBlock check.left &&
  expiredOverlapDoesNotBlock check.right

def drainAfterOverlapClearHolds : Bool :=
  allDrainFenceChecks.all drainAfterOverlapClearCase

structure ObligationSet where
  requiresWitness : Bool
  requiresClaim : Bool
  requiresAuthz : Bool
  requiresBarrier : Bool
  requiresAnchor : Bool
deriving Repr

def requirementNotWeakened (baseRequired : Bool) (overlaidRequired : Bool) : Bool :=
  if baseRequired then overlaidRequired else true

def overlayDoesNotWeaken (base : ObligationSet) (overlaid : ObligationSet) : Bool :=
  requirementNotWeakened base.requiresWitness overlaid.requiresWitness &&
  requirementNotWeakened base.requiresClaim overlaid.requiresClaim &&
  requirementNotWeakened base.requiresAuthz overlaid.requiresAuthz &&
  requirementNotWeakened base.requiresBarrier overlaid.requiresBarrier &&
  requirementNotWeakened base.requiresAnchor overlaid.requiresAnchor

def preservedBy (base : ObligationSet) (overlaid : ObligationSet) : Bool :=
  overlayDoesNotWeaken base overlaid

def overlayAcceptedNeverWeakens (base : ObligationSet) (overlaid : ObligationSet) : Bool :=
  if preservedBy base overlaid then overlayDoesNotWeaken base overlaid else true

def allObligationSets : List ObligationSet :=
  List.flatten (boolValues.map fun requiresWitness =>
  List.flatten (boolValues.map fun requiresClaim =>
  List.flatten (boolValues.map fun requiresAuthz =>
  List.flatten (boolValues.map fun requiresBarrier =>
  boolValues.map fun requiresAnchor => {
    requiresWitness := requiresWitness,
    requiresClaim := requiresClaim,
    requiresAuthz := requiresAuthz,
    requiresBarrier := requiresBarrier,
    requiresAnchor := requiresAnchor
  }))))

def overlayMonotonicityHolds : Bool :=
  allObligationSets.all fun base =>
    allObligationSets.all fun overlaid =>
      overlayAcceptedNeverWeakens base overlaid

structure CommittedTruth where
  readinessCommitted : Bool
  promotionCommitted : Bool
  evidenceCommitted : Bool
deriving Repr

structure ConstraintUpdate where
  rewritesReadiness : Bool
  rewritesPromotion : Bool
  rewritesEvidence : Bool
deriving Repr

def constraintTruthPairs
    (update : ConstraintUpdate)
    (committed : CommittedTruth) : List (Bool × Bool) := [
  (update.rewritesReadiness, committed.readinessCommitted),
  (update.rewritesPromotion, committed.promotionCommitted),
  (update.rewritesEvidence, committed.evidenceCommitted)
]

def truthCategoryPreserved (rewrites : Bool) (committed : Bool) : Bool :=
  !rewrites || !committed

def preservesCommittedTruth (update : ConstraintUpdate) (committed : CommittedTruth) : Bool :=
  (constraintTruthPairs update committed).all fun (rewrites, committed) =>
    truthCategoryPreserved rewrites committed

def allCommittedTruthStates : List CommittedTruth :=
  allBoolTriples.map fun (readinessCommitted, promotionCommitted, evidenceCommitted) => {
    readinessCommitted := readinessCommitted,
    promotionCommitted := promotionCommitted,
    evidenceCommitted := evidenceCommitted
  }

def allConstraintUpdates : List ConstraintUpdate :=
  allBoolTriples.map fun (rewritesReadiness, rewritesPromotion, rewritesEvidence) => {
    rewritesReadiness := rewritesReadiness,
    rewritesPromotion := rewritesPromotion,
    rewritesEvidence := rewritesEvidence
  }

def acceptedDoesNotRewriteCommitted
    (accepted : Bool)
    (pairs : List (Bool × Bool)) : Bool :=
  pairs.all fun (rewrites, committed) =>
    if accepted && committed then !rewrites else true

def uncommittedRewriteAllowed (pairs : List (Bool × Bool)) : Bool :=
  pairs.all fun (rewrites, committed) =>
    if rewrites && !committed then truthCategoryPreserved rewrites committed else true

def constraintUpdateFutureOnlyCase (update : ConstraintUpdate) (committed : CommittedTruth) : Bool :=
  let pairs := constraintTruthPairs update committed
  let accepted := preservesCommittedTruth update committed
  acceptedDoesNotRewriteCommitted accepted pairs &&
  uncommittedRewriteAllowed pairs

def constraintUpdateFutureOnlyHolds : Bool :=
  allConstraintUpdates.all fun update =>
    allCommittedTruthStates.all fun committed =>
      constraintUpdateFutureOnlyCase update committed

end CauslaneFormal
