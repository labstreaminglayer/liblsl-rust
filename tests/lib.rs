use lsl::*;

#[test]
fn clock_delivers() {
    assert_ne!(local_clock(), 0.0);
}
