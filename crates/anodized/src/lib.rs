//! Pragmatic specification annotations for Rust.
//!
#![doc = include_str!("../README.md")]

use proc_macro::TokenStream;
use quote::ToTokens;
use syn::{Item, parse_macro_input};

use anodized_core::{
    Spec,
    backend::{Backend, function::instrument_fn},
};

const _: () = {
    let count: u32 = cfg!(feature = "backend-no-checks") as u32;
    if count > 1 {
        panic!("anodized: backend features are mutually exclusive");
    }
};

const BACKEND: Backend = if cfg!(feature = "backend-no-checks") {
    Backend::NO_CHECKS
} else {
    Backend::DEFAULT
};

/// The main procedural macro for defining specifications on functions.
///
/// This macro parses spec annotations and injects `assert!` statements
/// into the function body to perform runtime checks.
#[proc_macro_attribute]
pub fn spec(args: TokenStream, input: TokenStream) -> TokenStream {
    // Parse the item to which the attribute is attached.
    let item = parse_macro_input!(input as Item);

    let result = match item {
        Item::Fn(func) => {
            let spec = parse_macro_input!(args as Spec);
            instrument_fn(BACKEND, spec, func)
        }
        unsupported_item => {
            let item_type = item_to_string(&unsupported_item);
            let msg = format!(
                r#"The `#[spec]` attribute doesn't yet support this item: `{}`.
If this is a problem for your use case, please open a feature
request at https://github.com/mkovaxx/anodized/issues/new"#,
                item_type
            );
            Err(syn::Error::new_spanned(unsupported_item, msg))
        }
    };

    match result {
        Ok(item) => item.into_token_stream().into(),
        Err(e) => e.to_compile_error().into(),
    }
}

fn item_to_string(item: &Item) -> &str {
    match item {
        Item::Const(_) => "const",
        Item::Enum(_) => "enum",
        Item::ExternCrate(_) => "extern crate",
        Item::Fn(_) => unreachable!(),
        Item::ForeignMod(_) => "extern block",
        Item::Impl(_) => "impl",
        Item::Macro(_) => "macro",
        Item::Mod(_) => "mod",
        Item::Static(_) => "static",
        Item::Struct(_) => "struct",
        Item::Trait(_) => "trait",
        Item::TraitAlias(_) => "trait alias",
        Item::Type(_) => "type",
        Item::Union(_) => "union",
        Item::Use(_) => "use",
        _ => "<unknown>",
    }
}
