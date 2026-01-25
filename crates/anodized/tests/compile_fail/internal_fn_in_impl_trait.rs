#![no_main]

use anodized::spec;

trait T {
    fn f(&self);
}

#[spec]
impl T for S {
    fn __anodized_f(&self) {}
}

struct S;
