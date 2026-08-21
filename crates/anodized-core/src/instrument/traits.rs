#[cfg(test)]
#[path = "traits_tests.rs"]
mod traits_tests;

use syn::{
    Attribute, Block, FnArg, ImplItem, ImplItemFn, Pat, PatConst, PatIdent, PatLit, PatParen,
    PatPath, PatRange, PatSlice, PatStruct, PatTuple, PatTupleStruct, ReturnType, TraitItem,
    TraitItemFn, Visibility, parse_quote,
};

use crate::{
    DataSpec, Spec,
    instrument::{Mode, find_spec_attr, make_item_error},
};

impl Mode {
    /// Expand trait items by mangling each method and adding a wrapper default impl.
    ///
    /// Mangling a function involves the following:
    /// 1. Rename the function following the pattern: `fn add` -> `fn __anodized_add`.
    /// 2. Make a new function with the original name that has a default impl; the
    ///    default impl performs runtime validation and calls the mangled function.
    pub fn instrument_trait(
        &self,
        spec: DataSpec,
        mut the_trait: syn::ItemTrait,
    ) -> syn::Result<syn::ItemTrait> {
        // Currently we don't support any spec fields for traits themselves.
        if !spec.is_empty() {
            return Err(spec.spec_err(
                "Unsupported spec element on trait. Try placing it on an item inside the trait",
            ));
        }
        let _ = move || spec;

        let mut new_trait_items = Vec::with_capacity(the_trait.items.len() * 5);

        for item in the_trait.items.into_iter() {
            match item {
                TraitItem::Fn(mut func) => {
                    let (spec_attr, other_attrs) = find_spec_attr(func.attrs)?;
                    func.attrs = other_attrs;
                    // NOTE: We have no way of knowing which attributes are
                    //   "external" - meant for the interface and belong on the wrapper,
                    //   "internal" - meant for the mangled implementation.
                    //   Right now we put all attribs on both functions, but that's certainly
                    //   not going to work in every situation.

                    let fn_spec: Spec = match spec_attr {
                        Some(spec_attr) => spec_attr.parse_args()?,
                        None => Spec::empty(),
                    };

                    let attrs: [Attribute; 2] = [
                        parse_quote!(#[doc(hidden)]),
                        parse_quote!(#[allow(warnings)]),
                    ];

                    if let Self::EmbedSpecs = self {
                        // Embed `spec` elements as `__anodized_fn_*` items.
                        let spec_requires_fn = TraitItemFn {
                            attrs: attrs.to_vec(),
                            sig: Self::build_precondition_fn_sig(
                                "__anodized_fn_requires",
                                &func.sig,
                            ),
                            default: Some(Self::build_precondition_fn_body(
                                &fn_spec.requires,
                                &fn_spec.maintains,
                            )),
                            semi_token: None,
                        };
                        let spec_ensures_fn = TraitItemFn {
                            attrs: attrs.to_vec(),
                            sig: Self::build_postcondition_fn_sig(
                                "__anodized_fn_ensures",
                                &func.sig,
                            ),
                            default: Some(Self::build_postcondition_fn_body(
                                &fn_spec.maintains,
                                &fn_spec.captures,
                                &fn_spec.ensures,
                            )?),
                            semi_token: None,
                        };

                        new_trait_items.push(TraitItem::Fn(spec_requires_fn));
                        new_trait_items.push(TraitItem::Fn(spec_ensures_fn));
                    }

                    if self.changes_anything() {
                        let spec_trait_qualifiers_const = Self::build_qualifier_const_item(
                            &attrs,
                            "__anodized_fn_qualifiers_trait",
                            fn_spec.qualifiers,
                            &func.sig.ident,
                        );
                        let spec_qualifiers_const = Self::build_qualifier_const_item(
                            &attrs,
                            "__anodized_fn_qualifiers",
                            fn_spec.qualifiers,
                            &func.sig.ident,
                        );
                        new_trait_items.push(TraitItem::Const(spec_trait_qualifiers_const));
                        new_trait_items.push(TraitItem::Const(spec_qualifiers_const));
                    }

                    if let Some(default_body) = &mut func.default {
                        // Handle loop specs in the body of the default impl.
                        self.instrument_loops_in_fn_body(default_body)?;
                    }

                    if let Mode::InjectChecks(_) = self {
                        let mangled_ident = mangle_ident(&func.sig.ident);

                        let mut mangled_fn = func.clone();
                        mangled_fn.sig.ident = mangled_ident.clone();
                        mangled_fn.attrs.retain(|attr| !attr.path().is_ident("doc"));
                        mangled_fn.attrs.push(parse_quote!(#[doc(hidden)]));
                        new_trait_items.push(TraitItem::Fn(mangled_fn));

                        let call_args = build_call_args(&func.sig.inputs)?;
                        let mut forwarding_body: Block = parse_quote!({
                            Self::#mangled_ident(#(#call_args),*)
                        });

                        self.instrument_fn(&fn_spec, &func.sig, &mut forwarding_body)?;

                        func.default = Some(forwarding_body);
                        func.semi_token = None;
                    }

                    if let Self::InjectChecks(check_settings) = self
                        && let Some(ref panic_settings) = check_settings.does_panic
                        && panic_settings.has_try_fn
                    {
                        // Build a wrapper that forwards to the "try_fn" entry point.
                        let mut wrapper_func = func.clone();
                        let mut wrapper_body: Block = parse_quote!({});
                        let mangled_ident = Self::build_try_fn_wrapper(
                            true,
                            &mut wrapper_func.sig,
                            &mut wrapper_body,
                        );
                        wrapper_func.default = Some(wrapper_body);
                        new_trait_items.push(TraitItem::Fn(wrapper_func));

                        // Create the "try_fn" entry point for e.g. fuzzing and PBT.
                        func.sig.ident = mangled_ident;
                        func.sig.output = match func.sig.output {
                            ReturnType::Default => {
                                parse_quote!(-> ::anodized::result::Result<()>)
                            }
                            ReturnType::Type(ra, ty) => {
                                parse_quote!(#ra ::anodized::result::Result<#ty>)
                            }
                        };
                        func.attrs = vec![parse_quote!(#[doc(hidden)]), parse_quote!(#[inline])];
                    }

                    new_trait_items.push(TraitItem::Fn(func));
                }
                TraitItem::Const(mut const_item) => {
                    let (spec, attrs) = find_spec_attr(const_item.attrs)?;
                    if let Some(ref spec_attr) = spec {
                        return Err(make_item_error(&spec_attr, "trait const"));
                    }
                    const_item.attrs = attrs;
                    new_trait_items.push(TraitItem::Const(const_item));
                }
                TraitItem::Type(mut type_item) => {
                    let (spec, attrs) = find_spec_attr(type_item.attrs)?;
                    if let Some(ref spec_attr) = spec {
                        return Err(make_item_error(&spec_attr, "trait type"));
                    }
                    type_item.attrs = attrs;
                    new_trait_items.push(TraitItem::Type(type_item));
                }
                TraitItem::Macro(mut macro_item) => {
                    let (spec, attrs) = find_spec_attr(macro_item.attrs)?;
                    if let Some(ref spec_attr) = spec {
                        return Err(make_item_error(&spec_attr, "trait macro"));
                    }
                    macro_item.attrs = attrs;
                    new_trait_items.push(TraitItem::Macro(macro_item));
                }
                TraitItem::Verbatim(token_stream) => {
                    new_trait_items.push(TraitItem::Verbatim(token_stream));
                }
                _ => unimplemented!(),
            }
        }
        the_trait.items = new_trait_items;
        Ok(the_trait)
    }

    /// Expand impl items by mangling methods for trait impls.
    ///
    /// The `#[spec]` attribute on an impl `fn` must narrow the `#[spec]` of the trait `fn`:
    /// - The impl's preconditions must follow from the trait's preconditions.
    /// - The impl's postconditions must entail the trait's postconditions.
    pub fn instrument_trait_impl(
        &self,
        spec: DataSpec,
        mut the_impl: syn::ItemImpl,
    ) -> syn::Result<syn::ItemImpl> {
        let Some((trait_bang, ref trait_path, _trait_for)) = the_impl.trait_ else {
            return Err(make_item_error(&the_impl, "inherent impl"));
        };

        if trait_bang.is_some() {
            return Err(make_item_error(&the_impl, "negative trait impl"));
        }

        if !spec.is_empty() {
            return Err(spec.spec_err("Unsupported spec element on trait impl."));
        }

        let mut new_items = Vec::with_capacity(the_impl.items.len() * 4);

        for item in the_impl.items.into_iter() {
            match item {
                ImplItem::Fn(mut func) => {
                    let (spec_attr, func_attrs) = find_spec_attr(func.attrs)?;
                    func.attrs = func_attrs;

                    if func.sig.ident.to_string().starts_with("__anodized_") {
                        return Err(syn::Error::new_spanned(
                            func.sig.ident,
                            r#"An item with the `__anodized_` prefix is internal. Do not implement it directly.
Instead, ensure that both the trait and the impl fn have a `#[spec]` annotation."#,
                        ));
                    }

                    let fn_spec: Spec = match spec_attr {
                        Some(spec_attr) => spec_attr.parse_args()?,
                        None => Spec::empty(),
                    };

                    let attrs: [Attribute; 2] = [
                        parse_quote!(#[doc(hidden)]),
                        parse_quote!(#[allow(warnings)]),
                    ];

                    if let Self::EmbedSpecs = self {
                        // Embed `spec` elements as `__anodized_fn_*` items.
                        let spec_requires_fn = ImplItemFn {
                            attrs: attrs.to_vec(),
                            sig: Self::build_precondition_fn_sig(
                                "__anodized_fn_requires",
                                &func.sig,
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
                                &func.sig,
                            ),
                            block: Self::build_postcondition_fn_body(
                                &fn_spec.maintains,
                                &fn_spec.captures,
                                &fn_spec.ensures,
                            )?,
                            vis: Visibility::Inherited,
                            defaultness: None,
                        };

                        new_items.push(ImplItem::Fn(spec_requires_fn));
                        new_items.push(ImplItem::Fn(spec_ensures_fn));
                    }

                    if self.changes_anything() {
                        let spec_qualifiers_const = Self::build_qualifier_const_item(
                            &attrs,
                            "__anodized_fn_qualifiers",
                            fn_spec.qualifiers,
                            &func.sig.ident,
                        );
                        new_items.push(ImplItem::Const(spec_qualifiers_const));
                    }

                    if let Mode::InjectChecks(_) = self {
                        self.with_try_fn(false).instrument_fn(
                            &fn_spec,
                            &func.sig,
                            &mut func.block,
                        )?;

                        // Add a compile-time check to the body.
                        func.block.stmts.insert(
                            0,
                            Self::build_qualifier_check_stmt(
                                &func.sig.ident,
                                &the_impl.self_ty,
                                trait_path,
                            ),
                        );

                        func.sig.ident = mangle_ident(&func.sig.ident);

                        // Add a default `#[inline]` attribute unless one is already there.
                        // The caller can supress this with `#[inline(never)]`
                        if !has_inline_attr(&func.attrs) {
                            func.attrs.push(parse_quote!(#[inline]));
                        }
                    }

                    new_items.push(ImplItem::Fn(func));
                }
                ImplItem::Const(mut const_item) => {
                    let (spec, attrs) = find_spec_attr(const_item.attrs)?;
                    if let Some(ref spec_attr) = spec {
                        return Err(make_item_error(&spec_attr, "trait impl const"));
                    }
                    const_item.attrs = attrs;
                    new_items.push(ImplItem::Const(const_item));
                }
                ImplItem::Type(mut type_item) => {
                    let (spec, attrs) = find_spec_attr(type_item.attrs)?;
                    if let Some(ref spec_attr) = spec {
                        return Err(make_item_error(&spec_attr, "trait impl type"));
                    }
                    type_item.attrs = attrs;
                    new_items.push(ImplItem::Type(type_item));
                }
                ImplItem::Macro(mut macro_item) => {
                    let (spec, attrs) = find_spec_attr(macro_item.attrs)?;
                    if let Some(ref spec_attr) = spec {
                        return Err(make_item_error(&spec_attr, "trait impl macro"));
                    }
                    macro_item.attrs = attrs;
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

/// Build argument tokens for calling the mangled trait method from the wrapper.
///
/// Purpose: the wrapper method needs to forward its arguments to the mangled
/// implementation, so this constructs a usable expression for each input.
///
/// Examples (inputs -> argument expressions):
/// - `fn f(&self, x: i32)` -> `self`, `x`
/// - `fn f(self, a: u8, b: u8)` -> `self`, `a`, `b`
/// - `fn f(input @ (left, right): (i32, i32))` -> `input`
/// - `fn f(Bounds { lower, upper }: Bounds)` -> `Bounds { lower, upper }`
///
/// The caller is responsible for ensuring these expressions are used in a call
/// like `Self::__anodized_f(#(#args),*)`.
///
/// Callers: only `instrument_trait` in this module should use this; it is not
/// part of the public API.
fn build_call_args(
    inputs: &syn::punctuated::Punctuated<FnArg, syn::Token![,]>,
) -> syn::Result<Vec<syn::Expr>> {
    let mut args = vec![];
    for input in inputs {
        let expr = match input {
            FnArg::Receiver(_) => parse_quote!(self),
            FnArg::Typed(pat_type) => {
                let pat = sanitize_pat_as_expr(&pat_type.pat)?;
                parse_quote!(#pat)
            }
        };
        args.push(expr);
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

/// Sanitize a pattern to be valid as an expression that reconstructs the matched value.
fn sanitize_pat_as_expr(pat: &Pat) -> syn::Result<Pat> {
    match pat {
        Pat::Const(pat_const) => Ok(Pat::Const(PatConst {
            attrs: vec![],
            const_token: pat_const.const_token,
            block: pat_const.block.clone(),
        })),
        Pat::Ident(pat_ident) if pat_ident.by_ref.is_none() => {
            let None = pat_ident.by_ref else {
                return Err(syn::Error::new_spanned(
                    pat_ident.by_ref,
                    "not allowed here due to `#[spec]`",
                ));
            };
            Ok(Pat::Ident(PatIdent {
                attrs: vec![],
                by_ref: None,
                mutability: None,
                ident: pat_ident.ident.clone(),
                subpat: None,
            }))
        }
        Pat::Lit(pat_lit) => Ok(Pat::Lit(PatLit {
            attrs: vec![],
            lit: pat_lit.lit.clone(),
        })),
        Pat::Path(pat_path) => Ok(Pat::Path(PatPath {
            attrs: vec![],
            qself: pat_path.qself.clone(),
            path: pat_path.path.clone(),
        })),
        Pat::Range(pat_range) => Ok(Pat::Range(PatRange {
            attrs: vec![],
            start: pat_range.start.clone(),
            limits: pat_range.limits,
            end: pat_range.end.clone(),
        })),
        Pat::Paren(pat_paren) => Ok(Pat::Paren(PatParen {
            attrs: vec![],
            paren_token: pat_paren.paren_token,
            pat: sanitize_pat_as_expr(&pat_paren.pat)?.into(),
        })),
        Pat::Reference(pat_reference) => sanitize_pat_as_expr(&pat_reference.pat),
        Pat::Type(pat_type) => sanitize_pat_as_expr(&pat_type.pat),
        Pat::Struct(pat_struct) => {
            let None = pat_struct.rest else {
                return Err(syn::Error::new_spanned(
                    &pat_struct.rest,
                    "not allowed here due to `#[spec]`",
                ));
            };
            let mut fields = syn::punctuated::Punctuated::<syn::FieldPat, syn::token::Comma>::new();
            for field_pat in &pat_struct.fields {
                let field_value = syn::FieldPat {
                    attrs: vec![],
                    member: field_pat.member.clone(),
                    colon_token: field_pat.colon_token,
                    pat: sanitize_pat_as_expr(&field_pat.pat)?.into(),
                };
                fields.push(field_value);
            }
            Ok(Pat::Struct(PatStruct {
                attrs: vec![],
                qself: pat_struct.qself.clone(),
                path: pat_struct.path.clone(),
                brace_token: pat_struct.brace_token,
                fields,
                rest: None,
            }))
        }
        Pat::Tuple(pat_tuple) => {
            let mut elems = syn::punctuated::Punctuated::<syn::Pat, syn::token::Comma>::new();
            for elem_pat in &pat_tuple.elems {
                let elem_value = sanitize_pat_as_expr(elem_pat)?;
                elems.push(elem_value);
            }
            Ok(Pat::Tuple(PatTuple {
                attrs: vec![],
                paren_token: pat_tuple.paren_token,
                elems,
            }))
        }
        Pat::TupleStruct(pat_tuple_struct) => {
            let mut elems = syn::punctuated::Punctuated::<syn::Pat, syn::token::Comma>::new();
            for elem_pat in &pat_tuple_struct.elems {
                let elem_value = sanitize_pat_as_expr(elem_pat)?;
                elems.push(elem_value);
            }
            Ok(Pat::TupleStruct(PatTupleStruct {
                attrs: vec![],
                qself: pat_tuple_struct.qself.clone(),
                path: pat_tuple_struct.path.clone(),
                paren_token: pat_tuple_struct.paren_token,
                elems,
            }))
        }
        Pat::Slice(pat_slice) => {
            let mut elems = syn::punctuated::Punctuated::<syn::Pat, syn::token::Comma>::new();
            for elem_pat in &pat_slice.elems {
                let elem_value = sanitize_pat_as_expr(elem_pat)?;
                elems.push(elem_value);
            }
            Ok(Pat::Slice(PatSlice {
                attrs: vec![],
                bracket_token: pat_slice.bracket_token,
                elems,
            }))
        }
        Pat::Verbatim(token_stream) => Ok(Pat::Verbatim(token_stream.clone())),
        Pat::Macro(pat_macro) => Err(syn::Error::new_spanned(
            pat_macro,
            "not allowed here due to `#[spec]`",
        )),
        Pat::Or(pat_or) => Err(syn::Error::new_spanned(
            pat_or,
            "or-pattern not allowed here due to `#[spec]`",
        )),
        Pat::Rest(pat_rest) => Err(syn::Error::new_spanned(
            pat_rest,
            "not allowed here due to `#[spec]`",
        )),
        Pat::Wild(pat_wild) => Err(syn::Error::new_spanned(
            pat_wild,
            "not allowed here due to `#[spec]`",
        )),
        _ => Err(syn::Error::new_spanned(
            pat,
            "not allowed here due to `#[spec]`",
        )),
    }
}
