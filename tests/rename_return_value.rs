use anodized::contract;

#[contract(
    returns: result,
    ensures: result > output,
    ensures: |val| val % 2 == 0,
)]
fn calculate_even_result(output: i32) -> i32 {
    if output % 2 == 0 {
        output + 2
    } else {
        output + 1
    }
}

#[test]
fn test_rename_success() {
    assert_eq!(calculate_even_result(4), 6);
    assert_eq!(calculate_even_result(5), 6);
}

#[test]
#[should_panic(expected = "Postcondition failed: val % 2 == 0")]
fn test_rename_panics_if_not_even() {
    #[contract(returns: result, ensures: |val| val % 2 == 0)]
    fn calculate_odd_result(output: i32) -> i32 {
        if output % 2 == 0 {
            output + 1
        } else {
            output + 2
        }
    }
    calculate_odd_result(4); // Returns 5, violating the postcondition.
}
