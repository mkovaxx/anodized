//! <https://github.com/anodized-rs/anodized/issues/201>
use anodized::spec;

#[spec]
fn id(x: &mut i32) -> &mut i32 {
    x
}
