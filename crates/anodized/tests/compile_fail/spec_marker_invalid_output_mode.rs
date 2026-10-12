#![no_main]
#![allow(unused_imports)]

use anodized::{spec, types::Spec};

#[spec]
fn f() -> Spec!(i32, out) {
    0
}
