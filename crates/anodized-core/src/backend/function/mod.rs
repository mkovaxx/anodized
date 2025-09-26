#[cfg(test)]
mod tests;

use crate::{Spec, backend::Backend};

use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::{ToTokens, quote};
use syn::{Block, Ident, ItemFn, Meta, parse::Result, parse_quote};

impl Backend {
    pub fn instrument_fn(self, spec: Spec, mut func: ItemFn) -> syn::Result<ItemFn> {
        let is_async = func.sig.asyncness.is_some();

        // Extract the return type from the function signature
        let return_type = match &func.sig.output {
            syn::ReturnType::Default => syn::parse_quote!(()),
            syn::ReturnType::Type(_, ty) => ty.as_ref().clone(),
        };

        // Generate the new, instrumented function body.
        let new_body = self.instrument_fn_body(&spec, &func.block, is_async, &return_type)?;

        // Replace the old function body with the new one.
        *func.block = new_body;

        Ok(func)
    }

    fn instrument_fn_body(
        self,
        spec: &Spec,
        original_body: &Block,
        is_async: bool,
        return_type: &syn::Type,
    ) -> Result<Block> {
        // The identifier for the return value binding.
        let output_ident = Ident::new("__anodized_output", Span::mixed_site());

        // --- Generate Precondition Checks ---
        let guard_assert = |assert_stmt: TokenStream2, cfg: Option<&Meta>| {
            if self.disable_runtime_checks {
                quote! { if false { #assert_stmt } }
            } else if let Some(cfg) = cfg {
                quote! { if cfg!(#cfg) { #assert_stmt } }
            } else {
                assert_stmt
            }
        };

        let preconditions = spec
            .requires
            .iter()
            .map(|condition| {
                let expr = &condition.expr;
                let expr_str = expr.to_token_stream().to_string();
                let assert = quote! { assert!(#expr, "Precondition failed: {}", #expr_str); };
                guard_assert(assert, condition.cfg.as_ref())
            })
            .chain(spec.maintains.iter().map(|condition| {
                let expr = &condition.expr;
                let expr_str = expr.to_token_stream().to_string();
                let assert = quote! { assert!(#expr, "Pre-invariant failed: {}", #expr_str); };
                guard_assert(assert, condition.cfg.as_ref())
            }));

        // --- Generate Combined Body and Capture Statement ---
        // Capture values and execute body in a single tuple assignment
        // This ensures captured values aren't accessible to the body itself

        // Chain capture aliases with output binding
        let aliases = spec
            .captures
            .iter()
            .map(|cb| &cb.alias)
            .chain(std::iter::once(&output_ident));

        // Chain capture expressions with body expression
        let capture_exprs = spec.captures.iter().map(|cb| {
            let expr = &cb.expr;
            quote! { #expr }
        });

        // Chain underscore types with return type for tuple type annotation
        let types = spec
            .captures
            .iter()
            .map(|_| quote! { _ })
            .chain(std::iter::once(quote! { #return_type }));

        let body_expr = if is_async {
            quote! { (async || #original_body)().await }
        } else {
            quote! { (|| #original_body)() }
        };

        let exprs = capture_exprs.chain(std::iter::once(body_expr));

        // Build tuple assignment with type annotation on the tuple
        let body_and_captures = quote! {
            let (#(#aliases),*): (#(#types),*) = (#(#exprs),*);
        };

        // --- Generate Postcondition Checks ---
        let postconditions = spec
            .maintains
            .iter()
            .map(|condition| {
                let expr = &condition.expr;
                let expr_str = expr.to_token_stream().to_string();
                let assert = quote! { assert!(#expr, "Post-invariant failed: {}", #expr_str); };
                guard_assert(assert, condition.cfg.as_ref())
            })
            .chain(spec.ensures.iter().map(|postcondition| {
                let closure = annotate_postcondition_closure_argument(
                    postcondition.closure.clone(),
                    return_type.clone(),
                );
                let closure_str = postcondition.closure.to_token_stream().to_string();

                let assert = quote! {
                    assert!((#closure)(&#output_ident), "Postcondition failed: {}", #closure_str);
                };
                guard_assert(assert, postcondition.cfg.as_ref())
            }));

        Ok(parse_quote! {
            {
                #(#preconditions)*
                #body_and_captures
                #(#postconditions)*
                #output_ident
            }
        })
    }
}

fn annotate_postcondition_closure_argument(
    mut closure: syn::ExprClosure,
    return_type: syn::Type,
) -> syn::ExprClosure {
    // Add type annotation: convert |param| to |param: &ReturnType|.
    if let Some(first_input) = closure.inputs.first_mut() {
        // Wrap the pattern with a type annotation
        let pattern = first_input.clone();
        *first_input = syn::Pat::Type(syn::PatType {
            attrs: vec![],
            pat: Box::new(pattern),
            colon_token: Default::default(),
            ty: Box::new(syn::Type::Reference(syn::TypeReference {
                and_token: Default::default(),
                lifetime: None,
                mutability: None,
                elem: Box::new(return_type),
            })),
        });
    }
    closure
}
