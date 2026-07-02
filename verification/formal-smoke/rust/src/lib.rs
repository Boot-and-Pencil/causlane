#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Event {
    GateOpened,
    UseAttempted,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CheckResult {
    pub accepted: bool,
    pub rejected_at: Option<usize>,
    pub use_count: usize,
}

pub fn reduce(trace: &[Event]) -> CheckResult {
    let mut gate_seen = false;
    let mut use_count = 0usize;
    let mut i = 0usize;
    while i < trace.len() {
        match trace[i] {
            Event::GateOpened => gate_seen = true,
            Event::UseAttempted => {
                use_count += 1;
                if !gate_seen {
                    return CheckResult {
                        accepted: false,
                        rejected_at: Some(i),
                        use_count,
                    };
                }
            }
        }
        i += 1;
    }
    CheckResult {
        accepted: true,
        rejected_at: None,
        use_count,
    }
}

pub fn reference_accepts(trace: &[Event]) -> bool {
    let mut gate_seen = false;
    for event in trace {
        match event {
            Event::GateOpened => gate_seen = true,
            Event::UseAttempted if !gate_seen => return false,
            Event::UseAttempted => {}
        }
    }
    true
}

pub fn trace_from_bits(len: usize, bits: u8) -> [Event; 4] {
    let mut trace = [Event::GateOpened; 4];
    let mut i = 0usize;
    while i < 4 {
        if i < len {
            trace[i] = if ((bits >> i) & 1) == 0 {
                Event::GateOpened
            } else {
                Event::UseAttempted
            };
        }
        i += 1;
    }
    trace
}

pub fn trace_from_bytes(data: &[u8]) -> ([Event; 4], usize) {
    let len = data.first().copied().unwrap_or(0) as usize % 5;
    let bits = data.get(1).copied().unwrap_or(0);
    (trace_from_bits(len, bits), len)
}

#[cfg(kani)]
#[kani::proof]
fn gate_before_use_kani_smoke() {
    let len: usize = kani::any();
    kani::assume(len <= 4);
    let bits: u8 = kani::any();
    let trace = trace_from_bits(len, bits);
    let checked = reduce(&trace[..len]);
    assert_eq!(checked.accepted, reference_accepts(&trace[..len]));

    let invalid = [Event::UseAttempted];
    assert!(!reduce(&invalid).accepted);
    let valid = [Event::GateOpened, Event::UseAttempted];
    assert!(reduce(&valid).accepted);
}
