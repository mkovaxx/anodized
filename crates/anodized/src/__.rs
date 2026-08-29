//! Module for the internal use of `anodized_macros`.

/// Make the evaluated closure mutation-free by coercing it to `Fn`.
pub fn eval<T>(closure: impl Fn() -> T) -> T {
    closure()
}

/// Coerce the evaluated closure to `FnOnce`.
///
/// For details, see: <https://github.com/anodized-rs/anodized/issues/201>
pub fn eval_once<T>(closure: impl FnOnce() -> T) -> T {
    closure()
}
