use num_bigint::BigInt;

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
#[repr(transparent)]
pub struct int(BigInt);

crate::trivial_refinement!(int);

mod interop;
use interop::impl_primitive_interop;

pub trait Primitive: Copy {}

impl_primitive_interop!(i8);
impl_primitive_interop!(i16);
impl_primitive_interop!(i32);
impl_primitive_interop!(i64);
impl_primitive_interop!(i128);
impl_primitive_interop!(isize);

impl_primitive_interop!(u8);
impl_primitive_interop!(u16);
impl_primitive_interop!(u32);
impl_primitive_interop!(u64);
impl_primitive_interop!(u128);
impl_primitive_interop!(usize);

impl ::core::ops::Add<int> for int {
    type Output = int;

    #[inline]
    fn add(self, rhs: int) -> int {
        int(self.0 + rhs.0)
    }
}

impl ::core::ops::Sub<int> for int {
    type Output = int;

    #[inline]
    fn sub(self, rhs: int) -> int {
        int(self.0 - rhs.0)
    }
}

impl ::core::ops::Mul<int> for int {
    type Output = int;

    #[inline]
    fn mul(self, rhs: int) -> int {
        int(self.0 * rhs.0)
    }
}

impl ::core::ops::Div<int> for int {
    type Output = int;

    #[inline]
    fn div(self, rhs: int) -> int {
        int(self.0 / rhs.0)
    }
}

impl ::core::ops::Rem<int> for int {
    type Output = int;

    #[inline]
    fn rem(self, rhs: int) -> int {
        int(self.0 % rhs.0)
    }
}
