#![no_main]

use anodized::spec;

#[spec]
trait T {
    fn f(&self);
}

#[spec]
impl T for S {
    #[spec]
    fn f(&self) {}
}

struct S;
