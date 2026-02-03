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
        cfg: Option<Meta>,
        expr: Expr,
    },
    Ensures {
        keyword: kw::ensures,
        cfg: Option<Meta>,
        expr: Expr,
    },
    Maintains {
        keyword: kw::maintains,
        cfg: Option<Meta>,
        expr: Expr,
    },
    Captures {
        keyword: kw::captures,
        expr: Expr,
    },
    Binds {
        keyword: kw::binds,
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
        let cfg = super::parse_cfg_attribute(&attrs)?;

        let lookahead = input.lookahead1();
        if lookahead.peek(kw::captures) {
            if cfg.is_some() {
                return Err(syn::Error::new(
                    attrs[0].span(),
                    "`cfg` attribute is not supported on `captures`",
                ));
            }

            // Parse `captures: <captures>`
            let keyword = input.parse::<kw::captures>()?;
            input.parse::<Token![:]>()?;
            Ok(SpecArg::Captures {
                keyword,
                expr: input.parse()?,
            })
        } else if lookahead.peek(kw::binds) {
            if cfg.is_some() {
                return Err(syn::Error::new(
                    attrs[0].span(),
                    "`cfg` attribute is not supported on `binds`",
                ));
            }

            // Parse `binds: <pattern>`
            let keyword = input.parse::<kw::binds>()?;
            input.parse::<Token![:]>()?;
            Ok(SpecArg::Binds {
                keyword,
                pattern: Pat::parse_single(input)?,
            })
        } else if lookahead.peek(kw::requires) {
            // Parse `requires: <conditions>`
            let keyword = input.parse::<kw::requires>()?;
            input.parse::<Token![:]>()?;
            Ok(SpecArg::Requires {
                keyword,
                cfg,
                expr: input.parse()?,
            })
        } else if lookahead.peek(kw::maintains) {
            // Parse `maintains: <conditions>`
            let keyword = input.parse::<kw::maintains>()?;
            input.parse::<Token![:]>()?;
            Ok(SpecArg::Maintains {
                keyword,
                cfg,
                expr: input.parse()?,
            })
        } else if lookahead.peek(kw::ensures) {
            // Parse `ensures: <conditions>`
            let keyword = input.parse::<kw::ensures>()?;
            input.parse::<Token![:]>()?;
            Ok(SpecArg::Ensures {
                keyword,
                cfg,
                expr: input.parse()?,
            })
        } else {
            Err(lookahead.error())
        }
    }
}
