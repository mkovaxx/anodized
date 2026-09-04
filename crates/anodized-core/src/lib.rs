#![doc = include_str!("../README.md")]

use proc_macro2::Span;
use syn::{Error, Expr, Meta, Pat};

use crate::qualifiers::FnQualifiers;

pub mod annotate;
pub mod instrument;
pub mod qualifiers;
pub mod syntax;

#[cfg(test)]
mod test_util;

/// Mandates that an AST node's `#[spec]` attribute be empty, i.e. contain no fields.
#[derive(Debug)]
pub struct EmptySpec;

/// Specifies the intended behavior of a function or method: `fn`.
#[derive(Debug)]
pub struct FnSpec {
    /// Qualifiers that constrain the behavior of the computation.
    pub qualifiers: FnQualifiers,
    /// Whether each input satisfies its type spec on entry/exit.
    pub input_specs: Vec<InputSpec>,
    /// Whether the output satisfies its type spec on exit.
    pub output_spec_on_exit: bool,
    /// Preconditions: conditions that must hold when the function is called.
    pub requires: Vec<Condition>,
    /// Invariants: conditions that must hold both when the function is called and when it returns.
    pub maintains: Vec<Condition>,
    /// Captures: expressions to snapshot at function entry for use in postconditions.
    pub captures: Vec<Capture>,
    /// Postconditions: conditions that must hold when the function returns.
    pub ensures: Vec<PostCondition>,
    /// The span in the source code, from which this spec was parsed.
    span: Span,
}

/// Determines where the input in a `fn` signature satisfies its type spec.
#[derive(Debug)]
pub struct InputSpec {
    /// Whether the input satisfies its type spec on entry.
    pub on_entry: bool,
    /// Whether the input satisfies its type spec on exit.
    pub on_exit: bool,
}

impl FnSpec {
    /// Returns `true` if the spec is empty (specifies nothing), otherwise returns `false`.
    pub fn is_empty(&self) -> bool {
        self.qualifiers.is_empty()
            && self.requires.is_empty()
            && self.maintains.is_empty()
            && self.ensures.is_empty()
            && self.captures.is_empty()
            && !self.output_spec_on_exit
            && !self
                .input_specs
                .iter()
                .any(|input_spec| input_spec.on_entry || input_spec.on_exit)
    }

    /// Construct an error from the whole spec.
    pub fn spec_err(&self, message: &str) -> Error {
        Error::new::<&str>(self.span, message)
    }
}

impl Default for InputSpec {
    fn default() -> Self {
        Self {
            on_entry: true,
            on_exit: true,
        }
    }
}

/// Specifies the intended behavior of a data type: `struct` or `enum`.
#[derive(Debug)]
pub struct DataSpec {
    /// Whether each field satisfies its type spec. Variant index first, field index second.
    pub field_specs: Vec<Vec<bool>>,
    /// Invariants: conditions that must hold for all instances of the data type.
    pub maintains: Vec<Condition>,
    /// The span in the source code, from which this spec was parsed.
    span: Span,
}

impl DataSpec {
    /// Returns `true` if the spec is empty (specifies nothing), otherwise returns `false`.
    pub fn is_empty(&self) -> bool {
        self.maintains.is_empty()
            && !self
                .field_specs
                .iter()
                .flatten()
                .any(|field_spec| *field_spec)
    }

    /// Construct an error from the whole spec.
    pub fn spec_err(&self, message: &str) -> Error {
        Error::new::<&str>(self.span, message)
    }
}

/// Specifies the intended behavior of a loop: `while` or `for`.
#[derive(Debug)]
pub struct LoopSpec {
    /// Loop invariants: conditions that must hold both before and after the loop's body runs.
    pub maintains: Vec<Condition>,
    /// Loop variant: an expression that decreases with each run of the loop's body.
    pub decreases: Option<LoopVariant>,
    /// The span in the source code, from which this spec was parsed.
    span: Span,
}

impl LoopSpec {
    /// Empty spec that contains no elements.
    pub fn empty() -> Self {
        Self {
            maintains: vec![],
            decreases: None,
            span: Span::call_site(),
        }
    }

    /// Returns `true` if the spec is empty (specifies nothing), otherwise returns `false`.
    pub fn is_empty(&self) -> bool {
        self.maintains.is_empty() && self.decreases.is_none()
    }

    /// Construct an error from the whole spec.
    pub fn spec_err(&self, message: &str) -> Error {
        Error::new::<&str>(self.span, message)
    }
}

/// A condition represented by a `bool`-valued expression.
#[derive(Debug)]
pub struct Condition {
    /// The expression that validates the condition, e.g. `value > 42`.
    pub expr: Expr,
    /// **Static analyzers can safely ignore this field.**
    ///
    /// Build configuration filter to decide whether to add runtime checks.
    /// Passed to a `cfg!()` guard in the instrumented function.
    pub cfg: Option<Meta>,
}

/// A postcondition represented by a pattern to bind the output and a `bool`-valued expression.
#[derive(Debug)]
pub struct PostCondition {
    /// The pattern to bind or destructure the function's output, e.g. `answer` or `(left, right)`.
    pub pat: Option<Pat>,
    /// The expression that validates the postcondition, e.g. `answer == "forty-two"`.
    pub expr: Expr,
    /// **Static analyzers can safely ignore this field.**
    ///
    /// Build configuration filter to decide whether to add runtime checks.
    /// Passed to a `cfg!()` guard in the instrumented function.
    pub cfg: Option<Meta>,
}

/// Captures an expression's value at function entry.
#[derive(Debug)]
pub struct Capture {
    /// The pattern to bind/destructure the captured value.
    pub pat: Pat,
    /// The expression to capture.
    pub expr: Expr,
}

/// Decreases with each run of a loop's body.
#[derive(Debug)]
pub struct LoopVariant {
    /// The expression that defines the variant.
    pub expr: Expr,
    /// **Static analyzers can safely ignore this field.**
    ///
    /// Build configuration filter to decide whether to add runtime checks.
    /// Passed to a `cfg!()` guard in the instrumented code.
    pub cfg: Option<Meta>,
}
