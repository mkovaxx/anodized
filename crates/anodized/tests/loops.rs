#![allow(clippy::needless_range_loop)]
use anodized::spec;

#[spec(
    inspects: ref output,
    ensures: [
        seq.iter().any(|elem| elem == output),
        seq.iter().all(|elem| elem <= output),
    ],
)]
pub fn find_maximum(seq: &[u8]) -> u8 {
    let mut max = 0;

    #[spec(
        maintains: seq[0..i].iter().all(|elem| elem <= &max),
    )]
    for i in 0..seq.len() {
        if seq[i] > max {
            max = seq[i]
        }
    }

    max
}

#[spec(
    requires: seq.is_sorted(),
    inspects: output,
    ensures: [
        output <= seq.len(),
        seq[0..output].iter().all(|item| item < value),
        seq[output..].iter().all(|item| item >= value),
    ],
)]
pub fn find_insert_position<T: Ord>(seq: &[T], value: &T) -> usize {
    let mut i = 0;

    #[spec(
        maintains: seq[0..i].iter().all(|item| item < value),
        decreases: seq.len() - i,
    )]
    while i < seq.len() && seq[i] < *value {
        i += 1;
    }

    i
}
