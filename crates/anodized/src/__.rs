//! Module for the internal use of `anodized_macros`.

pub fn eval<T>(cond: impl Fn() -> T) -> T {
    cond()
}
