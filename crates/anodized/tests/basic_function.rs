use anodized::spec;

#[spec(
    requires: divisor != 0,
    ensures: output < dividend,
)]
fn checked_divide(dividend: i32, divisor: i32) -> i32 {
    dividend / divisor
}

#[test]
fn test_divide_success() {
    checked_divide(10, 2);
}

#[test]
#[should_panic(expected = "Precondition failed: divisor != 0")]
fn test_divide_by_zero_panics() {
    checked_divide(10, 0);
}
