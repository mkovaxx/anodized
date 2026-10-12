#![no_main]
#![allow(unused_imports)]

use anodized::{spec, types::Spec};

#[spec]
struct S {
    field: Spec!(i32, inout),
}
