use anodized::spec;

#[spec(
    binds: (a, b),
    ensures: [
        a <= b,
        (*a, *b) == pair || (*b, *a) == pair,
    ],
)]
fn sort_pair(pair: (i32, i32)) -> (i32, i32) {
    // Deliberately wrong implementation to break the spec.
    pair
}

#[cfg(feature = "backend-check-and-panic")]
#[test]
#[should_panic(expected = "Postcondition failed: | (a, b) | a <= b")]
fn sort_fail_postcondition() {
    sort_pair((5, 2));
}
