#![no_main]

use anodized::spec;

// This test verifies that capture expressions are mutation-free.

#[spec(
    captures: first = input.pop(),
    ensures: |output| output == first,
)]
fn head(input: &mut Vec<i32>) -> Option<i32> {
    todo!()
}
