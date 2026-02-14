use proc_macro2::{Span, TokenStream, TokenTree};
use quote::{ToTokens, TokenStreamExt};
use syn::{
    Attribute, Expr, Ident, Pat, Token,
    parse::{Parse, ParseStream, Result},
    punctuated::Punctuated,
    token,
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
/// Each [`SpecArg`]'s value needs to be parsed in a way that allows invalid specs, e.g.
/// forms which do not correspond directly to an [`syn::Expr`] in standard Rust.
///
/// NOTE:
/// a [`SpecArgValue`] may hold unrelated syntactic elements such as ['syn::Expr`], [`syn::Pat`],
/// and even fragments that would never appear as part of a valid Rust program.
#[derive(Debug, Clone)]
pub enum SpecArgValue {
    Expr(Expr),
    Pat(Pat),
    Captures(Captures),
}

impl SpecArgValue {
    /// Return the `Expr` or fail.
    pub fn try_into_expr(self) -> Result<Expr> {
        if let Self::Expr(expr) = self {
            return Ok(expr);
        };
        Err(syn::Error::new_spanned(self, "expected an expression"))
    }

    /// Return the `Pat` or fail.
    pub fn try_into_pat(self) -> Result<Pat> {
        if let Self::Pat(pat) = self {
            return Ok(pat);
        };
        Err(syn::Error::new_spanned(self, "expected a pattern"))
    }

    /// Return the `Captures` or fail.
    pub fn try_into_captures(self) -> Result<Captures> {
        if let Self::Captures(captures) = self {
            return Ok(captures);
        };
        Err(syn::Error::new_spanned(
            self,
            "expected captures: expression `as` pattern",
        ))
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

impl ToTokens for SpecArgValue {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            SpecArgValue::Expr(expr) => expr.to_tokens(tokens),
            SpecArgValue::Pat(pat) => pat.to_tokens(tokens),
            SpecArgValue::Captures(captures) => captures.to_tokens(tokens),
        }
    }
}

/// A group of capture expressions, either a single one or a list.
/// These are not composed of top level [`syn::Expr`] expressions.
#[derive(Debug, Clone)]
pub enum Captures {
    One(CaptureExpr),
    Many {
        bracket: token::Bracket,
        elems: Punctuated<CaptureExpr, Token![,]>,
    },
}

impl Parse for Captures {
    fn parse(input: ParseStream) -> Result<Self> {
        // For bracketed input, we need to distinguish between:
        // 1. `[a, b, c]` - an array of capture expressions
        // 2. `[a, b, c] as slice` - a single capture with an array expr
        //
        // Multiple captures are in brackets, not followed by `as`
        if input.peek(token::Bracket) && !input.peek2(Token![as]) {
            // Parse as an array of captures
            let content;
            let bracket = syn::bracketed!(content in input);
            let elems = Punctuated::parse_terminated(&content)?;
            Ok(Captures::Many { bracket, elems })
        } else {
            // Otherwise parse as one capture
            Ok(Captures::One(input.parse()?))
        }
    }
}

impl ToTokens for Captures {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Self::One(capture_expr) => capture_expr.to_tokens(tokens),
            Self::Many { bracket, elems } => bracket.surround(tokens, |tokens| {
                elems.to_tokens(tokens);
            }),
        }
    }
}

/// The form in a `captures` clause: <expression> `as` <pattern>.
#[derive(Debug, Clone)]
pub struct CaptureExpr {
    pub expr: Option<Expr>,
    pub as_: Option<Token![as]>,
    pub pat: Option<Pat>,
}

impl ToTokens for CaptureExpr {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        if let Some(expr) = &self.expr {
            expr.to_tokens(tokens);
        }
        if let Some(as_) = &self.as_ {
            as_.to_tokens(tokens);
        }
        if let Some(pat) = &self.pat {
            pat.to_tokens(tokens);
        }
    }
}

impl Parse for CaptureExpr {
    fn parse(input: ParseStream) -> Result<Self> {
        let tokens_before_as = take_until_comma_or_last_as(input)?;
        let expr = syn::parse::Parser::parse2(Expr::parse, tokens_before_as).ok();
        // TODO: need to check that the entirety of `tokens_before_as` was consumed
        let as_ = if input.peek(Token![as]) {
            Some(input.parse()?)
        } else {
            None
        };
        let pat = SpecArgValue::parse_pat_or_nothing(input).ok();
        Ok(Self { expr, as_, pat })
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

/// Find the last `as` token that is before a comma (or end of input).
/// Consume and return tokens before the last `as`.
/// If no `as` is encountered, consume and return all tokens before a comma.
/// Groups (delimited by `()`, `[]`, `{}`) are considered atomically,
/// so any `as` or comma inside them is ignored.
fn take_until_comma_or_last_as(input: ParseStream) -> Result<TokenStream> {
    use syn::parse::discouraged::Speculative;
    let fork = input.fork();
    let mut peeked_tokens = TokenStream::new();
    let mut consumed_tokens = TokenStream::new();
    let mut has_seen_as = false;
    while !fork.is_empty() && !fork.peek(Token![,]) {
        if fork.peek(Token![as]) {
            has_seen_as = true;
            // Consumed peeked tokens
            consumed_tokens.extend(peeked_tokens);
            peeked_tokens = TokenStream::new();
            input.advance_to(&fork);
        }
        let token: TokenTree = fork.parse()?;
        peeked_tokens.append(token);
    }
    if has_seen_as {
        Ok(consumed_tokens)
    } else {
        input.advance_to(&fork);
        Ok(peeked_tokens)
    }
}
