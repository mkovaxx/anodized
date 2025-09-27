use anodized::spec;

#[spec(
    ensures: [
        |(a, b)| a <= b,
        |(a, b)| (*a, *b) == pair || (*b, *a) == pair,
    ],
)]
fn sort_pair(pair: (i32, i32)) -> (i32, i32) {
    let (a, b) = pair;
    // Deliberately wrong implementation to break the spec.
    (b, a)
}

#[cfg(not(any(feature = "backend-no-checks", feature = "backend-no-panic")))]
#[test]
#[should_panic(expected = "Postcondition failed: | (a, b) | a <= b")]
fn sort_fail_postcondition() {
    sort_pair((2, 5));
}
