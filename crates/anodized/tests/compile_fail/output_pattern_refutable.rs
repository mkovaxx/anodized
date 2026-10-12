#![no_main]

use anodized::spec;

#[spec(ensures: |Some(x)| x > 0)]
fn get_opt() -> Option<i32> {
    Some(10)
}
