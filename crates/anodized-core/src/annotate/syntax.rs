use proc_macro2::Span;
use syn::{
    Attribute, Expr, Ident, Pat, Token,
    parse::{Parse, ParseStream, Result},
    punctuated::Punctuated,
};

/// Raw spec arguments, i.e. as they appear in the `#[spec(...)]` proc macro invocation.
///
/// Can represent a well-formed but invalid spec so that e.g. `anodized-fmt` may work with it.
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

/// A single spec argument.
pub struct SpecArg {
    pub attrs: Vec<Attribute>,
    pub keyword: Keyword,
    pub keyword_span: Span,
    pub colon: Token![:],
    pub value: SpecArgValue,
}

impl Parse for SpecArg {
    fn parse(input: ParseStream) -> Result<Self> {
        let attrs = input.call(Attribute::parse_outer)?;
        let (keyword, keyword_span) = Keyword::parse(input)?;
        let colon = input.parse()?;
        let value = match keyword {
            Keyword::Binds => SpecArgValue::parse_pat_or_expr(input)?,
            _ => SpecArgValue::parse_expr_or_pat(input)?,
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

/// Each SpecArg's value needs to be parsed in a way that allows invalid specs.
#[derive(Debug, Clone)]
pub enum SpecArgValue {
    Expr(Expr),
    Pat(Pat),
}

impl SpecArgValue {
    /// Return the `Expr` or fail.
    pub fn try_into_expr(self) -> Result<Expr> {
        match self {
            Self::Expr(expr) => Ok(expr),
            Self::Pat(pat) => Err(syn::Error::new_spanned(pat, "expected an expression")),
        }
    }

    /// Return the `Pat` or fail.
    pub fn try_into_pat(self) -> Result<Pat> {
        match self {
            Self::Pat(pat) => Ok(pat),
            Self::Expr(expr) => Err(syn::Error::new_spanned(expr, "expected a pattern")),
        }
    }

    /// Try to parse as `Expr` then as `Pat`.
    fn parse_expr_or_pat(input: ParseStream) -> Result<Self> {
        if let Ok(expr) = Self::parse_expr_or_nothing(input) {
            Ok(Self::Expr(expr))
        } else if let Ok(pat) = Self::parse_pat_or_nothing(input) {
            Ok(Self::Pat(pat))
        } else {
            Err(input.error("expected an expression or a pattern"))
        }
    }

    /// Try to parse as `Pat` then as `Expr`.
    fn parse_pat_or_expr(input: ParseStream) -> Result<Self> {
        if let Ok(pat) = Self::parse_pat_or_nothing(input) {
            Ok(Self::Pat(pat))
        } else if let Ok(expr) = Self::parse_expr_or_nothing(input) {
            Ok(Self::Expr(expr))
        } else {
            Err(input.error("expected a pattern or an expression"))
        }
    }

    /// Try to parse as `Expr` but consume no input on failure.
    fn parse_expr_or_nothing(input: ParseStream<'_>) -> Result<Expr> {
        use syn::parse::discouraged::Speculative;
        let fork = input.fork();
        match Expr::parse(&fork) {
            Ok(expr) => {
                input.advance_to(&fork);
                Ok(expr)
            }
            Err(err) => Err(err),
        }
    }

    /// Try to parse as `Pat` but consume no input on failure.
    fn parse_pat_or_nothing(input: ParseStream<'_>) -> Result<Pat> {
        use syn::parse::discouraged::Speculative;
        let fork = input.fork();
        match Pat::parse_single(&fork) {
            Ok(pat) => {
                input.advance_to(&fork);
                Ok(pat)
            }
            Err(err) => Err(err),
        }
    }
}

/// Custom keywords for parsing. This allows us to use `requires`, `ensures`, etc.,
/// as if they were built-in Rust keywords during parsing.
pub mod kw {
    syn::custom_keyword!(requires);
    syn::custom_keyword!(maintains);
    syn::custom_keyword!(captures);
    syn::custom_keyword!(binds);
    syn::custom_keyword!(ensures);
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Debug)]
pub enum Keyword {
    Unknown(Ident),
    Requires,
    Maintains,
    Captures,
    Binds,
    Ensures,
}

impl Keyword {
    fn parse(input: ParseStream) -> Result<(Self, Span)> {
        use Keyword::*;
        Ok(if input.peek(kw::requires) {
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
        } else if input.peek(kw::ensures) {
            let token: kw::ensures = input.parse()?;
            (Ensures, token.span)
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
            Keyword::Requires => write!(f, "requires"),
            Keyword::Maintains => write!(f, "maintains"),
            Keyword::Captures => write!(f, "captures"),
            Keyword::Binds => write!(f, "binds"),
            Keyword::Ensures => write!(f, "ensures"),
            Keyword::Unknown(ident) => write!(f, "{}", ident),
        }
    }
}
