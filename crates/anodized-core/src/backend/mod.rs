pub mod function;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Backend {
    /// Anodized instrumentation with runtime checks.
    Default,
    /// Anodized instrumentation with no runtime checks.
    NoChecks,
}
