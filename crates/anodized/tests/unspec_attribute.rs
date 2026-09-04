#![no_main]

use anodized::{spec, unspec};

// Default: all inputs must satisfy their type specs on both entry and exit, and the output on exit.
#[spec]
fn one(x: P, y: &mut Q) -> R;

// Input `x` is "unspec-ified" (i.e. may not satisfy its type spec) on exit:
#[spec]
fn two(#[unspec(out)] x: P, y: &mut Q) -> R;

// Input `y` is unspecified on entry:
#[spec]
fn three(x: P, #[unspec(in)] y: &mut Q) -> R;

// Input `y` is unspecified on both entry and exit:
#[spec]
fn four(x: P, #[unspec] y: &mut Q) -> R;

// The output is unspecified on exit:
#[spec]
#[unspec(out)]
fn five(x: P, y: &mut Q) -> R;

// Default: all fields must satisfy their type specs:
#[spec]
struct Six {
    pub a: T,
    pub b: T,
}

// The field `b` is unspecified:
#[spec]
struct Seven {
    pub a: T,
    #[unspec]
    pub b: T,
}
