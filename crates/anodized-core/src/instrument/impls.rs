#[cfg(test)]
#[path = "impls_tests.rs"]
mod impls_tests;

use proc_macro2::TokenStream;
use syn::{
    Attribute, Error, ImplItem, ImplItemFn, ItemImpl, Result, ReturnType, Visibility, parse_quote,
};

use crate::{
    DataSpec, SpecImplItemFn,
    annotate::{ParseOnItem as _, get_attr_input, remove_spec_attr},
    instrument::{Mode, make_item_error},
};

impl Mode {
    /// Expand items inside an inherent impl.
    ///
    /// Reasons why impl functions must be treated differently from free-standing functions:
    /// - The `__anodized_fn_try_*` function must be qualified as `Self::` inside an impl.
    pub fn instrument_impl(&self, spec: DataSpec, mut the_impl: ItemImpl) -> Result<ItemImpl> {
        if the_impl.trait_.is_some() {
            return Err(make_item_error(&the_impl, "trait impl"));
        };

        if !spec.is_empty() {
            return Err(spec.spec_err("Unsupported spec element on inherent impl."));
        }

        let mut new_items = Vec::with_capacity(the_impl.items.len() * 4);

        for item in the_impl.items.into_iter() {
            match item {
                ImplItem::Fn(mut item_fn) => {
                    let spec_attr = remove_spec_attr(&mut item_fn.attrs)?;

                    if item_fn.sig.ident.to_string().starts_with("__anodized_") {
                        return Err(Error::new_spanned(
                            item_fn.sig.ident,
                            r#"An item with the `__anodized_` prefix is internal. Do not implement it directly.
Instead, ensure that both the impl block and the fn have a `#[spec]` annotation."#,
                        ));
                    }

                    let spec_input = match spec_attr {
                        Some(spec_attr) => get_attr_input(spec_attr)?,
                        None => TokenStream::new(),
                    };
                    let SpecImplItemFn {
                        spec: fn_spec,
                        item: mut item_fn,
                    } = SpecImplItemFn::parse_spec_on(item_fn, spec_input)?;

                    if let Self::EmbedSpecs = self {
                        // Embed `spec` elements as `__anodized_fn_*` items.
                        let attrs: [Attribute; 2] = [
                            parse_quote!(#[doc(hidden)]),
                            parse_quote!(#[allow(warnings)]),
                        ];

                        let spec_qualifiers_const = Self::build_qualifier_const_item(
                            &attrs,
                            "__anodized_fn_qualifiers",
                            fn_spec.qualifiers,
                            &item_fn.sig.ident,
                        );
                        let spec_requires_fn = ImplItemFn {
                            attrs: attrs.to_vec(),
                            sig: Self::build_precondition_fn_sig(
                                "__anodized_fn_requires",
                                &item_fn.sig,
                            ),
                            block: Self::build_precondition_fn_body(
                                &fn_spec.requires,
                                &fn_spec.maintains,
                            ),
                            vis: Visibility::Inherited,
                            defaultness: None,
                        };
                        let spec_ensures_fn = ImplItemFn {
                            attrs: attrs.to_vec(),
                            sig: Self::build_postcondition_fn_sig(
                                "__anodized_fn_ensures",
                                &item_fn.sig,
                            ),
                            block: Self::build_postcondition_fn_body(
                                &fn_spec.maintains,
                                &fn_spec.captures,
                                &fn_spec.ensures,
                            )?,
                            vis: Visibility::Inherited,
                            defaultness: None,
                        };

                        new_items.push(ImplItem::Const(spec_qualifiers_const));
                        new_items.push(ImplItem::Fn(spec_requires_fn));
                        new_items.push(ImplItem::Fn(spec_ensures_fn));
                    }

                    // Instrument function body.
                    self.instrument_fn(&fn_spec, &item_fn.sig, &mut item_fn.block)?;

                    if let Self::InjectChecks(check_settings) = self
                        && let Some(ref panic_settings) = check_settings.does_panic
                        && panic_settings.has_try_fn
                    {
                        // Build a wrapper that forwards to the "try_fn" entry point.
                        let mut wrapper_fn = item_fn.clone();
                        let mangled_ident = Self::build_try_fn_wrapper(
                            true,
                            &mut wrapper_fn.sig,
                            &mut wrapper_fn.block,
                        );
                        new_items.push(ImplItem::Fn(wrapper_fn));

                        // Create the "try_fn" entry point for e.g. fuzzing and PBT.
                        item_fn.sig.ident = mangled_ident;
                        item_fn.sig.output = match item_fn.sig.output {
                            ReturnType::Default => {
                                parse_quote!(-> ::anodized::result::Result<()>)
                            }
                            ReturnType::Type(ra, ty) => {
                                parse_quote!(#ra ::anodized::result::Result<#ty>)
                            }
                        };
                        item_fn.attrs = vec![parse_quote!(#[doc(hidden)]), parse_quote!(#[inline])];
                    }

                    new_items.push(ImplItem::Fn(item_fn));
                }
                ImplItem::Const(mut const_item) => {
                    if let Some(spec_attr) = remove_spec_attr(&mut const_item.attrs)? {
                        return Err(make_item_error(&spec_attr, "impl const"));
                    }
                    new_items.push(ImplItem::Const(const_item));
                }
                ImplItem::Type(mut type_item) => {
                    if let Some(spec_attr) = remove_spec_attr(&mut type_item.attrs)? {
                        return Err(make_item_error(&spec_attr, "impl type"));
                    }
                    new_items.push(ImplItem::Type(type_item));
                }
                ImplItem::Macro(mut macro_item) => {
                    if let Some(spec_attr) = remove_spec_attr(&mut macro_item.attrs)? {
                        return Err(make_item_error(&spec_attr, "impl macro"));
                    }
                    new_items.push(ImplItem::Macro(macro_item));
                }
                ImplItem::Verbatim(token_stream) => {
                    new_items.push(ImplItem::Verbatim(token_stream))
                }
                _ => unimplemented!(),
            };
        }

        the_impl.items = new_items;
        Ok(the_impl)
    }
}
