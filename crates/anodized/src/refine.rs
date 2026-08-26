pub trait Refined {
    fn predicate(&self) -> bool;
}

impl<T: Refined> Refined for &T {
    fn predicate(&self) -> bool {
        <T as Refined>::predicate(self)
    }
}

impl<T: Refined> Refined for &mut T {
    fn predicate(&self) -> bool {
        <T as Refined>::predicate(self)
    }
}

impl<T: Refined> Refined for Option<T> {
    fn predicate(&self) -> bool {
        match self {
            Some(inner) => inner.predicate(),
            None => true,
        }
    }
}

impl<T: Refined, E: Refined> Refined for Result<T, E> {
    fn predicate(&self) -> bool {
        match self {
            Ok(okay) => okay.predicate(),
            Err(err) => err.predicate(),
        }
    }
}

impl<T: Refined> Refined for Box<T> {
    fn predicate(&self) -> bool {
        self.as_ref().predicate()
    }
}

impl Refined for () {
    fn predicate(&self) -> bool {
        true
    }
}

impl<T1: Refined> Refined for (T1,) {
    fn predicate(&self) -> bool {
        self.0.predicate()
    }
}

impl<T1: Refined, T2: Refined> Refined for (T1, T2) {
    fn predicate(&self) -> bool {
        self.0.predicate() && self.1.predicate()
    }
}

impl<T1: Refined, T2: Refined, T3: Refined> Refined for (T1, T2, T3) {
    fn predicate(&self) -> bool {
        self.0.predicate() && self.1.predicate() && self.2.predicate()
    }
}

/// Implement `Refined` for concrete types, with `predicate` always `true`.
macro_rules! trivial_refinement {
    ($($ty:ty),+ $(,)?) => {
        $(
            impl $crate::refine::Refined for $ty {
                fn predicate(&self) -> bool { true }
            }
        )+
    };
}

#[rustfmt::skip]
trivial_refinement!(
    bool,
    u8, u16, u32, u64, u128, usize,
    i8, i16, i32, i64, i128, isize,
    f32, f64,
    char, str, String,
);
