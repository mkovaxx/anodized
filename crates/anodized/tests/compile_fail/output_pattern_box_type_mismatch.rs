#![no_main]

use anodized::spec;

#[spec(ensures: |(a, b)| a <= b)]
fn sort_pair_box(pair: Box<(i32, i32)>) -> Box<(i32, i32)> {
    pair
}
