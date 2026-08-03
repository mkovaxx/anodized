use anodized::spec;

#[spec(
    functional,
    requires: divisor != 0,
    ensures: |output| output < dividend,
)]
fn checked_divide(dividend: i32, divisor: i32) -> i32 {
    dividend / divisor
}

#[test]
fn divide_success() {
    checked_divide(10, 2);
}

#[cfg(all(anodized_print, anodized_panic))]
#[test]
#[should_panic(expected = "precondition failed:\
\n    divisor != 0")]
fn divide_by_zero_panics() {
    checked_divide(10, 0);
}
