/// Try to call a function with a [`spec`](crate::spec) and return a [`Result`].
///
/// The macro must wrap a call expression of the following supported cases:
/// ```ignore
/// // free function with a qualified name
/// let result = try_call! { qualified::free_fn(...) };
/// ```
///
/// ```ignore
/// // method
/// let result = try_call! { receiver.method(...) };
/// ```
///
/// ```ignore
/// // function qualified by a type or trait
/// let result = try_call! { Type::associated_fn(...) };
/// let result = try_call! { <Type>::associated_fn(...) };
/// let result = try_call! { <Type as Trait>::trait_fn(...) };
/// ```
pub use anodized_macros::try_call;

/// Return type of a call wrapped by [`try_call!`].
pub type Result<T> = std::result::Result<T, Error<T>>;

/// Construct a precondition failure.
pub fn pre_err<T>() -> Result<T> {
    Result::Err(Error::Pre)
}

/// Construct a postcondition failure.
pub fn post_err<T>(output: T) -> Result<T> {
    Result::Err(Error::Post(output))
}

pub use Error::Post as PostError;
pub use Error::Pre as PreError;

/// Error that represents a pre/postcondition failure.
pub enum Error<T> {
    /// Preconditions failed.
    Pre,
    /// Postconditions failed.
    Post(T),
}

pub type Messages = String;
