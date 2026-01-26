#![no_main]

use anodized::spec;

#[spec]
trait T {
    type U;
}

#[spec]
impl T for S {
    #[spec]
    type U = bool;
}

struct S;
