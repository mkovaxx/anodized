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
    Capture, Condition, PostCondition, Spec,
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
            let eval = build_cond_eval(&condition.expr);
            statements.push(parse_quote! { let #name = #eval; });
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
        ensures: &[PostCondition],
    ) -> Result<Block> {
        let mut statements: Vec<Stmt> = vec![];
        let mut clauses: Vec<Expr> = vec![];

        for condition in maintains {
            let i = clauses.len();
            let name = Ident::new(&format!("__anodized_clause_{}", i + 1), Span::mixed_site());
            let eval = build_cond_eval(&condition.expr);
            statements.push(parse_quote! { let #name = #eval; });
            clauses.push(parse_quote! { #name });
        }

        {
            let patterns = captures.iter().map(|capture| &capture.pat);
            let values = captures
                .iter()
                .map(|capture| build_capture_eval(&capture.expr));
            statements.push(parse_quote! { let (#(#patterns),*) = (#(#values),*); });
        }

        for postcond in ensures {
            let i = clauses.len();
            let name = Ident::new(&format!("__anodized_clause_{}", i + 1), Span::mixed_site());
            let expr = &postcond.expr;
            let eval = if let Some(pat) = &postcond.pat {
                build_cond_eval(&parse_quote! {
                    { let #pat = __anodized_output; #expr }
                })
            } else {
                build_cond_eval(expr)
            };
            statements.push(parse_quote! { let #name = #eval; });
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
        let mut precond_checks: Vec<Stmt> = vec![parse_quote! {
            let __anodized_pre = true;
        }];
        for condition in spec.requires.iter().chain(&spec.maintains) {
            let check = self.build_precond_check(&condition.cfg, &condition.expr);
            precond_checks.push(parse_quote! {
                let __anodized_pre = __anodized_pre & #check;
            });
        }

        // Bind capture values and function output in a single tuple assignment.
        // This ensures captured values are inaccessible to the body.
        let patterns = spec
            .captures
            .iter()
            .map(|cb| &cb.pat)
            .chain(std::iter::once(&output_ident));

        let body_expr: Expr = if is_async {
            parse_quote! { (async || #return_type #original_body)().await }
        } else {
            parse_quote! { (|| #return_type #original_body)() }
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
        for condition in &spec.maintains {
            let check = self.build_postcond_check(&condition.cfg, &None, &condition.expr);
            postcond_checks.push(parse_quote! {
                let __anodized_post = __anodized_post & #check;
            });
        }
        for postcond in &spec.ensures {
            let check = self.build_postcond_check(&postcond.cfg, &postcond.pat, &postcond.expr);
            postcond_checks.push(parse_quote! {
                let __anodized_post = __anodized_post & #check;
            });
        }

        let do_run_checks = self.does_print || self.does_panic.is_some();

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
                if #do_run_checks {
                    #(#precond_checks)*
                    if !__anodized_pre {
                        #precond_fail_action
                    }
                }
                #captures_and_output
                if #do_run_checks {
                    #(#postcond_checks)*
                    if !__anodized_post {
                        #postcond_fail_action
                    }
                }
                #output_expr
            }
        })
    }

    fn build_precond_check(&self, cfg: &Option<Meta>, expr: &Expr) -> Expr {
        let repr = expr.to_token_stream().to_string();
        let eval = build_cond_eval(expr);
        self.build_cond_check("precondition failed: {}", cfg, &eval, &repr)
    }

    fn build_postcond_check(&self, cfg: &Option<Meta>, pat: &Option<Pat>, expr: &Expr) -> Expr {
        let repr = expr.to_token_stream().to_string();
        let eval = if let Some(pat) = pat {
            build_cond_eval(&parse_quote! {
                { let #pat = __anodized_output; #expr }
            })
        } else {
            build_cond_eval(expr)
        };
        self.build_cond_check("postcondition failed: {}", cfg, &eval, &repr)
    }

    fn build_cond_check(&self, msg: &str, cfg: &Option<Meta>, expr: &Expr, repr: &str) -> Expr {
        if self.does_print {
            let cfg_guard = match cfg {
                Some(meta) => quote! { !cfg!(#meta) || },
                None => quote!(),
            };
            parse_quote! {
                ( #cfg_guard #expr || eprintln!(#msg, #repr) != () )
            }
        } else {
            expr.clone()
        }
    }

    fn build_fail_action(&self, message: &str) -> Option<Stmt> {
        self.does_panic
            .as_ref()
            .map(|_| parse_quote! { panic!(#message); })
    }
}

fn build_cond_eval(expr: &Expr) -> Expr {
    parse_quote! { ::anodized::__::eval::<bool>(|| #expr) }
}

fn build_capture_eval(expr: &Expr) -> Expr {
    parse_quote! { ::anodized::__::eval(|| #expr) }
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
