#![no_main]

use anodized::spec;

#[spec(ensures: |Some(ref key)| !key.is_empty())]
fn get_opt() -> Option<String> {
    Some("THE GOAT".into())
}
