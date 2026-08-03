use anodized::spec;

#[spec(
    ensures: |(a, b)| [
        a <= b,
        (a, b) == pair || (b, a) == pair,
    ],
)]
#[allow(unused)]
fn sort_pair(pair: (i32, i32)) -> (i32, i32) {
    // Deliberately wrong implementation to break the spec.
    pair
}

#[cfg(all(anodized_print, anodized_panic))]
#[test]
#[should_panic(expected = "postcondition failed:\
\n    a <= b")]
fn sort_fail_postcondition() {
    sort_pair((5, 2));
}
