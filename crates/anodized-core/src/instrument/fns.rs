#[cfg(test)]
#[path = "fns_tests.rs"]
mod fns_tests;

use quote::{ToTokens, quote};
use syn::{
    Attribute, Block, Expr, Ident, Meta, Pat, Path, ReturnType, Signature, Stmt, Type,
    parse::{Parse, Result},
    parse_quote, parse_quote_spanned,
    spanned::Spanned,
};

use crate::{
    Capture, Condition, FnSpec, PostCondition,
    instrument::{CheckSettings, Mode, patterns::TamePat},
    qualifiers::FnQualifiers,
};

impl Mode {
    pub fn instrument_fn(&self, spec: &FnSpec, sig: &Signature, body: &mut Block) -> Result<()> {
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
        let mut stmts: Vec<Stmt> = vec![];
        Self::emit_precondition_checks(requires, maintains, &mut stmts, None);
        parse_quote! {
            {
                #(#stmts)*
                __anodized_pre
            }
        }
    }

    pub fn emit_precondition_checks(
        requires: &[Condition],
        maintains: &[Condition],
        statements: &mut Vec<Stmt>,
        maybe_instrument_eval: Option<fn(&str, &Option<Meta>, &Expr, &str) -> Expr>,
    ) {
        statements.push(parse_quote! {
            let __anodized_pre = true;
        });

        for precondition in requires {
            let eval = build_cond_eval(&precondition.expr);
            let instrumented_eval = if let Some(instrument_eval) = maybe_instrument_eval {
                let repr = precondition.expr.to_token_stream().to_string();
                instrument_eval("precondition failed: {}", &precondition.cfg, &eval, &repr)
            } else {
                eval
            };
            let check = build_precond_check(&instrumented_eval);
            statements.push(check);
        }

        for preinvariant in maintains {
            let eval = build_cond_eval(&preinvariant.expr);
            let instrumented_eval = if let Some(instrument_eval) = maybe_instrument_eval {
                let repr = preinvariant.expr.to_token_stream().to_string();
                instrument_eval("preinvariant failed: {}", &preinvariant.cfg, &eval, &repr)
            } else {
                eval
            };
            let check = build_precond_check(&instrumented_eval);
            statements.push(check);
        }
    }

    pub fn build_postcondition_fn_body(
        maintains: &[Condition],
        captures: &[Capture],
        ensures: &[PostCondition],
    ) -> Block {
        let mut stmts: Vec<Stmt> = vec![];
        Self::emit_postcondition_checks(maintains, captures, ensures, &mut stmts, None);
        parse_quote! {
            {
                #(#stmts)*
                __anodized_post
            }
        }
    }

