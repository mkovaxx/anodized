#![doc = include_str!("../README.md")]

use proc_macro::TokenStream;
use quote::ToTokens;
use syn::{Item, parse_macro_input};

use anodized_core::{Spec, instrument::Backend};

const _: () = {
    let count: u32 = cfg!(feature = "runtime-check-and-panic") as u32
        + cfg!(feature = "runtime-check-and-print") as u32
        + cfg!(feature = "runtime-no-check") as u32;
    if count > 1 {
        panic!("anodized: runtime features are mutually exclusive");
    }
};

const BACKEND: Backend = if cfg!(feature = "runtime-check-and-panic") {
    Backend::CHECK_AND_PANIC
} else if cfg!(feature = "runtime-check-and-print") {
    Backend::CHECK_AND_PRINT
} else if cfg!(feature = "runtime-no-check") {
    Backend::NO_CHECK
} else {
    panic!(
        r#"anodized: a runtime feature must be selected:
`runtime-check-and-panic`
`runtime-check-and-print`
`runtime-no-check`"#
    )
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
            BACKEND
                .instrument_fn(spec, func)
                .map(|tokens| tokens.into_token_stream())
        }
        Item::Trait(the_trait) => {
            let spec = parse_macro_input!(args as Spec);
            BACKEND
                .instrument_trait(spec, the_trait)
                .map(|tokens| tokens.into_token_stream())
        }
        Item::Impl(the_impl) if the_impl.trait_.is_some() => {
            let spec = parse_macro_input!(args as Spec);
            BACKEND
                .instrument_trait_impl(spec, the_impl)
                .map(|tokens| tokens.into_token_stream())
        }
        Item::Impl(ref the_impl) if the_impl.trait_.is_none() => {
            Err(make_item_error(&item, "inherent impl"))
        }
        Item::Const(_) => Err(make_item_error(&item, "const")),
        Item::Enum(_) => Err(make_item_error(&item, "enum")),
        Item::ExternCrate(_) => Err(make_item_error(&item, "extern crate")),
        Item::ForeignMod(_) => Err(make_item_error(&item, "extern block")),
        Item::Macro(_) => Err(make_item_error(&item, "macro")),
        Item::Mod(_) => Err(make_item_error(&item, "mod")),
        Item::Static(_) => Err(make_item_error(&item, "static")),
        Item::Struct(_) => Err(make_item_error(&item, "struct")),
        Item::TraitAlias(_) => Err(make_item_error(&item, "trait alias")),
        Item::Type(_) => Err(make_item_error(&item, "type")),
        Item::Union(_) => Err(make_item_error(&item, "union")),
        Item::Use(_) => Err(make_item_error(&item, "use")),
        Item::Verbatim(_) => Err(make_item_error(&item, "<verbatim>")),
        _ => Err(make_item_error(&item, "<unknown>")),
    };

    match result {
        Ok(item) => item.into(),
        Err(e) => e.to_compile_error().into(),
    }
}

fn make_item_error(item: &syn::Item, item_type: &str) -> syn::Error {
    let msg = format!(
        r#"The `#[spec]` attribute doesn't yet support this item: `{}`.
If this is a problem for your use case, please open a feature
request at https://github.com/mkovaxx/anodized/issues/new"#,
        item_type
    );
    syn::Error::new_spanned(item, msg)
}
