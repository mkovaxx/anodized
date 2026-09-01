#![no_main]

use anodized::spec;

#[spec(ensures: |output: u32| output <= input.len())]
fn count_chars(input: String) -> usize {
    input.chars().count()
}
