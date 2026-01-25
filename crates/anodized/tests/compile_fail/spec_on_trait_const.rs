#![no_main]

use anodized::spec;

#[spec]
trait T {
    #[spec]
    const C: i32;
}
