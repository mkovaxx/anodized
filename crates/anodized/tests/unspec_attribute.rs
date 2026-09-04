#![allow(unused)]

use anodized::{spec, unspec};

/// A `struct` with a type spec, a.k.a. refinement.
#[spec(
    maintains: [
        // Something something...
        todo!()
    ],
)]
struct T;

/// Default: all inputs satisfy their type specs on both entry and exit, and the output on exit.
#[spec]
fn one(x: T, y: &mut T) -> T {
    todo!()
}

/// Input `x` may not satisfy its type spec on exit.
#[spec]
fn two(#[unspec(out)] x: T, y: &mut T) -> T {
    todo!()
}

/// Input `y` may not satisfy its type spec on entry.
#[spec]
fn three(x: T, #[unspec(in)] y: &mut T) -> T {
    todo!()
}

/// Input `y` may not satisfy its type spec on both entry and exit.
#[spec]
fn four(x: T, #[unspec] y: &mut T) -> T {
    todo!()
}

/// The output may not satisfy its type spec on exit.
#[spec]
#[unspec(out)]
fn five(x: T, y: &mut T) -> T {
    todo!()
}

/// Default: all fields must satisfy their type specs.
#[spec]
struct Six {
    pub a: T,
    pub b: T,
}

/// The field `b` may not satisfy its type spec.
#[spec]
struct Seven {
    pub a: T,
    #[unspec]
    pub b: T,
}
