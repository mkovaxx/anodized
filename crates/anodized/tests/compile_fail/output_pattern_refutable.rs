// Tests writing refutable patterns in a postcondition pattern (e.g. `|Some(x)|`).
//
// QUESTION: What should the correct behavior be?
// Option A: Yield a compiler error (`E0005`), because `let` bindings require irrefutable patterns.
// Option B: Support refutable patterns using `if let` guards.
use anodized::spec;

#[spec(ensures: |Some(x)| x > 0)]
fn get_opt() -> Option<i32> {
    Some(10)
}

fn main() {}