    pub fn emit_postcondition_checks(
        maintains: &[Condition],
        captures: &[Capture],
        ensures: &[PostCondition],
        statements: &mut Vec<Stmt>,
        maybe_instrument_eval: Option<fn(&str, &Option<Meta>, &Expr, &str) -> Expr>,
    ) {
        statements.push(parse_quote! {
            let __anodized_post = true;
        });

        for postinvariant in maintains {
            let eval = build_cond_eval(&postinvariant.expr);
            let instrumented_eval = if let Some(instrument_eval) = &maybe_instrument_eval {
                let repr = postinvariant.expr.to_token_stream().to_string();
                instrument_eval("postinvariant failed: {}", &postinvariant.cfg, &eval, &repr)
            } else {
                eval
            };
            let check = build_postcond_check(&None, &instrumented_eval);
            statements.push(check);
        }

        {
            let patterns = captures.iter().map(|capture| &capture.pat);
            let values = captures
                .iter()
                .map(|capture| build_capture_eval(&capture.expr));
            statements.push(parse_quote! { let (#(#patterns),*) = (#(#values),*); });
        }

        for postcondition in ensures {
            let eval = build_cond_eval(&postcondition.expr);
            let instrumented_eval = if let Some(instrument_eval) = &maybe_instrument_eval {
                let repr = postcondition.expr.to_token_stream().to_string();
                instrument_eval("postcondition failed: {}", &postcondition.cfg, &eval, &repr)
            } else {
                eval
            };
            let check = build_postcond_check(&postcondition.pat, &instrumented_eval);
            statements.push(check);
        }
    }
}

impl CheckSettings {
    fn instrument_fn_body(
        &self,
        spec: &FnSpec,
        original_body: &Block,
        is_async: bool,
        return_type: &ReturnType,
    ) -> Result<Block> {
        // The identifier for the return value binding.
        let output_ident: Pat = parse_quote!(__anodized_output);

        // Generate precondition checks.
        let mut precond_checks: Vec<Stmt> = vec![parse_quote! {
            let __anodized_pre = true;
        }];
        for precondition in &spec.requires {
            let eval = build_cond_eval(&precondition.expr);
            let instrumented_eval = self.instrument_cond_eval(
                "precondition failed: {}",
                &precondition.cfg,
                &eval,
                &precondition.expr.to_token_stream().to_string(),
            );
            let check = build_precond_check(&instrumented_eval);
            precond_checks.push(check);
        }
        for preinvariant in &spec.maintains {
            let eval = build_cond_eval(&preinvariant.expr);
            let instrumented_eval = self.instrument_cond_eval(
                "preinvariant failed: {}",
                &preinvariant.cfg,
                &eval,
                &preinvariant.expr.to_token_stream().to_string(),
            );
            let check = build_precond_check(&instrumented_eval);
            precond_checks.push(check);
        }

        // Bind capture values and function output in a single tuple assignment.
        // This ensures captured values are inaccessible to the body.
        let patterns = spec
            .captures
            .iter()
            .map(|cb| &cb.pat)
            .chain(std::iter::once(&output_ident));

        let body_expr: Expr = if is_async {
            parse_quote! {
                ::anodized::__::eval_once(async || #return_type #original_body).await
            }
        } else {
            parse_quote! {
                ::anodized::__::eval_once(|| #return_type #original_body)
            }
        };
        let values = spec
            .captures
            .iter()
            .map(|cb| build_capture_eval(&cb.expr))
            .chain(std::iter::once(body_expr));

        let captures_and_output = quote! {
            let (#(#patterns),*) = (#(#values),*);
        };

        // Generate postcondition checks.
        let mut postcond_checks: Vec<Stmt> = vec![parse_quote! {
            let __anodized_post = true;
        }];
        for postinvariant in &spec.maintains {
            let eval = build_cond_eval(&postinvariant.expr);
            let instrumented_eval = self.instrument_cond_eval(
                "postinvariant failed: {}",
                &postinvariant.cfg,
                &eval,
                &postinvariant.expr.to_token_stream().to_string(),
            );
            let check = build_postcond_check(&None, &instrumented_eval);
            postcond_checks.push(check);
        }
        for postcondition in &spec.ensures {
            let eval = build_cond_eval(&postcondition.expr);
            let instrumented_eval = self.instrument_cond_eval(
                "postcondition failed: {}",
                &postcondition.cfg,
                &eval,
                &postcondition.expr.to_token_stream().to_string(),
            );
            let check = build_postcond_check(&postcondition.pat, &instrumented_eval);
            postcond_checks.push(check);
        }

        let (output_expr, precond_fail_action, postcond_fail_action) =
            if let Some(ref panic_settings) = self.does_panic
                && panic_settings.has_try_fn
            {
                (
                    quote! { Ok(#output_ident) },
                    Some(parse_quote! { return ::anodized::result::pre_err(); }),
                    Some(parse_quote! { return ::anodized::result::post_err(#output_ident); }),
                )
            } else {
                (
                    quote! { #output_ident },
                    self.build_fail_action("precondition failed"),
                    self.build_fail_action("postcondition failed"),
                )
            };

        Ok(parse_quote! {
            {
                #(#precond_checks)*
                if !__anodized_pre {
                    #precond_fail_action
                }
                #captures_and_output
                #(#postcond_checks)*
                if !__anodized_post {
                    #postcond_fail_action
                }
                #output_expr
            }
        })
    }

    fn instrument_cond_eval(&self, msg: &str, cfg: &Option<Meta>, cond: &Expr, repr: &str) -> Expr {
        let span = cond.span();

        let guard: Option<Expr> = if self.does_print || self.does_panic.is_some() {
            cfg.as_ref().map(|meta| parse_quote! { !cfg!(#meta) })
        } else {
            Some(parse_quote! { true })
        };

        let printer: Option<Expr> = if self.does_print {
            Some(parse_quote! { eprintln!(#msg, #repr) != () })
        } else {
            None
        };

        let maybe_exprs = [guard.as_ref(), Some(cond), printer.as_ref()];
        let exprs = maybe_exprs.iter().flatten();

        if exprs.clone().count() > 1 {
            parse_quote_spanned! { span => ( #(#exprs)||* ) }
        } else {
            parse_quote_spanned! { span => #(#exprs)||* }
        }
    }

    fn build_fail_action(&self, message: &str) -> Option<Stmt> {
        self.does_panic
            .as_ref()
            .map(|_| parse_quote! { panic!(#message); })
    }
}

fn build_cond_eval(expr: &Expr) -> Expr {
    let span = expr.span();
    parse_quote_spanned! { span => ::anodized::__::eval::<bool>(|| #expr) }
}

fn build_capture_eval(expr: &Expr) -> Expr {
    parse_quote! { ::anodized::__::eval(|| #expr) }
}

fn build_precond_check(expr: &Expr) -> Stmt {
    parse_quote! {
        let __anodized_pre = __anodized_pre & #expr;
    }
}

fn build_postcond_check(tame_pat: &Option<TamePat>, expr: &Expr) -> Stmt {
    match tame_pat {
        Some(TamePat::Borrowing(brw_pat)) => {
            parse_quote! {
                let (__anodized_post, __anodized_output) = ::anodized::__::apply_keep(
                    |__anodized_output| {
                        ::anodized::__::coerce_input(
                            #[allow(unused)] |#brw_pat| (), &__anodized_output);
                        let #brw_pat = __anodized_output else { unreachable!() };
                        (__anodized_post & #expr, __anodized_output)
                    },
                    __anodized_output,
                );
            }
        }
        Some(TamePat::Invertible(inv_pat, inv_expr)) => {
            parse_quote! {
                let (__anodized_post, __anodized_output) = ::anodized::__::apply_keep(
                    |#inv_pat| (__anodized_post & #expr, #inv_expr),
                    __anodized_output,
                );
            }
        }
        None => {
            parse_quote! {
                let __anodized_post = __anodized_post & #expr;
            }
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
