use syn::{
    FieldValue, Ident, Member, Token,
    parse::{Parse, ParseStream, Result},
    punctuated::Punctuated,
};

/// Raw spec fields, i.e. as they appear in the `#[spec(...)]` proc macro invocation.
///
/// Represents a syntactically well-formed but otherwise unvalidated set of `spec` fields.
///
/// It reuses Rust's grammar of fields inside a `struct` expression. For reference, see:
/// https://doc.rust-lang.org/reference/expressions/struct-expr.html#railroad-StructExprField
#[derive(Debug, Clone)]
pub struct SpecFields {
    pub fields: Punctuated<FieldValue, Token![,]>,
}

impl Parse for SpecFields {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(Self {
            fields: Punctuated::<FieldValue, Token![,]>::parse_terminated(input)?,
        })
    }
}

impl SpecFields {
    /// Check whether the spec fields are sorted correctly, ignoring unknown keywords.
    pub fn is_sorted(&self) -> bool {
        self.fields
            .iter()
            .map(|field| Keyword::from(&field.member))
            .filter(|keyword| !matches!(keyword, Keyword::Unknown(_)))
            .is_sorted()
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub enum Keyword {
    Unknown(Ident),
    Functional,
    Pure,
    Total,
    Deterministic,
    Effectfree,
    Infallible,
    Terminating,
    Requires,
    Maintains,
    Captures,
    // TODO: Remove `binds` and `inspects` before v0.6 is released.
    Binds,
    Inspects,
    Ensures,
    Decreases,
}

impl From<&Member> for Keyword {
    fn from(value: &Member) -> Self {
        use Keyword::*;
        match value {
            Member::Named(ident) if ident == "functional" => Functional,
            Member::Named(ident) if ident == "pure" => Pure,
            Member::Named(ident) if ident == "total" => Total,
            Member::Named(ident) if ident == "deterministic" => Deterministic,
            Member::Named(ident) if ident == "effectfree" => Effectfree,
            Member::Named(ident) if ident == "infallible" => Infallible,
            Member::Named(ident) if ident == "terminating" => Terminating,
            Member::Named(ident) if ident == "requires" => Requires,
            Member::Named(ident) if ident == "maintains" => Maintains,
            Member::Named(ident) if ident == "captures" => Captures,
            Member::Named(ident) if ident == "binds" => Binds,
            Member::Named(ident) if ident == "inspects" => Inspects,
            Member::Named(ident) if ident == "ensures" => Ensures,
            Member::Named(ident) if ident == "decreases" => Decreases,
            Member::Named(ident) => Unknown(ident.clone()),
            Member::Unnamed(index) => Unknown(Ident::new(&format!("{}", index.index), index.span)),
        }
    }
}

impl std::fmt::Display for Keyword {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Keyword::Unknown(ident) => write!(f, "{}", ident),
            Keyword::Functional => write!(f, "functional"),
            Keyword::Pure => write!(f, "pure"),
            Keyword::Total => write!(f, "total"),
            Keyword::Deterministic => write!(f, "deterministic"),
            Keyword::Effectfree => write!(f, "effectfree"),
            Keyword::Infallible => write!(f, "infallible"),
            Keyword::Terminating => write!(f, "terminating"),
            Keyword::Requires => write!(f, "requires"),
            Keyword::Maintains => write!(f, "maintains"),
            Keyword::Captures => write!(f, "captures"),
            Keyword::Binds => write!(f, "binds"),
            Keyword::Inspects => write!(f, "inspects"),
            Keyword::Ensures => write!(f, "ensures"),
            Keyword::Decreases => write!(f, "decreases"),
        }
    }
}
