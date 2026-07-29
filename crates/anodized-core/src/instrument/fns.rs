#[cfg(test)]
#[path = "fns_tests.rs"]
mod fns_tests;

use proc_macro2::Span;
use quote::{ToTokens, quote};
use syn::{
    Attribute, Block, Expr, Ident, Meta, Pat, Path, ReturnType, Signature, Stmt, Type,
    parse::{Parse, Result},
    parse_quote,
};

use crate::{
    Capture, Condition, Spec,
    instrument::{CheckSettings, Mode},
    qualifiers::FnQualifiers,
};

impl Mode {
    pub fn instrument_fn(&self, spec: &Spec, sig: &Signature, body: &mut Block) -> syn::Result<()> {
        self.instrument_loops_in_fn_body(body)?;

        let Mode::InjectChecks(check_config) = self else {
            return Ok(());
        };

        let is_async = sig.asyncness.is_some();

        // Generate the new, instrumented function body.
        let new_body = check_config.instrument_fn_body(spec, body, is_async, &sig.output)?;

        // Replace the old function body with the new one.
        *body = new_body;

        Ok(())
    }

    pub fn build_precondition_fn_sig(prefix: &str, sig: &Signature) -> Signature {
        Signature {
            constness: sig.constness,
            asyncness: sig.asyncness,
            unsafety: sig.unsafety,
            abi: sig.abi.clone(),
            fn_token: sig.fn_token,
            ident: syn::Ident::new(&format!("{prefix}_{}", sig.ident), sig.ident.span()),
            generics: sig.generics.clone(),
            paren_token: sig.paren_token,
            inputs: sig.inputs.clone(),
            variadic: sig.variadic.clone(),
            output: parse_quote!(-> bool),
        }
    }

