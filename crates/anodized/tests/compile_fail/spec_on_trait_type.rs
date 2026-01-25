#![no_main]

use anodized::spec;

#[spec]
trait T {
    #[spec]
    type U;
}
