// Tests writing `mut` bindings in a postcondition pattern (e.g. `|(mut a, b)|`).
// Current behavior is bad: The macro panics with a confusing error.
//
// QUESTION: What should the correct behavior be?
// Option A: The macro should emit a clear diagnostic error rejecting `mut` bindings in postconditions.
// Option B: The macro should strip `mut` when re-assembling the output reconstruction expression.
use anodized::spec;

#[spec(ensures: |(mut a, b)| a <= b)]
fn sort_pair(pair: (i32, i32)) -> (i32, i32) {
    pair
}

fn main() {}
