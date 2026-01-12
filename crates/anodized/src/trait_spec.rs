use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::{FnArg, ItemFn, Pat, TraitItem, parse_quote};
use crate::BACKEND;

use anodized_core::Spec;

/// Expand trait items by mangling each method and adding a wrapper default impl.
///
/// Mangling a function involves the following:
/// 1. Rename the function following the pattern: `fn add` -> `fn __anodized_add`
/// 2. Make a new function with the original name that has a default impl; the
///    default impl performs runtime validation and calls the mangled function.
pub fn instrument_trait(
    args: TokenStream,
    mut the_trait: syn::ItemTrait
) -> syn::Result<syn::ItemTrait> {
    // Currently we don't support any spec arguments for traits themselves.
    let _spec: Spec = syn::parse(args)?;

    let mut new_trait_items = Vec::with_capacity(the_trait.items.len() * 2);

    for item in the_trait.items.into_iter() {
        match item {
            TraitItem::Fn(mut func) => {
                let mut spec = None;
                let mut other_attrs = Vec::new();
                for attr in core::mem::take(&mut func.attrs) {
                    if attr.path().is_ident("spec") {
                        match attr.parse_args::<Spec>() {
                            Ok(parsed) => {
                                spec = Some(parsed);
                            }
                            Err(e) => return Err(e),
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

                let call_args = build_call_args(&func.sig.inputs)?;
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
                    let instrumented = BACKEND.instrument_fn(spec, wrapper_item)?;
                    wrapper_block = *instrumented.block;
                }

                let mut wrapper_fn = func;
                wrapper_fn.attrs = other_attrs;
                wrapper_fn.default = Some(wrapper_block);
                wrapper_fn.semi_token = None;

                new_trait_items.push(TraitItem::Fn(mangled_fn));
                new_trait_items.push(TraitItem::Fn(wrapper_fn));
            }
            other => new_trait_items.push(other),
        }
    }
    the_trait.items = new_trait_items;
    Ok(the_trait)
}

/// Build argument tokens for calling the mangled trait method from the wrapper.
///
/// Purpose: the wrapper method needs to forward its arguments to the mangled
/// implementation, so this extracts a usable token for each input.
///
/// Examples (inputs -> output tokens):
/// - `fn f(&self, x: i32)` -> `self, x`
/// - `fn f(self, a: u8, b: u8)` -> `self, a, b`
///
/// The caller is responsible for ensuring these tokens are used in a call
/// expression like `Self::__anodized_f(#(#args),*)`.
///
/// Callers: only `instrument_trait` in this module should use this; it is not
/// part of the public API.
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
