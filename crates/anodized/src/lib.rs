#![doc = include_str!("../../../README.md")]

use proc_macro::TokenStream;
use quote::ToTokens;
use syn::{Item, ItemFn, parse_macro_input};

use anodized_core::{Contract, instrument_fn_body};

/// The main procedural macro for defining contracts on functions.
///
/// This macro parses contract annotations and injects `assert!` statements
/// into the function body to perform runtime checks in debug builds.
#[proc_macro_attribute]
pub fn contract(args: TokenStream, input: TokenStream) -> TokenStream {
    // Parse the item to which the attribute is attached.
    let item = parse_macro_input!(input as Item);

    match item {
        Item::Fn(func) => handle_fn(args, func),
        item => {
            let item_type = item_to_string(&item);
            let msg = format!(
                r#"The `#[contract]` attribute doesn't yet support this item: `{}`.
If this is a problem for your use case, please open a feature
request at https://github.com/mkovaxx/anodized/issues/new"#,
                item_type
            );
            syn::Error::new_spanned(item, msg)
                .to_compile_error()
                .into()
        }
    }
}

fn handle_fn(args: TokenStream, mut func: ItemFn) -> TokenStream {
    let contract = parse_macro_input!(args as Contract);
    let is_async = func.sig.asyncness.is_some();

    // Generate the new, instrumented function body.
    let new_body = match instrument_fn_body(&contract, &func.block, is_async) {
        Ok(body) => body,
        Err(e) => return e.to_compile_error().into(),
    };

    // Replace the old function body with the new one.
    *func.block = new_body;

    // Return the modified function.
    func.into_token_stream().into()
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
