#![no_main]
#![allow(unused_imports)]

use anodized::{spec, unspec};

#[spec]
struct S {
    #[unspec(out)]
    field: i32,
}
