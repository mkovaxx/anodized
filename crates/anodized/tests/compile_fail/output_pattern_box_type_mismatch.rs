// Tests destructuring a smart-pointer return type (`Box<(T, T)>`) by value `|(a, b)|`.
//
// QUESTION: What should the correct behavior be?
// Option A: Yield a type mismatch error (`E0308`), guiding the user toward dereferencing `*` or `|output|`.
// Option B: Automatically auto-deref smart pointers when destructuring.
use anodized::spec;

#[spec(ensures: |(a, b)| a <= b)]
fn sort_pair_box(pair: Box<(i32, i32)>) -> Box<(i32, i32)> {
    pair
}

fn main() {}
