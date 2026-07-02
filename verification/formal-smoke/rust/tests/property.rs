use formal_smoke_model::{reference_accepts_bounded, reduce, reduce_bounded, trace_from_bits, Event};

#[test]
fn property_gate_before_use_matches_reference_for_bounded_traces() {
    for len in 0usize..=4 {
        for bits in 0u8..16 {
            let trace = trace_from_bits(len, bits);
            assert_eq!(reduce_bounded(&trace, len).accepted, reference_accepts_bounded(&trace, len));
        }
    }
}

#[test]
fn property_negative_control_rejects_use_before_gate() {
    let invalid = [Event::UseAttempted];
    assert!(!reduce(&invalid).accepted);
}

#[test]
fn property_positive_control_accepts_gate_then_use() {
    let valid = [Event::GateOpened, Event::UseAttempted];
    assert!(reduce(&valid).accepted);
}
