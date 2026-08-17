pub trait Refined {
    fn is_valid(&self) -> bool;
}

impl<T: Refined> Refined for &T {
    fn is_valid(&self) -> bool {
        <T as Refined>::is_valid(self)
    }
}

impl<T: Refined> Refined for &mut T {
    fn is_valid(&self) -> bool {
        <T as Refined>::is_valid(self)
    }
}

impl<T: Refined> Refined for Option<T> {
    fn is_valid(&self) -> bool {
        match self {
            Some(inner) => inner.is_valid(),
            None => true,
        }
    }
}

impl<T: Refined, E: Refined> Refined for Result<T, E> {
    fn is_valid(&self) -> bool {
        match self {
            Ok(okay) => okay.is_valid(),
            Err(err) => err.is_valid(),
        }
    }
}

impl<T: Refined> Refined for Box<T> {
    fn is_valid(&self) -> bool {
        self.as_ref().is_valid()
    }
}

impl Refined for () {
    fn is_valid(&self) -> bool {
        true
    }
}

impl<T1: Refined> Refined for (T1,) {
    fn is_valid(&self) -> bool {
        self.0.is_valid()
    }
}

impl<T1: Refined, T2: Refined> Refined for (T1, T2) {
    fn is_valid(&self) -> bool {
        self.0.is_valid() && self.1.is_valid()
    }
}

impl<T1: Refined, T2: Refined, T3: Refined> Refined for (T1, T2, T3) {
    fn is_valid(&self) -> bool {
        self.0.is_valid() && self.1.is_valid() && self.2.is_valid()
    }
}

/// Implement `Refined` for concrete types, with `is_valid` always `true`.
macro_rules! tautological_refinement {
    ($($ty:ty),+ $(,)?) => {
        $(
            impl $crate::refinement::Refined for $ty {
                fn is_valid(&self) -> bool { true }
            }
        )+
    };
}

#[rustfmt::skip]
tautological_refinement!(
    bool,
    u8, u16, u32, u64, u128, usize,
    i8, i16, i32, i64, i128, isize,
    f32, f64,
    char, str, String,
);
