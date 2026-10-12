#[cfg(test)]
#[path = "fns_tests.rs"]
mod fns_tests;

use quote::ToTokens;
use syn::{
    Attribute, Block, Expr, FnArg, Ident, Meta, Pat, Path, ReturnType, Signature, Stmt, Token,
    Type,
    parse::{Parse, Result},
    parse_quote, parse_quote_spanned,
    punctuated::Punctuated,
    spanned::Spanned,
    token::Comma,
};

use crate::{
    Capture, Condition, FnSpec, InputSpecFlags, PostCondition,
    instrument::{CheckSettings, Mode, SpecEmbedding, patterns::TamePat},
    qualifiers::FnQualifiers,
};

impl Mode {
    pub fn instrument_fn(
        &self,
        spec: &FnSpec,
        sig: &mut Signature,
        body: &mut Block,
    ) -> Result<()> {
        self.instrument_loops_in_fn_body(body)?;

        let Mode::InjectChecks(check_config) = self else {
            return Ok(());
        };

        sanitize_input_patterns(&mut sig.inputs, &spec.input_spec_flags);

        // Instrument the function.
        check_config.instrument_fn_sig_and_body(spec, sig, body)?;

        Ok(())
    }

    pub fn build_precondition_fn_sig(
        &self,
        attrs: &mut Vec<Attribute>,
        prefix: &str,
        sig: &Signature,
    ) -> Signature {
        if let Self::EmbedSpecs(SpecEmbedding { uses_charon: true }) = self {
            let sibling = syn::LitStr::new(&sig.ident.to_string(), sig.ident.span());
            attrs.push(parse_quote!(
                #[charon::contract(kind = "precondition", for = #sibling)]
            ));
        }
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

    pub fn build_postcondition_fn_sig(
        &self,
        attrs: &mut Vec<Attribute>,
        prefix: &str,
        sig: &Signature,
    ) -> Signature {
        if let Self::EmbedSpecs(SpecEmbedding { uses_charon: true }) = self {
            let sibling = syn::LitStr::new(&sig.ident.to_string(), sig.ident.span());
            attrs.push(parse_quote!(
                #[charon::contract(kind = "postcondition", for = #sibling)]
            ));
        }
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

    pub fn build_precondition_fn_body<'a, 'b>(
        inputs: impl Iterator<Item = (&'a FnArg, &'b InputSpecFlags)> + Clone,
        requires: &[Condition],
        maintains: &[Condition],
    ) -> Block {
        let mut stmts: Vec<Stmt> = vec![];
        emit_precondition_checks(inputs, requires, maintains, &mut stmts, |eval, _, _, _| {
            eval.clone()
        });
        parse_quote! {
            {
                #(#stmts)*
                __anodized_pre
            }
        }
    }

    pub fn build_postcondition_fn_body<'a, 'b>(
        inputs: impl Iterator<Item = (&'a FnArg, &'b InputSpecFlags)> + Clone,
        output_spec: Option<&ReturnType>,
        maintains: &[Condition],
        captures: &[Capture],
        ensures: &[PostCondition],
    ) -> Block {
        let mut stmts: Vec<Stmt> = vec![];
        let output_eval: Expr = parse_quote! {
            ::anodized::__::eval_once(|| { __anodized_output })
        };
        emit_input_bindings(inputs.clone(), &mut stmts);
        emit_captures_and_output_binding(inputs.clone(), captures, output_eval, &mut stmts);
        emit_postcondition_checks(
            inputs,
            output_spec,
            maintains,
            ensures,
            &mut stmts,
            |eval, _, _, _| eval.clone(),
        );
        parse_quote! {
            {
                #(#stmts)*
                __anodized_post
            }
        }
    }
}

pub(super) fn sanitize_input_patterns(
    inputs: &mut Punctuated<FnArg, Comma>,
    input_spec_flags: &[InputSpecFlags],
) {
    for (i, (input, flags)) in inputs.iter_mut().zip(input_spec_flags).enumerate() {
        if let FnArg::Typed(pat_type) = input
            && (flags.on_entry().is_some() || flags.on_exit().is_some())
        {
            let ident = Ident::new(&format!("__anodized_input_{}", i + 1), pat_type.pat.span());
            *pat_type.pat.as_mut() = parse_quote! { #ident };
        }
    }
}

impl CheckSettings {
    fn instrument_fn_sig_and_body(
        &self,
        spec: &FnSpec,
        sig: &Signature,
        body: &mut Block,
    ) -> Result<()> {
        let inputs = sig.inputs.iter().zip(&spec.input_spec_flags);

        let (output_expr, precond_fail_action, postcond_fail_action) =
            if let Some(ref panic_settings) = self.does_panic
                && panic_settings.has_try_fn
            {
                (
                    parse_quote! { Ok(__anodized_output) },
                    Some(parse_quote! { return ::anodized::result::pre_err(); }),
                    Some(parse_quote! { return ::anodized::result::post_err(__anodized_output); }),
                )
            } else {
                (
                    parse_quote! { __anodized_output },
                    self.build_fail_action("precondition failed"),
                    self.build_fail_action("postcondition failed"),
                )
            };

        let mut stmts: Vec<Stmt> = vec![];

        for (input, flags) in inputs.clone() {
            let (FnArg::Typed(pat_type), Some(pat)) = (
                input,
                flags
                    .on_entry()
                    .or_else(|| flags.on_exit().map(TamePat::get_pat)),
            ) else {
                continue;
            };
            let ty = &pat_type.ty;
            stmts.push(parse_quote! {
                #[allow(unused)]
                let _ = |#pat: #ty| ();
            });
        }

        // Generate precondition checks.
        emit_precondition_checks(
            inputs.clone(),
            &spec.requires,
            &spec.maintains,
            &mut stmts,
            |eval, cfg, msg, repr| self.instrument_cond_eval(eval, cfg, msg, repr),
        );
        stmts.push(parse_quote! {
            if !__anodized_pre {
                #precond_fail_action
            }
        });

        // Generate the binding for captures and the return value.
        let return_type = &sig.output;
        let output_eval: Expr = if sig.asyncness.is_some() {
            parse_quote! {
                ::anodized::__::eval_once(async || #return_type #body).await
            }
        } else {
            parse_quote! {
                ::anodized::__::eval_once(|| #return_type #body)
            }
        };
        emit_captures_and_output_binding(inputs.clone(), &spec.captures, output_eval, &mut stmts);

        // Generate postcondition checks.
        emit_postcondition_checks(
            inputs,
            spec.output_spec_flag.then_some(return_type),
            &spec.maintains,
            &spec.ensures,
            &mut stmts,
            |eval, cfg, msg, repr| self.instrument_cond_eval(eval, cfg, msg, repr),
        );
        stmts.push(parse_quote! {
            if !__anodized_post {
                #postcond_fail_action
            }
        });

        stmts.push(Stmt::Expr(output_expr, None));

        *body = Block {
            brace_token: body.brace_token,
            stmts,
        };

        Ok(())
    }

    fn instrument_cond_eval(
        &self,
        cond: &Expr,
        cfg: &Option<Meta>,
        msg: &str,
        repr: &Expr,
    ) -> Expr {
        let span = cond.span();

        let guard: Option<Expr> = if self.does_print || self.does_panic.is_some() {
            cfg.as_ref().map(|meta| parse_quote! { !cfg!(#meta) })
        } else {
            Some(parse_quote! { true })
        };

        let printer: Option<Expr> = if self.does_print {
            let repr_str = repr.to_token_stream().to_string();
            Some(parse_quote! { eprintln!(#msg, #repr_str) != () })
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

trait FnInstrumentEval: Fn(&Expr, &Option<Meta>, &str, &Expr) -> Expr {}
impl<F: Fn(&Expr, &Option<Meta>, &str, &Expr) -> Expr> FnInstrumentEval for F {}

fn emit_precondition_checks<'a, 'b>(
    inputs: impl Iterator<Item = (&'a FnArg, &'b InputSpecFlags)> + Clone,
    requires: &[Condition],
    maintains: &[Condition],
    statements: &mut Vec<Stmt>,
    instrument_eval: impl FnInstrumentEval,
) {
    statements.push(parse_quote! {
        let __anodized_pre = true;
    });

    // Enforce type specs of inputs.

    for (i, (input, flags)) in inputs.clone().enumerate() {
        if flags.on_entry().is_none() {
            continue;
        }
        let instrumented_eval = match input {
            FnArg::Receiver(receiver) => {
                let message = "precondition failed: type spec of `self`, `{}`";
                let self_token = &receiver.self_token;
                let and_token: Option<Token![&]> = match receiver.reference {
                    Some(_) => None,
                    None => Some(Default::default()),
                };
                let expr = parse_quote! {
                    ::anodized::__::eval_type_spec(#and_token #self_token)
                };
                instrument_eval(&expr, &None, message, &expr)
            }
            FnArg::Typed(pat_type) => {
                let message = format!("precondition failed: type spec of input {}, `{{}}`", i + 1);
                let ident = Ident::new(&format!("__anodized_input_{}", i + 1), pat_type.pat.span());
                let expr = parse_quote! {
                    ::anodized::__::eval_type_spec(&#ident)
                };
                instrument_eval(&expr, &None, &message, &expr)
            }
        };
        let check = build_precond_check(&instrumented_eval);
        statements.push(check);
    }

    emit_input_bindings(inputs, statements);

    for precondition in requires {
        let eval = build_cond_eval(&precondition.expr);
        let instrumented_eval = instrument_eval(
            &eval,
            &precondition.cfg,
            "precondition failed: {}",
            &precondition.expr,
        );
        let check = build_precond_check(&instrumented_eval);
        statements.push(check);
    }

    for preinvariant in maintains {
        let eval = build_cond_eval(&preinvariant.expr);
        let instrumented_eval = instrument_eval(
            &eval,
            &preinvariant.cfg,
            "preinvariant failed: {}",
            &preinvariant.expr,
        );
        let check = build_precond_check(&instrumented_eval);
        statements.push(check);
    }
}

fn emit_input_bindings<'a, 'b>(
    inputs: impl Iterator<Item = (&'a FnArg, &'b InputSpecFlags)>,
    statements: &mut Vec<Stmt>,
) {
    let mut input_idents = vec![];
    let mut input_pats = vec![];

    for (i, (input, flags)) in inputs.enumerate() {
        let FnArg::Typed(pat_type) = input else {
            continue;
        };
        let Some(pat) = flags
            .on_entry()
            .or_else(|| flags.on_exit().map(TamePat::get_pat))
        else {
            continue;
        };
        let ident = Ident::new(&format!("__anodized_input_{}", i + 1), pat_type.pat.span());
        input_idents.push(ident);
        input_pats.push(pat);
    }

    if !input_pats.is_empty() {
        statements.push(parse_quote! {
            let (#(#input_pats),*) = (#(#input_idents),*) else { unreachable!() };
        });
    }
}

fn emit_captures_and_output_binding<'a, 'b>(
    inputs: impl Iterator<Item = (&'a FnArg, &'b InputSpecFlags)>,
    captures: &[Capture],
    output_eval: Expr,
    statements: &mut Vec<Stmt>,
) {
    let mut patterns = vec![];
    let mut values = vec![];

    for capture in captures {
        patterns.push(capture.pat.clone());
        let capture_eval = build_capture_eval(&capture.expr);
        values.push(capture_eval);
    }

    let output_ident = Pat::Path(parse_quote! { __anodized_output });
    patterns.push(output_ident);
    values.push(output_eval);

    for (i, (input, flags)) in inputs.enumerate() {
        let (FnArg::Typed(pat_type), Some(TamePat::Invertible(_, inv_expr))) =
            (input, flags.on_exit())
        else {
            continue;
        };
        let ident = Ident::new(&format!("__anodized_input_{}", i + 1), pat_type.pat.span());
        patterns.push(parse_quote! { #ident });
        values.push(*inv_expr.clone());
    }

    let binding = if patterns.len() > 1 {
        parse_quote! { let (#(#patterns,)*) = (#(#values,)*); }
    } else {
        parse_quote! { let #(#patterns)* = #(#values)*; }
    };

    statements.push(binding);
}

fn emit_postcondition_checks<'a, 'b>(
    inputs: impl Iterator<Item = (&'a FnArg, &'b InputSpecFlags)>,
    output_spec: Option<&ReturnType>,
    maintains: &[Condition],
    ensures: &[PostCondition],
    statements: &mut Vec<Stmt>,
    instrument_eval: impl FnInstrumentEval,
) {
    statements.push(parse_quote! {
        let __anodized_post = true;
    });

    if output_spec.is_some() {
        let expr = parse_quote! {
            ::anodized::__::eval_type_spec(&__anodized_output)
        };
        let instrumented_eval = instrument_eval(
            &expr,
            &None,
            "postcondition failed: type spec of output, `{}`",
            &expr,
        );
        statements.push(build_postcond_check(&None, &instrumented_eval));
    }

    // Enforce type specs of inputs on exit.

    let mut input_idents = vec![];
    let mut input_pats = vec![];
    for (i, (input, flags)) in inputs.enumerate() {
        let Some(tame_pat) = flags.on_exit() else {
            continue;
        };
        let instrumented_eval = match input {
            FnArg::Receiver(receiver) => {
                let message = "postcondition failed: type spec of `self`, `{}`";
                let self_token = &receiver.self_token;
                let and_token: Option<Token![&]> = match receiver.reference {
                    Some(_) => None,
                    None => Some(Default::default()),
                };
                let expr = parse_quote! {
                    ::anodized::__::eval_type_spec(#and_token #self_token)
                };
                instrument_eval(&expr, &None, message, &expr)
            }
            FnArg::Typed(pat_type) => {
                let message = format!("postcondition failed: type spec of input {}, `{{}}`", i + 1);
                let ident = Ident::new(&format!("__anodized_input_{}", i + 1), pat_type.pat.span());
                let expr = parse_quote! {
                    ::anodized::__::eval_type_spec(&#ident)
                };
                if let TamePat::Invertible(pat, _) = tame_pat {
                    input_idents.push(ident);
                    input_pats.push(pat);
                }
                instrument_eval(&expr, &None, &message, &expr)
            }
        };
        let check = build_postcond_check(&None, &instrumented_eval);
        statements.push(check);
    }

    if !input_pats.is_empty() {
        statements.push(parse_quote! {
            let (#(#input_pats),*) = (#(#input_idents),*) else { unreachable!() };
        });
    }

    for postinvariant in maintains {
        let eval = build_cond_eval(&postinvariant.expr);
        let instrumented_eval = instrument_eval(
            &eval,
            &postinvariant.cfg,
            "postinvariant failed: {}",
            &postinvariant.expr,
        );
        let check = build_postcond_check(&None, &instrumented_eval);
        statements.push(check);
    }

    for postcondition in ensures {
        let eval = build_cond_eval(&postcondition.expr);
        let instrumented_eval = instrument_eval(
            &eval,
            &postcondition.cfg,
            "postcondition failed: {}",
            &postcondition.expr,
        );
        let check = build_postcond_check(&postcondition.pat, &instrumented_eval);
        statements.push(check);
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
