use anodized::spec;

#[spec(
    ensures: |(a, b)| [
        a <= b,
        (a, b) == pair || (b, a) == pair,
    ],
)]
#[allow(unused)]
fn sort_pair_i32(pair: (i32, i32)) -> (i32, i32) {
    // Deliberately wrong implementation to break the spec.
    pair
}

#[cfg(all(anodized_print, anodized_panic))]
#[test]
#[should_panic(expected = "postcondition failed")]
fn sort_pair_i32_fail_postcondition() {
    sort_pair_i32((5, 2));
}

#[spec(
    // Because `T` is generic, the output can only consist of elements of the input.
    ensures: |(a, b)| a <= b,
)]
#[allow(unused)]
fn sort_pair<T: Ord>(pair: (T, T)) -> (T, T) {
    // Deliberately wrong implementation to break the spec.
    pair
}

#[cfg(all(anodized_print, anodized_panic))]
#[test]
#[should_panic(expected = "postcondition failed")]
fn sort_pair_fail_postcondition() {
    sort_pair((5, 2));
}

#[spec(ensures: |(mut a, b)| a <= b)]
#[allow(unused_mut)]
fn sort_pair_with_mut_binding(pair: (i32, i32)) -> (i32, i32) {
    // Deliberately wrong implementation to break the spec.
    pair
}

#[cfg(all(anodized_print, anodized_panic))]
#[test]
#[should_panic(expected = "postcondition failed")]
fn output_pattern_with_mut_binding() {
    sort_pair_with_mut_binding((5, 2));
}

#[spec(ensures: |pair @ (a, b)| a <= b)]
fn sort_pair_with_at_binding(pair: (i32, i32)) -> (i32, i32) {
    // Deliberately wrong implementation to break the spec.
    pair
}

#[cfg(all(anodized_print, anodized_panic))]
#[test]
#[should_panic(expected = "postcondition failed")]
fn output_pattern_with_at_binding() {
    sort_pair_with_at_binding((5, 2));
}
