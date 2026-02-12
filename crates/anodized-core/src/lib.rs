#![doc = include_str!("../README.md")]

use proc_macro2::Span;
use syn::{Expr, Meta, Pat};

pub mod annotate;
pub mod instrument;

#[cfg(test)]
mod test_util;

/// Specifies the intended behavior of a function or method.
#[derive(Debug)]
pub struct Spec {
    /// Preconditions: conditions that must hold when the function is called.
    pub requires: Vec<PreCondition>,
    /// Invariants: conditions that must hold both when the function is called and when it returns.
    pub maintains: Vec<PreCondition>,
    /// Captures: expressions to snapshot at function entry for use in postconditions.
    pub captures: Vec<Capture>,
    /// Postconditions: conditions that must hold when the function returns.
    pub ensures: Vec<PostCondition>,
    /// The span in the source code, from which this spec was parsed
    span: Span,
}

impl Spec {
    /// Returns `true` if the spec contract is empty (specifies nothing), otherwise returns `false`
    pub fn is_empty(&self) -> bool {
        self.requires.is_empty()
            && self.maintains.is_empty()
            && self.ensures.is_empty()
            && self.captures.is_empty()
    }
    /// Call to construct an error from the whole spec
    pub fn spec_err(&self, message: &str) -> syn::Error {
        syn::Error::new::<&str>(self.span, message)
    }
}

/// A precondition represented by a `bool`-valued expression.
#[derive(Debug)]
pub struct PreCondition {
    /// The closure that validates the precondition,
    /// takes no input, e.g. `|| input.is_valid()`.
    pub closure: syn::ExprClosure,
    /// **Static analyzers can safely ignore this field.**
    ///
    /// Build configuration filter to decide whether to add runtime checks.
    /// Passed to a `cfg!()` guard in the instrumented function.
    pub cfg: Option<Meta>,
}

/// A postcondition represented by a closure that takes the return value as a reference.
#[derive(Debug)]
pub struct PostCondition {
    /// The closure that validates the postcondition, taking the function's
    /// return value by reference, e.g. `|output| *output > 0`.
    pub closure: syn::ExprClosure,
    /// **Static analyzers can safely ignore this field.**
    ///
    /// Build configuration filter to decide whether to add runtime checks.
    /// Passed to a `cfg!()` guard in the instrumented function.
    pub cfg: Option<Meta>,
}

/// Captures an expression's value at function entry.
#[derive(Debug)]
pub struct Capture {
    /// The expression to capture.
    pub expr: Expr,
    /// The pattern to bind/destructure the captured value.
    pub pat: Pat,
}
