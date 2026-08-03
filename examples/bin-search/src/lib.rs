use anodized::spec;

#[spec(
    requires: seq.is_sorted(),
    ensures: |output| match output {
        Some(index) => index < seq.len() && seq[index] == *value,
        None => seq.iter().all(|item| item != value),
    },
)]
pub fn bin_search<T: Ord>(seq: &[T], value: &T) -> Option<usize> {
    use std::cmp::Ordering::*;
    let mut lo = 0;
    let mut hi = seq.len();
    while lo < hi {
        let mid = lo + (hi - lo) / 2;
        match seq[mid].cmp(value) {
            Less => lo = mid + 1,
            Greater => hi = mid,
            Equal => return Some(mid),
        }
    }
    None
}
