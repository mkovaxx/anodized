#![cfg_attr(anodized_charon, feature(register_tool))]
#![cfg_attr(anodized_charon, register_tool(charon))]

use anodized::{spec, types::Spec};

#[spec(
    maintains: match self {
        Ascending(vec) => vec.is_sorted(),
        Descending(vec) => vec.iter().rev().is_sorted(),
    }
)]
pub enum MonotonicVec<T: Ord> {
    Ascending(Vec<T>),
    Descending(Vec<T>),
}

#[spec(
    maintains: match self {
        Small { count, .. } => *count <= UNBOXED_CAPACITY,
        Large(vec) => vec.len() > UNBOXED_CAPACITY,
    }
)]
pub enum SmallVec<T: Default + Spec, const UNBOXED_CAPACITY: usize = 128> {
    Small {
        count: usize,
        buffer: Spec!([T; UNBOXED_CAPACITY]),
    },
    Large(Vec<T>),
}
