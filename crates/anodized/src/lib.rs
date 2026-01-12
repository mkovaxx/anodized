#![doc = include_str!("../README.md")]

use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::{ToTokens, quote};
use syn::{FnArg, Item, ItemFn, Pat, TraitItem, parse_macro_input, parse_quote};

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
            BACKEND.instrument_fn(spec, func).map(|tokens| tokens.into_token_stream())
        },
        Item::Trait(the_trait) => {
            // Mangling a function involves the following:
            //
            // 1. Rename the function following the pattern: `fn add` would be mangled to `fn __anodized_add`
            // 2. Make a new function with a with the original name that has a default impl; the default impl will
            //  perform all runtime validation, and call through to the mangled function.
            //
            // Every function in a trait gets mangled, regardless of whether or not it has a [spec] decorator or not.
            //  This is because the impl has no way of knowing if there is a spec or not on the trait item.


            //Currently we don't support any spec arguments for traits themselves - only for the
            // items within the trait
            let _spec = parse_macro_input!(args as Spec);

            let mut replacement_trait = the_trait.clone();
            let mut new_trait_items = Vec::with_capacity(the_trait.items.len() * 2);

            //Deal with spec macro markup on items within the trait
            for item in replacement_trait.items.into_iter() {
                match item {
                    TraitItem::Fn(mut func) => {
                        let mut spec = None;
                        let mut other_attrs = Vec::new();
                        for attr in core::mem::take(&mut func.attrs) {
                            if attr.path().is_ident("spec") {
                                match attr.parse_args::<Spec>() {
                                    Ok(parsed) => {
                                        spec = Some(parsed)
                                    },
                                    Err(e) => return e.to_compile_error().into()
                                }
                            } else {
                                other_attrs.push(attr);
                            }
                        }
                        func.attrs = other_attrs.clone();

                        let original_ident = func.sig.ident.clone();
                        let mangled_ident = syn::Ident::new(
                            &format!("__anodized_{original_ident}"),
                            Span::mixed_site(),
                        );

                        let mut mangled_fn = func.clone();
                        mangled_fn.sig.ident = mangled_ident.clone();

                        let call_args = match build_call_args(&func.sig.inputs) {
                            Ok(call_args) => call_args,
                            Err(e) => return e.to_compile_error().into()
                        };
                        let mut wrapper_block: syn::Block = parse_quote!({
                            Self::#mangled_ident(#(#call_args),*)
                        });

                        if let Some(spec) = spec {
                            let wrapper_item = ItemFn {
                                attrs: Vec::new(),
                                vis: syn::Visibility::Inherited,
                                sig: func.sig.clone(),
                                block: Box::new(wrapper_block),
                            };
                            match BACKEND.instrument_fn(spec, wrapper_item) {
                                Ok(instrumented) => {wrapper_block = *instrumented.block;},
                                Err(e) => return e.to_compile_error().into()
                            }
                        }

                        let mut wrapper_fn = func;
                        wrapper_fn.attrs = other_attrs;
                        wrapper_fn.default = Some(wrapper_block);
                        wrapper_fn.semi_token = None;

                        new_trait_items.push(TraitItem::Fn(mangled_fn));
                        new_trait_items.push(TraitItem::Fn(wrapper_fn));
                    },
                    other => new_trait_items.push(other),
                }
            }
            replacement_trait.items = new_trait_items;
            Ok(replacement_trait).map(|tokens| tokens.into_token_stream())
        },
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
        Ok(item) => item.into(),
        Err(e) => e.to_compile_error().into(),
    }
}

/// Build argument tokens for calling the mangled trait method from the wrapper.
///
/// Purpose: the wrapper method needs to forward its arguments to the mangled
/// implementation, so this extracts a usable token for each input.
///
/// Examples (inputs -> output tokens):
/// - `fn f(&self, x: i32)` -> `self, x`
/// - `fn f(self, a: u8, b: u8)` -> `self, a, b`
fn build_call_args(
    inputs: &syn::punctuated::Punctuated<FnArg, syn::Token![,]>,
) -> syn::Result<Vec<proc_macro2::TokenStream>> {
    let mut args = Vec::new();
    for input in inputs.iter() {
        match input {
            FnArg::Receiver(_) => {
                args.push(quote! { self });
            }
            FnArg::Typed(pat) => match pat.pat.as_ref() {
                Pat::Ident(pat_ident) => {
                    let ident = &pat_ident.ident;
                    args.push(quote! { #ident });
                }
                _ => {
                    return Err(syn::Error::new_spanned(
                        &pat.pat,
                        "unsupported pattern in trait method arguments",
                    ));
                }
            },
        }
    }
    Ok(args)
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
