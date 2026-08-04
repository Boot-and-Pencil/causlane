use loom::sync::Arc;
use loom::sync::atomic::{AtomicU8, Ordering};
use loom::thread;

#[test]
fn loom_publishes_only_the_exact_selected_identity() {
    loom::model(|| {
        let selected = Arc::new(AtomicU8::new(0));
        let writer = Arc::clone(&selected);
        let handle = thread::spawn(move || writer.store(7, Ordering::Release));
        assert!(handle.join().is_ok(), "writer must finish");
        assert_eq!(selected.load(Ordering::Acquire), 7);
    });
}

#[cfg(feature = "detection-fixture")]
#[test]
fn loom_detection_control_rejects_a_widened_identity() {
    loom::model(|| {
        let selected = Arc::new(AtomicU8::new(0));
        let writer = Arc::clone(&selected);
        let handle = thread::spawn(move || writer.store(7, Ordering::Release));
        assert!(handle.join().is_ok(), "writer must finish");
        assert_eq!(selected.load(Ordering::Acquire), 8);
    });
}

