//! Module for the internal use of `anodized_macros`.

/// Make the evaluated closure mutation-free by coercing it to `Fn`.
pub fn eval<T>(closure: impl Fn() -> T) -> T {
    closure()
}
