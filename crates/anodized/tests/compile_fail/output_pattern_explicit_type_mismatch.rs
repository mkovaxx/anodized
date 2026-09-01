#![no_main]

use anodized::spec;

struct SomeType;

#[spec(ensures: |_: SomeType| true)]
fn count_chars(input: String) -> usize {
    input.chars().count()
}
