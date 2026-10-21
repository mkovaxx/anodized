#![no_main]

use anodized::spec;

#[spec(ensures: |output: u32| true)]
fn count_chars(input: String) -> usize {
    input.chars().count()
}
