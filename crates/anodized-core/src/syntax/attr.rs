use proc_macro2::TokenStream;
use syn::{Attribute, Error, Meta, Path, parse::Result};

/// Removes a single `#[spec]` attribute, if present, from an attribute list.
///
/// If there are multiple `#[spec]` attributes, returns `Err`.
/// The arguments of the `#[spec]` attribute are *not* validated.
pub fn remove_spec_attr(attrs: &mut Vec<Attribute>) -> Result<Option<Attribute>> {
    let mut maybe_index = None;

    for (i, attr) in attrs
        .iter()
        .enumerate()
        .filter(|(_, attr)| path_matches_name(attr.path(), "spec"))
    {
        if maybe_index.is_some() {
            return Err(Error::new_spanned(
                attr,
                "multiple `#[spec]` attributes are not allowed",
            ));
        }
        maybe_index = Some(i);
    }

    if let Some(index) = maybe_index {
        Ok(Some(attrs.remove(index)))
    } else {
        Ok(None)
    }
}

fn path_matches_name(path: &Path, name: &str) -> bool {
    path.get_ident().is_some_and(|ident| *ident == name)
}

/// Get the `TokenStream` that represents the arguments of an `Attribute`.
///
/// - If the attribute has no arguments, returns an empty stream.
/// - If the attribute has a `key = value` style argument, returns an error.
pub fn get_attr_input(attr: Attribute) -> Result<TokenStream> {
    match attr.meta {
        Meta::Path(_) => Ok(TokenStream::new()),
        Meta::List(list) => Ok(list.tokens),
        Meta::NameValue(key_value) => Err(Error::new_spanned(
            key_value.eq_token,
            "expected arguments between delimiters `()`, `[]`, or `{}`",
        )),
    }
}
