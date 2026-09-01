#[diagnostic::on_unimplemented(
    label = "type at the boundary of a `#[spec]`",
    message = "\
data spec enforcement needs `{Self}` to implement trait `anodized::data::Refine`",
    note = "\
if `{Self}` is a concrete local type, place a `#[spec]` attribute on its definition",
    note = "\
if `{Self}` is a concrete foreign type, wrap it in a local type such as `struct NewType({Self})`",
    note = "\
if `{Self}` is a type parameter, restrict it with the trait `{Self}: Refine`",
    note = "\
*UNSAFE*: alternatively, use `#[uncheck]` to locally disable data spec enforcement here"
)]
pub trait Refine {
    fn predicate(&self) -> bool;
}

impl<T: Refine> Refine for &T {
    fn predicate(&self) -> bool {
        <T as Refine>::predicate(self)
    }
}

impl<T: Refine> Refine for &mut T {
    fn predicate(&self) -> bool {
        <T as Refine>::predicate(self)
    }
}

impl<T: Refine> Refine for Option<T> {
    fn predicate(&self) -> bool {
        match self {
            Some(inner) => inner.predicate(),
            None => true,
        }
    }
}

impl<T: Refine, E: Refine> Refine for Result<T, E> {
    fn predicate(&self) -> bool {
        match self {
            Ok(okay) => okay.predicate(),
            Err(err) => err.predicate(),
        }
    }
}

impl<T: Refine> Refine for Box<T> {
    fn predicate(&self) -> bool {
        self.as_ref().predicate()
    }
}

impl Refine for () {
    fn predicate(&self) -> bool {
        true
    }
}

impl<T1: Refine> Refine for (T1,) {
    fn predicate(&self) -> bool {
        self.0.predicate()
    }
}

impl<T1: Refine, T2: Refine> Refine for (T1, T2) {
    fn predicate(&self) -> bool {
        self.0.predicate() && self.1.predicate()
    }
}

impl<T1: Refine, T2: Refine, T3: Refine> Refine for (T1, T2, T3) {
    fn predicate(&self) -> bool {
        self.0.predicate() && self.1.predicate() && self.2.predicate()
    }
}

/// Implement `Refine` for concrete types, with `predicate` always `true`.
macro_rules! trivial_refinement {
    ($($ty:ty),+ $(,)?) => {
        $(
            impl $crate::data::Refine for $ty {
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
