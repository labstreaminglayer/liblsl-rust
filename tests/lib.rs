use lsl::*;

#[test]
fn clock_is_working() {
    assert_ne!(local_clock(), 0.0);
}
