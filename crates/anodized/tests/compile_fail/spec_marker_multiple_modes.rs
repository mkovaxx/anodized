#![no_main]
#![allow(unused_imports)]

use anodized::{spec, types::Spec};

#[spec]
fn f(x: Spec!(i32, out, inout)) {}