    pub fn build_postcondition_fn_sig(prefix: &str, sig: &Signature) -> Signature {
        let mut inputs = sig.inputs.clone();
        let output_binder = match &sig.output {
            ReturnType::Type(_, return_type) => parse_quote! { __anodized_output: #return_type },
            ReturnType::Default => parse_quote! { __anodized_output: () },
        };
        inputs.push(output_binder);

        Signature {
            constness: sig.constness,
            asyncness: sig.asyncness,
            unsafety: sig.unsafety,
            abi: sig.abi.clone(),
            fn_token: sig.fn_token,
            ident: syn::Ident::new(&format!("{prefix}_{}", sig.ident), sig.ident.span()),
            generics: sig.generics.clone(),
            paren_token: sig.paren_token,
            inputs,
            variadic: sig.variadic.clone(),
            output: parse_quote!(-> bool),
        }
    }

    pub fn build_qualifier_const_item<SomeConstItem: Parse>(
        attrs: &[Attribute],
        prefix: &str,
        qualifiers: FnQualifiers,
        fn_ident: &Ident,
    ) -> SomeConstItem {
        let qualifier_bits = qualifiers.bits();
        let name: Ident = syn::Ident::new(&format!("{}_{}", prefix, fn_ident), fn_ident.span());
        parse_quote! {
            #(#attrs)*
            const #name: u32 = #qualifier_bits;
        }
    }

    pub fn build_qualifier_check_stmt(
        fn_ident: &Ident,
        impl_type: &Type,
        trait_path: &Path,
    ) -> Stmt {
        let impl_const_name = Ident::new(
            &format!("__anodized_fn_qualifiers_{}", fn_ident),
            fn_ident.span(),
        );

        let trait_const_name = Ident::new(
            &format!("__anodized_fn_qualifiers_trait_{}", fn_ident),
            fn_ident.span(),
        );

        let message = format!(
            "the qualifiers on the impl `{}::{fn_ident}` cannot be weaker than the qualifiers on the trait `{}::{fn_ident}`",
            impl_type.to_token_stream(),
            trait_path.to_token_stream(),
        );

        parse_quote! {
            const {
                assert!(
                    Self::#impl_const_name == Self::#trait_const_name | Self::#impl_const_name,
                    #message,
                );
            };
        }
    }

    pub fn build_precondition_fn_body(requires: &[Condition], maintains: &[Condition]) -> Block {
        let mut statements: Vec<Stmt> = vec![];
        let mut clauses: Vec<Expr> = vec![];

        for condition in requires.iter().chain(maintains) {
            let i = clauses.len();
            let name = Ident::new(&format!("__anodized_clause_{}", i + 1), Span::mixed_site());
            let expr = &condition.expr;
            statements.push(parse_quote! { let #name = (|| -> bool { #expr })(); });
            clauses.push(parse_quote! { #name });
        }

        if clauses.is_empty() {
            clauses.push(parse_quote!(true));
        }

        parse_quote! {
            {
                #(#statements)*
                #(#clauses)&&*
            }
        }
    }

    pub fn build_postcondition_fn_body(
        maintains: &[Condition],
        captures: &[Capture],
        binds: &Option<Pat>,
        ensures: &[Condition],
    ) -> Result<Block> {
        let mut statements: Vec<Stmt> = vec![];
        let mut clauses: Vec<Expr> = vec![];

        for condition in maintains {
            let i = clauses.len();
            let name = Ident::new(&format!("__anodized_clause_{}", i + 1), Span::mixed_site());
            let expr = &condition.expr;
            statements.push(parse_quote! { let #name = (|| -> bool { #expr })(); });
            clauses.push(parse_quote! { #name });
        }

        {
            let patterns = binds
                .iter()
                .chain(captures.iter().map(|capture| &capture.pat));

            let return_value: Option<Expr> =
                binds.as_ref().map(|_| parse_quote! { __anodized_output });
            let values = return_value
                .into_iter()
                .chain(captures.iter().map(|capture| -> Expr {
                    let expr = &capture.expr;
                    // Wrap in closure to guard against `return`.
                    parse_quote! { (|| #expr)() }
                }));
            statements.push(parse_quote! { let (#(#patterns),*) = (#(#values),*); });
        }

        for condition in ensures {
            let i = clauses.len();
            let name = Ident::new(&format!("__anodized_clause_{}", i + 1), Span::mixed_site());
            let expr = &condition.expr;
            statements.push(parse_quote! { let #name = (|| -> bool { #expr })(); });
            clauses.push(parse_quote! { #name });
        }

        if clauses.is_empty() {
            clauses.push(parse_quote!(true));
        }

        Ok(parse_quote! {
            {
                #(#statements)*
                #(#clauses)&&*
            }
        })
    }
}

impl CheckSettings {
    fn instrument_fn_body(
        &self,
        spec: &Spec,
        original_body: &Block,
        is_async: bool,
        return_type: &ReturnType,
    ) -> Result<Block> {
        // The identifier for the return value binding.
        let output_ident: Pat = parse_quote!(__anodized_output);

        // Generate precondition checks.
        let mut precondition_clauses: Vec<Expr> = vec![];
        for condition in spec.requires.iter().chain(&spec.maintains) {
            let expr = &condition.expr;
            let repr = expr.to_token_stream().to_string();
            let expr = parse_quote! { __anodized_eval_pre(|| -> bool { #expr }) };
            let clause = self.build_clause_eval(&condition.cfg, &expr, &repr);
            precondition_clauses.push(clause);
        }
        if precondition_clauses.is_empty() {
            precondition_clauses.push(parse_quote!(true));
        }

        // Bind capture values and function output in a single tuple assignment.
        // This ensures captured values are inaccessible to the body.
        let patterns = spec
            .captures
            .iter()
            .map(|cb| &cb.pat)
            .chain(std::iter::once(&output_ident));

        let body_expr = if is_async {
            quote! { (async || #return_type #original_body)().await }
        } else {
            quote! { (|| #return_type #original_body)() }
        };
        let values = spec
            .captures
            .iter()
            .map(|cb| {
                let expr = &cb.expr;
                // Evaluate expression in a closure to prevent early return.
                quote! { (|| #expr)() }
            })
            .chain(std::iter::once(body_expr));

        let captures_and_output = quote! {
            let (#(#patterns),*) = (#(#values),*);
        };

        // Generate postcondition checks.
        let mut postcondition_clauses: Vec<Expr> = vec![];
        for condition in spec.maintains.iter().chain(&spec.ensures) {
            let expr = &condition.expr;
            let repr = expr.to_token_stream().to_string();
            let expr = parse_quote! { __anodized_eval_post(|| -> bool { #expr }) };
            let clause = self.build_clause_eval(&condition.cfg, &expr, &repr);
            postcondition_clauses.push(clause);
        }
        if postcondition_clauses.is_empty() {
            postcondition_clauses.push(parse_quote!(true));
        }

        let do_run_checks = self.does_print || self.does_panic.is_some();

        let (output_expr, precond_fail_action, postcond_fail_action) =
            if let Some(ref panic_settings) = self.does_panic
                && panic_settings.has_try_fn
            {
                (
                    quote! { Ok(#output_ident) },
                    Some(parse_quote! {
                        return ::anodized::result::pre_err(__anodized_errors);
                    }),
                    Some(parse_quote! {
                        return ::anodized::result::post_err(#output_ident, __anodized_errors);
                    }),
                )
            } else {
                (
                    quote! { #output_ident },
                    self.build_fail_action("precondition failed"),
                    self.build_fail_action("postcondition failed"),
                )
            };

        let output_binder_stmt: Option<Stmt> = spec
            .inspects
            .as_ref()
            .map(|pat| parse_quote! { let #pat = #output_ident; });

        Ok(parse_quote! {
            {
                if #do_run_checks {
                    fn __anodized_eval_pre(c: impl Fn() -> bool) -> bool { c() }
                    let mut __anodized_errors = ::std::string::String::new();
                    let __anodized_precond = #(#precondition_clauses)&*;
                    if !__anodized_precond {
                        #precond_fail_action
                    }
                }
                #captures_and_output
                if #do_run_checks {
                    fn __anodized_eval_post(c: impl Fn() -> bool) -> bool { c() }
                    let mut __anodized_errors = ::std::string::String::new();
                    #output_binder_stmt;
                    let __anodized_postcond = #(#postcondition_clauses)&*;
                    if !__anodized_postcond {
                        #postcond_fail_action
                    }
                }
                #output_expr
            }
        })
    }

    fn build_clause_eval(&self, cfg: &Option<Meta>, expr: &Expr, repr: &str) -> Expr {
        if self.does_print {
            let br_and_repr = format!("\n    {repr}");
            let cfg_guard = match cfg {
                Some(meta) => quote! { !cfg!(#meta) || },
                None => quote!(),
            };
            parse_quote! { ( #cfg_guard #expr || __anodized_errors.push_str(#br_and_repr) != () ) }
        } else {
            expr.clone()
        }
    }

    fn build_fail_action(&self, message: &str) -> Option<Stmt> {
        let message_and_errors = format!("{message}:{{__anodized_errors}}");
        match (self.does_print, self.does_panic.is_some()) {
            (true, true) => Some(parse_quote! { panic!(#message_and_errors); }),
            (true, false) => Some(parse_quote! { eprintln!(#message_and_errors); }),
            (false, true) => Some(parse_quote! { panic!(#message); }),
            (false, false) => None,
        }
    }
}

pub(crate) fn make_try_fn_ident(ident: &Ident) -> Ident {
    Ident::new(&format!("__anodized_fn_try_{ident}"), ident.span())
}

pub fn make_try_call(mut expr: Expr) -> Result<Expr> {
    match &mut expr {
        Expr::Call(fn_call) => {
            if let Expr::Path(path) = fn_call.func.as_mut()
                && (path.qself.is_some() || path.path.segments.len() > 1)
            {
                let last_segment = path.path.segments.last_mut().expect("last segment");
                last_segment.ident = make_try_fn_ident(&last_segment.ident);
                return Ok(expr);
            }
        }
        Expr::MethodCall(method_call) => {
            method_call.method = make_try_fn_ident(&method_call.method);
            return Ok(expr);
        }
        _ => {}
    }

    Err(syn::Error::new_spanned(
        expr,
        "must be a method call or a qualified function call",
    ))
}
