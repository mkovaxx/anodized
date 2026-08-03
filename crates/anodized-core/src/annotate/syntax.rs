use syn::{
    FieldValue, Ident, Member, Token,
    parse::{Parse, ParseStream, Result},
    punctuated::Punctuated,
};

/// Raw spec arguments, i.e. as they appear in the `#[spec(...)]` proc macro invocation.
///
/// Can represent a well-formed but invalid spec so that e.g. `anodized-fmt` may work with it.
#[derive(Debug, Clone)]
pub struct SpecArgs {
    pub args: Punctuated<FieldValue, Token![,]>,
}

impl Parse for SpecArgs {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(Self {
            args: Punctuated::<FieldValue, Token![,]>::parse_terminated(input)?,
        })
    }
}

impl SpecArgs {
    /// Check whether the spec arguments are sorted correctly, ignoring unknown keywords.
    pub fn is_sorted(&self) -> bool {
        self.args
            .iter()
            .map(|field| Keyword::from(&field.member))
            .filter(|keyword| !matches!(keyword, Keyword::Unknown(_)))
            .is_sorted()
    }
}

/// Custom keywords for parsing. This allows us to use `requires`, `ensures`, etc.,
/// as if they were built-in Rust keywords during parsing.
pub mod kw {
    syn::custom_keyword!(functional);
    syn::custom_keyword!(pure);
    syn::custom_keyword!(total);
    syn::custom_keyword!(deterministic);
    syn::custom_keyword!(effectfree);
    syn::custom_keyword!(infallible);
    syn::custom_keyword!(terminating);
    syn::custom_keyword!(requires);
    syn::custom_keyword!(maintains);
    syn::custom_keyword!(captures);
    syn::custom_keyword!(binds);
    syn::custom_keyword!(inspects);
    syn::custom_keyword!(ensures);
    syn::custom_keyword!(decreases);
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
