use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::{
    AttrStyle, Attribute, Error, FieldValue, Ident, MacroDelimiter, Member, Meta, MetaList, Path,
    Token,
    parse::{Parse, ParseStream, Result},
    punctuated::Punctuated,
    token::{Bracket, Paren, Pound},
};

/// Raw spec fields, i.e. as they appear in the `#[spec(...)]` proc macro invocation.
///
/// Represents a syntactically well-formed but otherwise unvalidated set of `spec` fields.
///
/// It reuses Rust's grammar of fields inside a `struct` expression. For reference, see:
///
/// <https://doc.rust-lang.org/reference/expressions/struct-expr.html#railroad-StructExprField>
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
    // TODO: Remove `binds` and `inspects` before v0.7.0 is released.
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

/// Represents a valid `#[unspec]` attribute.
#[derive(Debug, Clone)]
pub struct UnspecAttr {
    pub pound_token: Pound,
    pub bracket_token: Bracket,
    pub path: Path,
    pub paren_token: Option<Paren>,
    pub args: UnspecArgs,
}

impl From<UnspecAttr> for Attribute {
    fn from(unspec: UnspecAttr) -> Self {
        Attribute {
            pound_token: unspec.pound_token,
            style: AttrStyle::Outer,
            bracket_token: unspec.bracket_token,
            meta: if let Some(paren) = unspec.paren_token {
                Meta::List(MetaList {
                    path: unspec.path,
                    delimiter: MacroDelimiter::Paren(paren),
                    tokens: unspec.args.into_token_stream(),
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
                "must be an outer attribute (no `!`): `#[unspec]`",
            ));
        };
        match attr.meta {
            Meta::Path(path) if path_matches_name(&path, "unspec") => Ok(UnspecAttr {
                pound_token: attr.pound_token,
                bracket_token: attr.bracket_token,
                path,
                paren_token: None,
                args: UnspecArgs::None,
            }),
            Meta::List(list) if path_matches_name(&list.path, "unspec") => {
                let MacroDelimiter::Paren(paren) = list.delimiter else {
                    return Err(syn::Error::new(
                        list.delimiter.span().open(),
                        "expected arguments in parentheses",
                    ));
                };
                Ok(UnspecAttr {
                    pound_token: attr.pound_token,
                    bracket_token: attr.bracket_token,
                    path: list.path,
                    paren_token: Some(paren),
                    args: syn::parse2(list.tokens)?,
                })
            }
            Meta::NameValue(key_val) if path_matches_name(&key_val.path, "unspec") => Err(
                syn::Error::new_spanned(key_val.eq_token, "expected arguments in parentheses"),
            ),
            _ => Err(syn::Error::new_spanned(
                attr,
                "expected an `#[unspec]` attribute",
            )),
        }
    }
}

fn path_matches_name(path: &Path, name: &str) -> bool {
    path.get_ident()
        .is_some_and(|ident| ident.to_string() == name)
}

/// Represents valid arguments to the `#[unspec]` attribute.
#[derive(Debug, Clone, Copy)]
pub enum UnspecArgs {
    /// No arguments: `#[unspec]`.
    None,
    /// An `in` argument: `#[unspec(in)]`.
    In(Token![in]),
    /// An `out` argument: `#[unspec(out)]`.
    Out(kw::out),
}

impl ToTokens for UnspecArgs {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            UnspecArgs::None => {}
            UnspecArgs::In(in_token) => in_token.to_tokens(tokens),
            UnspecArgs::Out(out_token) => out_token.to_tokens(tokens),
        }
    }
}

impl Parse for UnspecArgs {
    fn parse(input: ParseStream) -> Result<Self> {
        let args = if input.peek(Token![in]) {
            Self::In(input.parse::<Token![in]>()?)
        } else if input.peek(kw::out) {
            Self::Out(input.parse::<kw::out>()?)
        } else {
            return Err(input.error("expected `in` or `out`"));
        };

        if !input.is_empty() {
            return Err(input.error("expected exactly one argument"));
        }

        Ok(args)
    }
}

/// Removes a single `#[unspec]` attribute if present, from an attribute list.
///
/// If there are multiple `#[unspec]` attributes, return `Err`.
/// The arguments of the `unspec` attribute are *not* validated.
pub fn remove_unspec_attr(attrs: &mut Vec<Attribute>) -> Result<Option<Attribute>> {
    let mut maybe_index = None;

    for (i, attr) in attrs
        .iter()
        .enumerate()
        .filter(|(_, attr)| path_matches_name(attr.path(), "unspec"))
    {
        if maybe_index.is_some() {
            return Err(Error::new_spanned(
                attr,
                "multiple `#[unspec]` attributes are not allowed",
            ));
        }
        maybe_index = Some(i);
    }

    if let Some(index) = maybe_index {
        let attr = attrs.remove(index);
        Ok(Some(attr))
    } else {
        Ok(None)
    }
}

mod kw {
    syn::custom_keyword!(out);
}
