pub trait Refined {
    /// Called to validate
    fn check_pre(&self) -> bool;
    fn check_post(&self) -> bool;
}

impl<T: Refined> Refined for &T {
    fn check_pre(&self) -> bool {
        <T as Refined>::check_pre(self)
    }

    fn check_post(&self) -> bool {
        <T as Refined>::check_post(self)
    }
}

impl<T: Refined> Refined for &mut T {
    fn check_pre(&self) -> bool {
        <T as Refined>::check_pre(self)
    }

    fn check_post(&self) -> bool {
        // This is intentional, not a mistake. On `&mut T`, `check_post` forwards to `check_pre`.
        <T as Refined>::check_pre(self)
    }
}

impl<T: Refined> Refined for Option<T> {
    fn check_pre(&self) -> bool {
        match self {
            Some(inner) => inner.check_pre(),
            None => true,
        }
    }

    fn check_post(&self) -> bool {
        match self {
            Some(inner) => inner.check_post(),
            None => true,
        }
    }
}

impl<T: Refined, E: Refined> Refined for Result<T, E> {
    fn check_pre(&self) -> bool {
        match self {
            Ok(okay) => okay.check_pre(),
            Err(err) => err.check_pre(),
        }
    }

    fn check_post(&self) -> bool {
        match self {
            Ok(okay) => okay.check_post(),
            Err(err) => err.check_post(),
        }
    }
}

impl<T: Refined> Refined for Box<T> {
    fn check_pre(&self) -> bool {
        self.as_ref().check_pre()
    }

    fn check_post(&self) -> bool {
        self.as_ref().check_post()
    }
}

impl Refined for () {
    fn check_pre(&self) -> bool {
        true
    }

    fn check_post(&self) -> bool {
        true
    }
}

impl<T1: Refined> Refined for (T1,) {
    fn check_pre(&self) -> bool {
        self.0.check_pre()
    }

    fn check_post(&self) -> bool {
        self.0.check_post()
    }
}

impl<T1: Refined, T2: Refined> Refined for (T1, T2) {
    fn check_pre(&self) -> bool {
        self.0.check_pre() && self.1.check_pre()
    }

    fn check_post(&self) -> bool {
        self.0.check_post() && self.1.check_post()
    }
}

impl<T1: Refined, T2: Refined, T3: Refined> Refined for (T1, T2, T3) {
    fn check_pre(&self) -> bool {
        self.0.check_pre() && self.1.check_pre() && self.2.check_pre()
    }

    fn check_post(&self) -> bool {
        self.0.check_post() && self.1.check_post() && self.2.check_post()
    }
}

/// Implement `Refined` for concrete types, with `check_pre/post` always `true`.
macro_rules! tautological_refinement {
    ($($ty:ty),+ $(,)?) => {
        $(
            impl $crate::refinement::Refined for $ty {
                fn check_pre(&self) -> bool { true }
                fn check_post(&self) -> bool { true }
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
