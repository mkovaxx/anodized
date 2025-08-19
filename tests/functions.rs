use anodized::contract;

#[contract(
    requires: divisor != 0,
    ensures: output < dividend,
)]
fn checked_divide(dividend: i32, divisor: i32) -> i32 {
    dividend / divisor
}

#[test]
fn precondition_true() {
    // This call satisfies the contract and runs fine.
    println!("10 / 2 = {}", checked_divide(10, 2));
}

#[test]
#[should_panic(expected = "Precondition failed: divisor != 0")]
fn precondition_false() {
    // This call violates the precondition and will panic in debug builds.
    println!("10 / 0 = {}", checked_divide(10, 0));
}
