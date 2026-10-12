#![cfg_attr(anodized_charon, feature(register_tool))]
#![cfg_attr(anodized_charon, register_tool(charon))]
#![allow(unused)]

use anodized::{spec, types::Spec};

/// A `struct` with a type spec, a.k.a. refinement.
#[spec(
    maintains: [
        // Something something...
        todo!()
    ],
)]
struct T;

/// Default: type specs of inputs and the output are not enforced.
#[spec]
fn one(x: T, y: &mut T) -> T {
    todo!()
}

/// The type spec of input `x` must hold on entry.
#[spec]
fn two(x: Spec!(T), y: &mut T) -> T {
    todo!()
}

/// The type spec of input `y` must hold on exit.
#[spec]
fn three(x: T, y: Spec!(&mut T, out)) -> T {
    todo!()
}

/// The type spec of input `y` must hold on both entry and exit.
#[spec]
fn four(x: T, y: Spec!(&mut T, inout)) -> T {
    todo!()
}

/// The type spec of the output must hold on exit.
#[spec]
fn five(x: T, y: &mut T) -> Spec!(T) {
    todo!()
}

/// Default: the type specs of fields are not enforced.
#[spec]
struct Six {
    pub a: T,
    pub b: T,
}

/// The type spec of field `b` must hold.
#[spec]
struct Seven {
    pub a: T,
    pub b: Spec!(T),
}
