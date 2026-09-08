#![no_main]
#![allow(unused_imports)]

use anodized::{spec, unspec};

#[spec]
fn f(#[unspec] #[unspec(in)] x: i32) {}
