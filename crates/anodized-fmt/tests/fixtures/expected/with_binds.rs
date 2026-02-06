use anodized::spec;

#[spec(
    binds: result,
    ensures: *result % 2 == 0,
)]
fn calculate_odd_result(output: i32) -> i32 {
    if output % 2 == 0 {
        output + 1
    } else {
        output + 2
    }
}

#[cfg(feature = "runtime-check-and-panic")]
#[test]
#[should_panic(expected = "Postcondition failed: | result | * result % 2 == 0")]
fn rename_panics_if_not_even() {
    // Returns 5, violating the postcondition.
    calculate_odd_result(4);
}
