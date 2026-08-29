// Tests writing subpattern `@` bindings in a postcondition pattern (e.g. `|x @ (a, b)|`).
// Current behavior is bad: The macro panics with a confusing error.
//
// QUESTION: What should the correct behavior be?
// Option A: The macro should emit a clear diagnostic error rejecting `@` subpattern bindings.
// Option B: The macro should support `@` bindings by stripping the `@` alias when reconstructing the output expression.
use anodized::spec;

#[spec(ensures: |x @ (a, b)| a <= b)]
fn sort_pair(pair: (i32, i32)) -> (i32, i32) {
    pair
}

fn main() {}
