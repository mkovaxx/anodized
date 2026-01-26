#![no_main]

use anodized::spec;

trait T {
    #[spec]
    fn f(&self);
}
