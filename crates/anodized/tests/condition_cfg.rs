//! A `#[cfg]` on a condition must gate the check in every build configuration.
//!
//! These tests are gated on `anodized_panic` alone, not on `all(anodized_print,
//! anodized_panic)` as the other panic tests are. That difference is the point: the guard was
//! once emitted only alongside printing, so a gated-off condition was still checked whenever
//! printing was disabled, and no test that required both flags could see it.

use anodized::spec;

#[spec(
    #[cfg(any())]
    requires: false,
)]
pub fn gated_off_precondition(value: i32) -> i32 {
    value
}

#[spec(
    #[cfg(all())]
    requires: false,
)]
pub fn gated_on_precondition(value: i32) -> i32 {
    value
}

#[spec(
    #[cfg(any())]
    ensures: |output| output == value + 1,
)]
pub fn gated_off_postcondition(value: i32) -> i32 {
    value
}

#[spec(
    #[cfg(all())]
    ensures: |output| output == value + 1,
)]
pub fn gated_on_postcondition(value: i32) -> i32 {
    value
}

#[test]
fn gated_off_conditions_are_never_checked() {
    assert_eq!(gated_off_precondition(1), 1);
    assert_eq!(gated_off_postcondition(2), 2);
}

#[cfg(anodized_panic)]
#[test]
#[should_panic(expected = "precondition failed")]
fn gated_on_precondition_is_checked() {
    gated_on_precondition(1);
}

#[cfg(anodized_panic)]
#[test]
#[should_panic(expected = "postcondition failed")]
fn gated_on_postcondition_is_checked() {
    gated_on_postcondition(1);
}
