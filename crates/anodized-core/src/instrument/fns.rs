#[cfg(test)]
#[path = "fns_tests.rs"]
mod fns_tests;

use proc_macro2::Span;
use quote::{ToTokens, quote};
use syn::{
    Attribute, Block, Expr, FnArg, Ident, Meta, Pat, Path, ReturnType, Signature, Stmt, Type,
    parse::{Parse, Result},
    parse_quote, parse_quote_spanned,
    spanned::Spanned,
};

use crate::{
    Capture, Condition, PostCondition, Spec,
    instrument::{
        CheckSettings, Mode,
        patterns::{IdentGenerator, TamePat, tame_pattern},
    },
    qualifiers::FnQualifiers,
};

impl Mode {
    pub fn instrument_fn(
        &self,
        spec: &Spec,
        sig: &mut Signature,
        body: &mut Block,
    ) -> syn::Result<()> {
        self.instrument_loops_in_fn_body(body)?;

        let Mode::InjectChecks(check_config) = self else {
            return Ok(());
        };

        // Instrument the function.
        check_config.instrument_fn_sig_and_body(spec, sig, body)?;

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
    fn instrument_fn_sig_and_body(
        &self,
        spec: &Spec,
        sig: &mut Signature,
        body: &mut Block,
    ) -> Result<()> {
        // The identifier for the return value binding.
        let output_ident: Pat = parse_quote!(__anodized_output);
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

        let mut stmts: Vec<Stmt> = vec![];

        let mut checked_inputs: Vec<(Ident, TamePat, &Type)> = vec![];

        let mut id_gen = IdentGenerator::new();
        if self.check_data {
            for (i, input) in sig.inputs.iter_mut().enumerate() {
                match input {
                    FnArg::Receiver(_) => {
                        todo!()
                    }
                    FnArg::Typed(arg) => {
                        let ident =
                            Ident::new(&format!("__anodized_input_{}", i + 1), arg.pat.span());
                        let coercion = parse_quote! {
                            let _ = |#arg| ();
                        };
                        let new_pat: Pat = parse_quote! { #ident };
                        let pat: Pat = std::mem::replace(&mut arg.pat, new_pat);
                        stmts.push(coercion);
                        let tame_pat = tame_pattern(&mut id_gen, pat)?;
                        checked_inputs.push((ident, tame_pat, arg.ty.as_ref()));
                    }
                }
            }
        }

        // Generate precondition checks.
        stmts.push(parse_quote! {
            let __anodized_pre = true;
        });

        if self.check_data {
            // Check data specs of inputs.
            for (i, (ident, _, ty)) in checked_inputs.iter().enumerate() {
                let message = format!("precondition failed: data spec of input {}", i + 1);
                let expr = parse_quote! {
                    <#ty as ::anodized::types::Refine>::predicate(&#ident)
                };
                let check = self.build_precond_check("{}", &None, expr, &message);
                stmts.push(check);
            }
            // Bind input patterns.
            let input_idents = checked_inputs.iter().map(|(ident, _, _)| ident);
            let input_pats = checked_inputs
                .iter()
                .map(|(_, tame_pat, _)| match tame_pat {
                    TamePat::Borrowing(pat) | TamePat::Invertible(pat, _) => pat,
                });
            stmts.push(parse_quote! {
                let (#(#input_pats),*) = (#(#input_idents),*) else { unreachable!() };
            });
        }

        for precondition in &spec.requires {
            let check = self.build_precond_check(
                "precondition failed: {}",
                &precondition.cfg,
                build_cond_eval(&precondition.expr),
                &precondition.expr.to_token_stream().to_string(),
            );
            stmts.push(check);
        }
        for preinvariant in &spec.maintains {
            let check = self.build_precond_check(
                "preinvariant failed: {}",
                &preinvariant.cfg,
                build_cond_eval(&preinvariant.expr),
                &preinvariant.expr.to_token_stream().to_string(),
            );
            stmts.push(check);
        }
        stmts.push(parse_quote! {
            if !__anodized_pre {
                #precond_fail_action
            }
        });

        // Bind capture values and function output in a single tuple assignment.
        // This ensures captured values are inaccessible to the body.
        let patterns = spec
            .captures
            .iter()
            .map(|cb| &cb.pat)
            .chain(std::iter::once(&output_ident));

        let output = &sig.output;
        let body_expr: Expr = if sig.asyncness.is_some() {
            parse_quote! {
                ::anodized::__::eval_once(async || #output #body).await
            }
        } else {
            parse_quote! {
                ::anodized::__::eval_once(|| #output #body)
            }
        };
        let values = spec
            .captures
            .iter()
            .map(|cb| build_capture_eval(&cb.expr))
            .chain(std::iter::once(body_expr));

        stmts.push(parse_quote! {
            let (#(#patterns),*) = (#(#values),*);
        });

        // Generate postcondition checks.
        stmts.push(parse_quote! {
            let __anodized_post = true;
        });

        if self.check_data {
            let ret_type = match &sig.output {
                ReturnType::Default => quote! { () },
                ReturnType::Type(_, ty) => ty.to_token_stream(),
            };
            // Check data spec of the output.
            let message = "postcondition failed: data spec of output";
            let expr = parse_quote! {
                <#ret_type as ::anodized::types::Refine>::predicate(&#output_ident)
            };
            let check = self.build_postcond_check("{}", &None, &None, expr, message);
            stmts.push(check);
            // Unbind invertible input patterns.
            let invertible_inputs =
                checked_inputs
                    .iter()
                    .filter_map(|(ident, tame_pat, ty)| match tame_pat {
                        TamePat::Borrowing(_) => None,
                        TamePat::Invertible(pat, expr) => Some((ident, pat, expr, ty)),
                    });
            let input_inv_idents = invertible_inputs.clone().map(|(ident, _, _, _)| ident);
            let input_inv_exprs = invertible_inputs.clone().map(|(_, _, expr, _)| expr);
            stmts.push(parse_quote! {
                let (#(#input_inv_idents),*) = (#(#input_inv_exprs),*);
            });
            // Check data spec out inputs again.
            for (i, (ident, _, ty)) in checked_inputs.iter().enumerate() {
                let message = format!("postcondition failed: data spec of input {}", i + 1);
                let expr = parse_quote! {
                    <#ty as ::anodized::types::Refine>::predicate(&#ident)
                };
                let check = self.build_postcond_check("{}", &None, &None, expr, &message);
                stmts.push(check);
            }
            // Re-bind invertible inputs.
            let input_inv_idents = invertible_inputs.clone().map(|(ident, _, _, _)| ident);
            let input_inv_pats = invertible_inputs.clone().map(|(_, pat, _, _)| pat);
            stmts.push(parse_quote! {
                let (#(#input_inv_pats),*) = (#(#input_inv_idents),*) else { unreachable!() };
            });
        };

        for postinvariant in &spec.maintains {
            let check = self.build_postcond_check(
                "postinvariant failed: {}",
                &postinvariant.cfg,
                &None,
                build_cond_eval(&postinvariant.expr),
                &postinvariant.expr.to_token_stream().to_string(),
            );
            stmts.push(check);
        }
        for postcondition in &spec.ensures {
            let tame_pat = if let Some(pat) = &postcondition.pat {
                Some(tame_pattern(&mut id_gen, pat.clone())?)
            } else {
                None
            };
            let check = self.build_postcond_check(
                "postcondition failed: {}",
                &postcondition.cfg,
                &tame_pat,
                build_cond_eval(&postcondition.expr),
                &postcondition.expr.to_token_stream().to_string(),
            );
            stmts.push(check);
        }
        stmts.push(parse_quote! {
            if !__anodized_post {
                #postcond_fail_action
            }
        });

        *body = parse_quote! {
            {
                #(#stmts)*
                #output_expr
            }
        };

        Ok(())
    }

    fn build_precond_check(&self, msg: &str, cfg: &Option<Meta>, eval: Expr, repr: &str) -> Stmt {
        let check = self.build_cond_check(msg, cfg, eval, repr);
        parse_quote! {
            let __anodized_pre = __anodized_pre & #check;
        }
    }

    fn build_postcond_check(
        &self,
        msg: &str,
        cfg: &Option<Meta>,
        tame_pat: &Option<TamePat>,
        eval: Expr,
        repr: &str,
    ) -> Stmt {
        let check = self.build_cond_check(msg, cfg, eval, repr);
        match tame_pat {
            Some(TamePat::Borrowing(brw_pat)) => {
                parse_quote! {
                    let (__anodized_post, __anodized_output) = ::anodized::__::apply_keep(
                        |__anodized_output| {
                            ::anodized::__::coerce_input(
                                #[allow(unused)] |#brw_pat| (), &__anodized_output);
                            let #brw_pat = __anodized_output else { unreachable!() };
                            (__anodized_post & #check, __anodized_output)
                        },
                        __anodized_output,
                    );
                }
            }
            Some(TamePat::Invertible(inv_pat, inv_expr)) => {
                parse_quote! {
                    let (__anodized_post, __anodized_output) = ::anodized::__::apply_keep(
                        |#inv_pat| (__anodized_post & #check, #inv_expr),
                        __anodized_output,
                    );
                }
            }
            None => {
                parse_quote! {
                    let __anodized_post = __anodized_post & #check;
                }
            }
        }
    }

    fn build_cond_check(&self, msg: &str, cfg: &Option<Meta>, cond: Expr, repr: &str) -> Expr {
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

        let maybe_exprs = [guard, Some(cond), printer];
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
