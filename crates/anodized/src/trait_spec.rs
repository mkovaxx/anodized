use proc_macro::TokenStream;
use quote::quote;
use syn::{Attribute, FnArg, ImplItem, ItemFn, Pat, TraitItem, parse_quote};
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
    let spec: Spec = syn::parse(args.clone())?;
    if !spec.is_empty() {
        return Err(syn::Error::new_spanned::<proc_macro2::TokenStream, &str>(
            args.into(),
            "unsupported spec element on trait.  Maybe it should go on an item within the trait",
        ));
    }

    let mut new_trait_items = Vec::with_capacity(the_trait.items.len() * 2);

    for item in the_trait.items.into_iter() {
        match item {
            TraitItem::Fn(mut func) => {
                let (spec, other_attrs) = parse_spec_attr(func.attrs)?;

                //ISSUE: We have no way of knowing which attributes are "externally facing", i.e. they are meant
                // for the interface and therefore belong on the wrapper with the un-mangled name, and which ones
                // are "internally facing", and are meant for the mangled implementation.  Right now we put all
                // attribs on both functions, but that's certainly not going to work in every situation
                func.attrs = other_attrs.clone();

                let original_ident = func.sig.ident.clone();
                let mangled_ident = mangle_ident(&original_ident);

                let mut mangled_fn = func.clone();
                mangled_fn.sig.ident = mangled_ident.clone();
                mangled_fn.attrs.retain(|attr| !attr.path().is_ident("doc"));
                mangled_fn.attrs.push(parse_quote!(#[doc(hidden)]));

                let call_args = build_call_args(&func.sig.inputs)?;
                let mut wrapper_block: syn::Block = parse_quote!({
                    Self::#mangled_ident(#(#call_args),*)
                });

                if let Some((spec, _spec_attr)) = spec {
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

/// Expand impl items by mangling methods for trait impls
///
/// `#[spec]` attributes on the impl items themselves may not have `requires`, `maintains`, nor `ensures` directives.
pub fn instrument_impl(
    args: TokenStream,
    mut the_impl: syn::ItemImpl,
) -> syn::Result<syn::ItemImpl> {
    let spec: Spec = syn::parse(args.clone())?;
    if !spec.is_empty() {
        return Err(syn::Error::new_spanned::<proc_macro2::TokenStream, &str>(
            args.into(),
            "unsupported spec element on impl block.  Maybe it should go on an item within the block",
        ));
    }

    if the_impl.trait_.is_none() {
        return Err(syn::Error::new_spanned(
            &the_impl.self_ty,
            "anodized only supports specs on trait impl blocks",
        ));
    }

    let mut new_items = Vec::with_capacity(the_impl.items.len());

    for item in the_impl.items.into_iter() {
        match item {
            ImplItem::Fn(mut func) => {

                let (spec, mut func_attrs) = parse_spec_attr(func.attrs)?;
                if let Some((_, spec_attr)) = spec {
                    return Err(syn::Error::new_spanned(
                        spec_attr,
                        "trait impl methods may not have spec attributes.  Implementations must respect the contract of the trait interface.  Please file an issue on github if you need implementation-specific validation",
                    ));

                    // QUESTION: Do we want to allow a spec, so long as it doesn't contain `requires`, `maintains`, nor `ensures`?
                    //
                    // if !spec.requires.is_empty() || !spec.maintains.is_empty() || !spec.ensures.is_empty() {
                    //     return Err(syn::Error::new_spanned(
                    //         spec_attr,
                    //         "trait impl method specs may not contain `requires`, `maintains`, nor `ensures`",
                    //     ));
                    // }
                    // func_attrs.push(spec_attr);
                }

                let original_ident = func.sig.ident.clone();
                if !original_ident.to_string().starts_with("__anodized_") {
                    func.sig.ident = mangle_ident(&original_ident);
                }

                //Add a default `#[inline]` attribute unless one is already there.
                //The caller can supress this with `#[inline(never)]`
                if !has_inline_attr(&func_attrs) {
                    func_attrs.push(parse_quote!(#[inline]));
                }

                func.attrs = func_attrs;
                new_items.push(ImplItem::Fn(func));
            }
            other => new_items.push(other),
        }
    }

    the_impl.items = new_items;
    Ok(the_impl)
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

/// Prefix an identifier with `__anodized_`, preserving the original span.
/// Used when generating mangled method names in trait and impl expansion.
fn mangle_ident(original_ident: &syn::Ident) -> syn::Ident {
    syn::Ident::new(
        &format!("__anodized_{original_ident}"),
        original_ident.span(),
    )
}

/// Checks to see if any `#[inline]` (with or without arg) exists in the function's attribs
fn has_inline_attr(attrs: &[syn::Attribute]) -> bool {
    attrs.iter().any(|attr| attr.path().is_ident("inline"))
}

/// Parses out the `[spec]` attrib from a function's attribute list
///
/// Returns the parsed spec, the spec [Attribute] and the remaining attributes
fn parse_spec_attr(
    attrs: Vec<Attribute>
) -> syn::Result<(Option<(Spec, Attribute)>, Vec<Attribute>)> {
    let mut spec = None;
    let mut other_attrs = Vec::new();

    for attr in attrs {
        if attr.path().is_ident("spec") {
            if spec.is_some() {
                return Err(syn::Error::new_spanned(
                    attr,
                    "multiple `#[spec]` attributes on a single method are not supported",
                ));
            }
            spec = Some((attr.parse_args::<Spec>()?, attr));
        } else {
            other_attrs.push(attr);
        }
    }

    Ok((spec, other_attrs))
}
