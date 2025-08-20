use anodized::contract;

#[contract(
    returns: (a, b),
    ensures: a <= b,
    ensures: (a, b) == pair || (b, a) == pair,
)]
fn sort_pair(pair: (i32, i32)) -> (i32, i32) {
    // Deliberately wrong implementation to break the contract.
    pair
}

#[test]
#[should_panic(expected = "Postcondition failed: a <= b")]
fn test_sort_fail_postcondition() {
    sort_pair((5, 2));
}
