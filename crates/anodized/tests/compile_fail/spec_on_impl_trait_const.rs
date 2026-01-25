#![no_main]

use anodized::spec;

#[spec]
trait T {
    const C: i32;
}

#[spec]
impl T for S {
    #[spec]
    const C: i32 = 42;
}

struct S;
