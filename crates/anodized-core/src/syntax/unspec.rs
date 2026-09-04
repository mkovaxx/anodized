use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::{
    AttrStyle, Attribute, Error, MacroDelimiter, Meta, MetaList, Path, Token,
    parse::{Parse, ParseStream, Result},
    token::{Bracket, Paren, Pound},
};

use crate::syntax::attr::path_matches_name;

/// Represents a valid `#[unspec]` attribute.
///
/// Has three valid forms:
/// - `#[unspec]`
/// - `#[unspec(in)]`
/// - `#[unspec(out)]`
#[derive(Debug, Clone)]
pub struct UnspecAttr {
    pub pound_token: Pound,
    pub bracket_token: Bracket,
    pub path: Path,
    pub arg: Option<(Paren, UnspecArg)>,
}

impl From<UnspecAttr> for Attribute {
    fn from(unspec: UnspecAttr) -> Self {
        Attribute {
            pound_token: unspec.pound_token,
            style: AttrStyle::Outer,
            bracket_token: unspec.bracket_token,
            meta: if let Some((paren, arg)) = unspec.arg {
                Meta::List(MetaList {
                    path: unspec.path,
                    delimiter: MacroDelimiter::Paren(paren),
                    tokens: arg.into_token_stream(),
                })
            } else {
                Meta::Path(unspec.path)
            },
        }
    }
}

impl TryFrom<Attribute> for UnspecAttr {
    type Error = Error;

    fn try_from(attr: Attribute) -> Result<Self> {
        if let AttrStyle::Inner(bang_token) = attr.style {
            return Err(syn::Error::new_spanned(
                bang_token,
                "expected an outer attribute (no `!`): `#[unspec]`",
            ));
        };

        let (path, arg) = match attr.meta {
            Meta::Path(path) if path_matches_name(&path, "unspec") => (path, None),
            Meta::List(list) if path_matches_name(&list.path, "unspec") => {
                let MacroDelimiter::Paren(paren) = list.delimiter else {
                    return Err(Error::new(
                        list.delimiter.span().open(),
                        "expected arguments in parentheses",
                    ));
                };
                let arg = list.parse_args()?;
                (list.path, Some((paren, arg)))
            }
            Meta::NameValue(key_val) if path_matches_name(&key_val.path, "unspec") => {
                return Err(Error::new_spanned(
                    key_val.eq_token,
                    "expected arguments in parentheses",
                ));
            }
            _ => {
                return Err(Error::new_spanned(
                    attr,
                    "expected an `#[unspec]` attribute",
                ));
            }
        };

        Ok(UnspecAttr {
            pound_token: attr.pound_token,
            bracket_token: attr.bracket_token,
            path,
            arg,
        })
    }
}

/// Represents valid arguments to the `#[unspec]` attribute.
#[derive(Debug, Clone, Copy)]
pub enum UnspecArg {
    /// An `in` argument, as in `#[unspec(in)]`.
    In(Token![in]),
    /// An `out` argument, as in `#[unspec(out)]`.
    Out(kw::out),
}

impl ToTokens for UnspecArg {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            UnspecArg::In(in_token) => in_token.to_tokens(tokens),
            UnspecArg::Out(out_token) => out_token.to_tokens(tokens),
        }
    }
}

impl Parse for UnspecArg {
    fn parse(input: ParseStream) -> Result<Self> {
        let arg = if input.peek(Token![in]) {
            Self::In(input.parse::<Token![in]>()?)
        } else if input.peek(kw::out) {
            Self::Out(input.parse::<kw::out>()?)
        } else {
            return Err(input.error("expected `in` or `out`"));
        };

        if !input.is_empty() {
            return Err(input.error("expected exactly one argument"));
        }

        Ok(arg)
    }
}

mod kw {
    syn::custom_keyword!(out);
}
