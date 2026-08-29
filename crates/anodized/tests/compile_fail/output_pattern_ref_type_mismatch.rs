// Tests destructuring a reference return type (`&mut (T, T)`) by value `|(a, b)|`.
//
// QUESTION: What should the correct behavior be?
// Option A: Yield a type mismatch error (`E0308`), guiding the user toward reference matching `|(ref a, ref b)|` or `|output|`.
// Option B: Automatically reborrow reference return types when destructuring.
use anodized::spec;

#[spec(ensures: |(a, b)| a <= b)]
fn sort_pair_ref(pair: &mut (i32, i32)) -> &mut (i32, i32) {
    pair
}

fn main() {}
