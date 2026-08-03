use anodized::spec;

#[spec(
    inspects: result,
    ensures: [
        result > output,
        |val| val % 2 == 0,
    ],
)]
fn calculate_even_result(output: i32) -> i32 {
    if output % 2 == 0 {
        output + 2
    } else {
        output + 1
    }
}

#[test]
fn rename_success() {
    calculate_even_result(4);
    calculate_even_result(5);
}

#[spec(
    inspects: result,
    ensures: result % 2 == 0,
)]
#[allow(unused)]
fn calculate_odd_result(output: i32) -> i32 {
    if output % 2 == 0 {
        output + 1
    } else {
        output + 2
    }
}

#[cfg(all(anodized_print, anodized_panic))]
#[test]
#[should_panic(expected = "postcondition failed:\
\n    result % 2 == 0")]
fn rename_panics_if_not_even() {
    // Returns 5, violating the postcondition.
    calculate_odd_result(4);
}
