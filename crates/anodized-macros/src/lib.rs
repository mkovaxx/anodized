#![doc = include_str!("../README.md")]

use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::ToTokens;
use syn::{Expr, Item, TraitItemFn, parse_macro_input};

use anodized_core::{
    annotate::{Specified as _, syntax::SpecFields},
    instrument::{CheckSettings, Mode, PanicSettings, fns::make_try_call, make_item_error},
};

const CONFIG: Mode = if cfg!(anodized_discard_specs) {
    Mode::ChangeNothing
} else {
    Mode::InjectChecks(CheckSettings {
        does_print: cfg!(anodized_print),
        does_panic: if cfg!(anodized_panic) {
            Some(PanicSettings {
                has_try_fn: cfg!(anodized_try),
            })
        } else {
            None
        },
    })
};

/// Attaches a specification to supported program elements.
///
/// This macro parses the spec and transforms the item's code to provide the following features:
/// - compile-time validation: the spec's syntax, scope, and types
/// - runtime checks: for supported items, configured by `cfg` settings
#[proc_macro_attribute]
pub fn spec(args: TokenStream, input: TokenStream) -> TokenStream {
    // Parse the `#[spec]` attribute's arguments.
    let spec_fields = parse_macro_input!(args as SpecFields);

    // Parse the item to which the attribute is attached.
    let item = parse_macro_input!(input as Item);

    let result = match item {
        Item::Fn(mut func) => func
            .with_spec_from_fields(spec_fields)
            .and_then(|spec| CONFIG.instrument_item_fn(spec, func)),
        Item::Trait(mut the_trait) => the_trait
            .with_spec_from_fields(spec_fields)
            .and_then(|spec| CONFIG.instrument_item_trait(spec, the_trait)),
        Item::Impl(mut the_impl) if the_impl.trait_.is_some() => the_impl
            .with_spec_from_fields(spec_fields)
            .and_then(|spec| CONFIG.instrument_item_trait_impl(spec, the_impl)),
        Item::Impl(mut the_impl) if the_impl.trait_.is_none() => the_impl
            .with_spec_from_fields(spec_fields)
            .and_then(|spec| CONFIG.instrument_item_impl(spec, the_impl)),
        Item::Const(_) => Err(make_item_error(&item, "const")),
        Item::Enum(mut the_enum) => the_enum
            .with_spec_from_fields(spec_fields)
            .and_then(|spec| CONFIG.instrument_item_enum(spec, the_enum)),
        Item::ExternCrate(_) => Err(make_item_error(&item, "extern crate")),
        Item::ForeignMod(_) => Err(make_item_error(&item, "extern block")),
        Item::Macro(_) => Err(make_item_error(&item, "macro")),
        Item::Mod(_) => Err(make_item_error(&item, "mod")),
        Item::Static(_) => Err(make_item_error(&item, "static")),
        Item::Struct(mut the_struct) => the_struct
            .with_spec_from_fields(spec_fields)
            .and_then(|spec| CONFIG.instrument_item_struct(spec, the_struct)),
        Item::TraitAlias(_) => Err(make_item_error(&item, "trait alias")),
        Item::Type(_) => Err(make_item_error(&item, "type")),
        Item::Union(_) => Err(make_item_error(&item, "union")),
        Item::Use(_) => Err(make_item_error(&item, "use")),
        Item::Verbatim(ref tokens) => {
            // Try to parse as a trait fn
            if let Ok(trait_fn) = syn::parse2::<TraitItemFn>(tokens.clone()) {
                Err(syn::Error::new_spanned(
                    &trait_fn,
                    r#"The enclosing trait must have a `#[spec]` annotation."#,
                ))
            } else {
                Err(make_item_error(&item, "<unexpected>"))
            }
        }
        _ => Err(make_item_error(&item, "<unknown>")),
    };

    match result {
        Ok(item) => item.into(),
        Err(e) => e.to_compile_error().into(),
    }
}

#[proc_macro]
pub fn try_call(args: TokenStream) -> TokenStream {
    if !CONFIG.emits_try_fn() {
        return syn::Error::new(
            Span::call_site(),
            "`try_call` needs the `anodized_try` build `cfg` to be enabled",
        )
        .to_compile_error()
        .into();
    }

    let expr = parse_macro_input!(args as Expr);

    match make_try_call(expr) {
        Ok(call) => call.into_token_stream().into(),
        Err(error) => error.to_compile_error().into(),
    }
}
