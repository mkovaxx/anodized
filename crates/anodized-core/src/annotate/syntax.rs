use proc_macro2::Span;
use syn::{
    Attribute, Expr, ExprCast, ExprPath, Ident, Pat, Token,
    parse::{Parse, ParseStream, Result},
    punctuated::Punctuated,
    token,
};

/// Raw spec arguments, i.e. as they appear in the `#[spec(...)]` proc macro invocation.
///
/// Can represent a well-formed but invalid spec so that e.g. `anodized-fmt` may work with it.
///
/// Can also contain representations items which are not strictly top level rust
/// expressions corresponding to [`syn::Expr`], see [`SpecArgValue`]
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
            Keyword::Captures => SpecArgValue::Captures(input.parse()?),
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

/// Each [`SpecArg`]'s value needs to be parsed in a way that allows invalid specs
/// and non top level rust expressions corresponding to [`syn::Expr`].
///
/// Some [SpecArgs](SpecArg) are not full rust expressions and instead components of
/// expressions such as the match arms of a [`syn::ExprMatch`] in capture lists
/// or [`syn::Pat`] in binds.
#[derive(Debug, Clone)]
pub enum SpecArgValue {
    Expr(Expr),
    Pat(Pat),
    Captures(CaptureList),
}

impl SpecArgValue {
    /// Return the `Expr` or fail.
    pub fn try_into_expr(self) -> Result<Expr> {
        match self {
            Self::Expr(expr) => Ok(expr),
            Self::Pat(pat) => Err(syn::Error::new_spanned(pat, "expected an expression")),
            Self::Captures(_) => Err(syn::Error::new(
                proc_macro2::Span::call_site(),
                "expected an expression, got captures",
            )),
        }
    }

    /// Return the `Pat` or fail.
    pub fn try_into_pat(self) -> Result<Pat> {
        match self {
            Self::Pat(pat) => Ok(pat),
            Self::Expr(expr) => Err(syn::Error::new_spanned(
                expr,
                "expected a pattern, got an expression",
            )),
            Self::Captures(_) => Err(syn::Error::new(
                proc_macro2::Span::call_site(),
                "expected a pattern, got captures",
            )),
        }
    }

    /// Return the `CaptureList` or fail.
    pub fn try_into_captures(self) -> Result<CaptureList> {
        match self {
            Self::Captures(list) => Ok(list),
            Self::Expr(expr) => Err(syn::Error::new_spanned(expr, "expected captures")),
            Self::Pat(pat) => Err(syn::Error::new_spanned(pat, "expected captures")),
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

/// A list of capture expressions, either a single one or an array.
/// These are not composed of top level [`syn::Expr`] expressions.
#[derive(Debug, Clone)]
pub enum CaptureList {
    Single(CaptureExpr),
    Array {
        bracket: token::Bracket,
        elems: Punctuated<CaptureExpr, Token![,]>,
    },
}

impl Parse for CaptureList {
    fn parse(input: ParseStream) -> Result<Self> {
        use syn::parse::discouraged::Speculative;

        // For bracketed input, we need to distinguish between:
        // 1. `[a, b, c]` - an array of capture expressions
        // 2. `[a, b, c] as slice` - a single capture with an array expr
        //
        // If it starts with a bracket, peek ahead to see if there's an `as` after the bracket
        if input.peek(token::Bracket) {
            let fork = input.fork();
            // Try to parse as a single expression with cast (e.g., `[a, b, c] as slice`)
            if let Ok(capture) = fork.parse::<CaptureExpr>() {
                // Only use this parse if it's a Cast (has `as` alias) or more complex
                match &capture {
                    CaptureExpr::Cast(_) | CaptureExpr::Expr(Expr::Cast(_)) => {
                        input.advance_to(&fork);
                        return Ok(CaptureList::Single(capture));
                    }
                    _ => {}
                }
            }
            // Otherwise, parse as an array of captures
            let content;
            let bracket = syn::bracketed!(content in input);
            let elems = Punctuated::parse_terminated(&content)?;
            Ok(CaptureList::Array { bracket, elems })
        } else {
            Ok(CaptureList::Single(input.parse()?))
        }
    }
}

/// An expression in a `capture` block which can either be a Cast, Ident, or a cast like pattern
/// match.
#[derive(Debug, Clone)]
pub enum CaptureExpr {
    Ident(ExprPath),
    Cast(ExprCast),
    Pattern(Pat),
    /// A general expression (for error reporting on complex expressions without alias)
    Expr(Expr),
}

impl Parse for CaptureExpr {
    fn parse(input: ParseStream) -> Result<Self> {
        use syn::parse::discouraged::Speculative;

        // Try `ident as pattern` syntax first
        // Parse: identifier, `as` keyword, then a pattern
        {
            let fork = input.fork();
            if let Ok(ident) = fork.parse::<Ident>()
                && fork.peek(Token![as])
            {
                let _: Token![as] = fork.parse().unwrap();
                // Try to parse the RHS as a pattern
                if let Ok(pat) = Pat::parse_single(&fork) {
                    // Only accept if it's a complex pattern (not just an identifier)
                    // Simple identifiers like `x as alias` should use Cast path
                    let is_complex_pattern = !matches!(&pat, Pat::Ident(p) if p.subpat.is_none());
                    if is_complex_pattern {
                        input.advance_to(&fork);
                        // Reconstruct as Pat::Ident with subpat
                        let pat_ident = syn::PatIdent {
                            attrs: vec![],
                            by_ref: None,
                            mutability: None,
                            ident,
                            subpat: Some((syn::token::At::default(), Box::new(pat))),
                        };
                        return Ok(CaptureExpr::Pattern(Pat::Ident(pat_ident)));
                    }
                }
            }
        }

        // Try Cast, but only accept if the RHS is a simple identifier.
        // This handles `expr as alias` syntax for simple aliasing.
        {
            let fork = input.fork();
            if let Ok(cast) = fork.parse::<ExprCast>()
                && let syn::Type::Path(ref type_path) = *cast.ty
                    && type_path.qself.is_none()
                        && type_path.path.leading_colon.is_none()
                        && type_path.path.segments.len() == 1
                        && type_path.path.segments[0].arguments.is_none()
                    {
                        input.advance_to(&fork);
                        return Ok(CaptureExpr::Cast(cast));
                    }
        }

        // Try ExprPath (e.g., `foo` or `foo::bar`)
        // Only accept if it's a complete expression (followed by comma or EOF)
        // Note: Do NOT accept if followed by `[` as that's indexing (e.g., `foo[0]`)
        {
            let fork = input.fork();
            if let Ok(path) = fork.parse::<ExprPath>() {
                // Check if the path is the complete expression
                if fork.is_empty() || fork.peek(Token![,]) {
                    input.advance_to(&fork);
                    return Ok(CaptureExpr::Ident(path));
                }
            }
        }

        // Fall back to general Expr (will be validated later for alias requirement)
        Ok(CaptureExpr::Expr(input.parse()?))
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

#[derive(PartialEq, PartialOrd, Clone, Debug)]
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
