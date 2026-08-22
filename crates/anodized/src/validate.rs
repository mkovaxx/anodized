pub trait Validated {
    /// Called to validate an input on entry or the output on exit.
    fn check_pre(&self) -> bool;
    /// Called to validate an input on exit.
    fn check_post(&self) -> bool;
}

impl<T: Validated> Validated for &T {
    fn check_pre(&self) -> bool {
        <T as Validated>::check_pre(self)
    }

    fn check_post(&self) -> bool {
        <T as Validated>::check_post(self)
    }
}

impl<T: Validated> Validated for &mut T {
    fn check_pre(&self) -> bool {
        <T as Validated>::check_pre(self)
    }

    fn check_post(&self) -> bool {
        // This is intentional, not a mistake. On `&mut T`, `check_post` forwards to `check_pre`.
        <T as Validated>::check_pre(self)
    }
}

impl<T: Validated> Validated for Option<T> {
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

impl<T: Validated, E: Validated> Validated for Result<T, E> {
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

impl<T: Validated> Validated for Box<T> {
    fn check_pre(&self) -> bool {
        self.as_ref().check_pre()
    }

    fn check_post(&self) -> bool {
        self.as_ref().check_post()
    }
}

impl Validated for () {
    fn check_pre(&self) -> bool {
        true
    }

    fn check_post(&self) -> bool {
        true
    }
}

impl<T1: Validated> Validated for (T1,) {
    fn check_pre(&self) -> bool {
        self.0.check_pre()
    }

    fn check_post(&self) -> bool {
        self.0.check_post()
    }
}

impl<T1: Validated, T2: Validated> Validated for (T1, T2) {
    fn check_pre(&self) -> bool {
        self.0.check_pre() && self.1.check_pre()
    }

    fn check_post(&self) -> bool {
        self.0.check_post() && self.1.check_post()
    }
}

impl<T1: Validated, T2: Validated, T3: Validated> Validated for (T1, T2, T3) {
    fn check_pre(&self) -> bool {
        self.0.check_pre() && self.1.check_pre() && self.2.check_pre()
    }

    fn check_post(&self) -> bool {
        self.0.check_post() && self.1.check_post() && self.2.check_post()
    }
}

/// Implement `Validated` for concrete types, with `check_pre/post` always `true`.
macro_rules! trivial_validation {
    ($($ty:ty),+ $(,)?) => {
        $(
            impl $crate::validate::Validated for $ty {
                fn check_pre(&self) -> bool { true }
                fn check_post(&self) -> bool { true }
            }
        )+
    };
}

#[rustfmt::skip]
trivial_validation!(
    bool,
    u8, u16, u32, u64, u128, usize,
    i8, i16, i32, i64, i128, isize,
    f32, f64,
    char, str, String,
);
