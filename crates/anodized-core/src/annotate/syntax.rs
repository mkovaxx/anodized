use proc_macro2::Span;
use syn::{
    Attribute, Expr, Ident, Meta, Pat, Token,
    parse::{Parse, ParseStream, Result},
    parse_quote,
    punctuated::Punctuated,
    spanned::Spanned,
};

/// Raw representation of spec argument syntax.
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

/// Custom keywords for parsing. This allows us to use `requires`, `ensures`, etc.,
/// as if they were built-in Rust keywords during parsing.
pub mod kw {
    syn::custom_keyword!(requires);
    syn::custom_keyword!(maintains);
    syn::custom_keyword!(captures);
    syn::custom_keyword!(binds);
    syn::custom_keyword!(ensures);
}

#[derive(PartialEq, PartialOrd, Clone, Copy, Debug)]
pub enum ArgOrder {
    Requires,
    Maintains,
    Captures,
    Binds,
    Ensures,
}

/// An intermediate enum to help parse either a condition or a `binds` setting.
pub enum SpecArg {
    Requires {
        keyword: kw::requires,
        attrs: Vec<Attribute>,
        expr: Expr,
    },
    Ensures {
        keyword: kw::ensures,
        attrs: Vec<Attribute>,
        expr: Expr,
    },
    Maintains {
        keyword: kw::maintains,
        attrs: Vec<Attribute>,
        expr: Expr,
    },
    Captures {
        keyword: kw::captures,
        attrs: Vec<Attribute>,
        expr: Expr,
    },
    Binds {
        keyword: kw::binds,
        attrs: Vec<Attribute>,
        pattern: Pat,
    },
}

impl SpecArg {
    pub fn get_order(&self) -> ArgOrder {
        match self {
            SpecArg::Requires { .. } => ArgOrder::Requires,
            SpecArg::Maintains { .. } => ArgOrder::Maintains,
            SpecArg::Captures { .. } => ArgOrder::Captures,
            SpecArg::Binds { .. } => ArgOrder::Binds,
            SpecArg::Ensures { .. } => ArgOrder::Ensures,
        }
    }

    pub fn get_keyword_span(&self) -> Span {
        match self {
            SpecArg::Requires { keyword, .. } => keyword.span,
            SpecArg::Ensures { keyword, .. } => keyword.span,
            SpecArg::Maintains { keyword, .. } => keyword.span,
            SpecArg::Captures { keyword, .. } => keyword.span,
            SpecArg::Binds { keyword, .. } => keyword.span,
        }
    }
}

impl Parse for SpecArg {
    fn parse(input: ParseStream) -> Result<Self> {
        let attrs = input.call(Attribute::parse_outer)?;

        let lookahead = input.lookahead1();
        if lookahead.peek(kw::captures) {
            // Parse `captures: <captures>`
            let keyword = input.parse::<kw::captures>()?;
            input.parse::<Token![:]>()?;
            Ok(SpecArg::Captures {
                keyword,
                attrs,
                expr: input.parse()?,
            })
        } else if lookahead.peek(kw::binds) {
            // Parse `binds: <pattern>`
            let keyword = input.parse::<kw::binds>()?;
            input.parse::<Token![:]>()?;
            Ok(SpecArg::Binds {
                keyword,
                attrs,
                pattern: Pat::parse_single(input)?,
            })
        } else if lookahead.peek(kw::requires) {
            // Parse `requires: <conditions>`
            let keyword = input.parse::<kw::requires>()?;
            input.parse::<Token![:]>()?;
            Ok(SpecArg::Requires {
                keyword,
                attrs,
                expr: input.parse()?,
            })
        } else if lookahead.peek(kw::maintains) {
            // Parse `maintains: <conditions>`
            let keyword = input.parse::<kw::maintains>()?;
            input.parse::<Token![:]>()?;
            Ok(SpecArg::Maintains {
                keyword,
                attrs,
                expr: input.parse()?,
            })
        } else if lookahead.peek(kw::ensures) {
            // Parse `ensures: <conditions>`
            let keyword = input.parse::<kw::ensures>()?;
            input.parse::<Token![:]>()?;
            Ok(SpecArg::Ensures {
                keyword,
                attrs,
                expr: input.parse()?,
            })
        } else {
            Err(lookahead.error())
        }
    }
}
