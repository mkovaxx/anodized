#![no_main]
#![allow(unused_imports)]

use anodized::{spec, unspec};

#[spec]
fn f(#[unspec(entry)] x: i32) {}
