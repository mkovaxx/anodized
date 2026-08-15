use anodized::spec;
use std::ops::Mul;

/// Encodes the concept of multiplicative inverse.
///
/// <https://en.wikipedia.org/wiki/Multiplicative_inverse>
#[spec]
pub trait MulInverse
where
    Self: Eq + Mul<Output = Self> + Sized,
    for<'a> &'a Self: Eq + Mul<Output = Self>,
{
    const ONE: Self;

    #[spec(
        ensures: |ref recip| [
            self * recip == Self::ONE,
            recip * self == Self::ONE,
        ],
    )]
    fn reciprocal(&self) -> Self;
}
