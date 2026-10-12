#![no_main]

use anodized::spec;

#[spec(ensures: |(a, b)| a <= b)]
fn sort_pair_ref(pair: &mut (i32, i32)) -> &mut (i32, i32) {
    pair
}
