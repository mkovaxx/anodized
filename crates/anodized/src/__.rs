//! Module for the internal use of `anodized_macros`.

/// Coerce the closure's input type based on a reference.
pub fn coerce_input<T>(_: impl FnOnce(T), _: &T) {}

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
