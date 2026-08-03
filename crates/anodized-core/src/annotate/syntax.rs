use proc_macro2::Span;
use syn::{
    Attribute, Expr, Ident, Token,
    parse::{Parse, ParseStream, Result},
    punctuated::Punctuated,
};

/// Raw spec arguments, i.e. as they appear in the `#[spec(...)]` proc macro invocation.
///
/// Can represent a well-formed but invalid spec so that e.g. `anodized-fmt` may work with it.
#[derive(Debug, Clone)]
pub struct SpecArgs {
    pub args: Punctuated<SpecArg, Token![,]>,
}

impl Parse for SpecArgs {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(Self {
            args: Punctuated::<SpecArg, Token![,]>::parse_terminated(input)?,
        })
    }
}

impl SpecArgs {
    /// Check whether the spec arguments are sorted correctly, ignoring unknown keywords.
    pub fn is_sorted(&self) -> bool {
        self.args
            .iter()
            .filter(|arg| !matches!(arg.keyword, Keyword::Unknown(_)))
            .is_sorted_by_key(|arg| &arg.keyword)
    }
}

/// A single spec argument.
#[derive(Debug, Clone)]
pub struct SpecArg {
    pub attrs: Vec<Attribute>,
    pub keyword: Keyword,
    pub keyword_span: Span,
    pub colon: Option<Token![:]>,
    pub value: Option<Expr>,
}

impl Parse for SpecArg {
    fn parse(input: ParseStream) -> Result<Self> {
        let attrs = input.call(Attribute::parse_outer)?;
        let (keyword, keyword_span) = Keyword::parse(input)?;

        let (colon, value) = if input.peek(Token![:]) {
            (input.parse()?, Some(input.parse()?))
        } else {
            (None, None)
        };

        Ok(Self {
            attrs,
            keyword,
            keyword_span,
            colon,
            value,
        })
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
    // TODO: Remove `binds` before v0.6 is released.
    Binds,
    Inspects,
    Ensures,
    Decreases,
}

impl Keyword {
    fn parse(input: ParseStream) -> Result<(Self, Span)> {
        use Keyword::*;
        Ok(if input.peek(kw::functional) {
            let keyword: kw::functional = input.parse()?;
            (Functional, keyword.span)
        } else if input.peek(kw::pure) {
            let keyword: kw::pure = input.parse()?;
            (Pure, keyword.span)
        } else if input.peek(kw::total) {
            let keyword: kw::total = input.parse()?;
            (Total, keyword.span)
        } else if input.peek(kw::deterministic) {
            let keyword: kw::deterministic = input.parse()?;
            (Deterministic, keyword.span)
        } else if input.peek(kw::effectfree) {
            let keyword: kw::effectfree = input.parse()?;
            (Effectfree, keyword.span)
        } else if input.peek(kw::infallible) {
            let keyword: kw::infallible = input.parse()?;
            (Infallible, keyword.span)
        } else if input.peek(kw::terminating) {
            let keyword: kw::terminating = input.parse()?;
            (Terminating, keyword.span)
        } else if input.peek(kw::requires) {
            let keyword: kw::requires = input.parse()?;
            (Requires, keyword.span)
        } else if input.peek(kw::maintains) {
            let token: kw::maintains = input.parse()?;
            (Maintains, token.span)
        } else if input.peek(kw::captures) {
            let token: kw::captures = input.parse()?;
            (Captures, token.span)
        } else if input.peek(kw::binds) {
            let token: kw::binds = input.parse()?;
            (Binds, token.span)
        } else if input.peek(kw::inspects) {
            let token: kw::inspects = input.parse()?;
            (Inspects, token.span)
        } else if input.peek(kw::ensures) {
            let token: kw::ensures = input.parse()?;
            (Ensures, token.span)
        } else if input.peek(kw::decreases) {
            let token: kw::decreases = input.parse()?;
            (Decreases, token.span)
        } else {
            let ident: Ident = input.parse()?;
            let span = ident.span();
            (Unknown(ident), span)
        })
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
