use quote::quote;
use syn::{FnArg, ImplItem, ItemFn, Pat, TraitItem, parse_quote};

use crate::{
    Spec,
    instrument::{Backend, find_spec_attr, make_item_error},
};

impl Backend {
    /// Expand trait items by mangling each method and adding a wrapper default impl.
    ///
    /// Mangling a function involves the following:
    /// 1. Rename the function following the pattern: `fn add` -> `fn __anodized_add`.
    /// 2. Make a new function with the original name that has a default impl; the
    ///    default impl performs runtime validation and calls the mangled function.
    pub fn instrument_trait(
        &self,
        spec: Spec,
        mut the_trait: syn::ItemTrait,
    ) -> syn::Result<syn::ItemTrait> {
        // Currently we don't support any spec arguments for traits themselves.
        if !spec.is_empty() {
            return Err(spec.spec_err(
                "Unsupported spec element on trait. Try placing it on an item inside the trait",
            ));
        }

        let mut new_trait_items = Vec::with_capacity(the_trait.items.len() * 2);

        for item in the_trait.items.into_iter() {
            match item {
                TraitItem::Fn(mut func) => {
                    let (spec_attr, other_attrs) = find_spec_attr(func.attrs)?;

                    // NOTE: We have no way of knowing which attributes are
                    //   "external" - meant for the interface and belong on the wrapper,
                    //   "internal" - meant for the mangled implementation.
                    //   Right now we put all attribs on both functions, but that's certainly
                    //   not going to work in every situation.
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

                    if let Some(spec_attr) = spec_attr {
                        let spec = spec_attr.parse_args()?;
                        let wrapper_item = ItemFn {
                            attrs: Vec::new(),
                            vis: syn::Visibility::Inherited,
                            sig: func.sig.clone(),
                            block: Box::new(wrapper_block),
                        };
                        let instrumented = self.instrument_fn(spec, wrapper_item)?;
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
    /// `#[spec]` attributes on the impl items themselves are not allowed.
    pub fn instrument_trait_impl(
        &self,
        spec: Spec,
        mut the_impl: syn::ItemImpl,
    ) -> syn::Result<syn::ItemImpl> {
        let Some((trait_bang, ref _trait_path, _trait_for)) = the_impl.trait_ else {
            return Err(make_item_error(&the_impl, "inherent impl"));
        };

        if trait_bang.is_some() {
            return Err(make_item_error(&the_impl, "negative trait impl"));
        }

        if !spec.is_empty() {
            return Err(spec.spec_err(
                "Unsupported spec element on trait impl. Try placing it on an item inside the impl",
            ));
        }

        let mut new_items = Vec::with_capacity(the_impl.items.len());

        for item in the_impl.items.into_iter() {
            match item {
                ImplItem::Fn(mut func) => {
                    let (spec, mut func_attrs) = find_spec_attr(func.attrs)?;
                    if let Some(spec_attr) = spec {
                        return Err(syn::Error::new_spanned(
                            spec_attr,
                            r#"The #[spec] attribute doesn't support items inside a trait impl.
If this is a problem for your use case, please open a feature
request at https://github.com/mkovaxx/anodized/issues/new"#,
                        ));
                    }

                    let original_ident = func.sig.ident;
                    func.sig.ident = mangle_ident(&original_ident);

                    // Add a default `#[inline]` attribute unless one is already there.
                    // The caller can supress this with `#[inline(never)]`
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

/// Checks to see if any `#[inline]` (with or without arg) exists in the function's attribs.
fn has_inline_attr(attrs: &[syn::Attribute]) -> bool {
    attrs.iter().any(|attr| attr.path().is_ident("inline"))
}
