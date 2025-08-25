#![cfg(not(doctest))]
#![doc = include_str!("../README.md")]

use proc_macro2::Span;
use quote::{ToTokens, quote};
use syn::{
    Attribute, Block, Expr, ExprClosure, Ident, Meta, Pat, Token,
    parse::{Parse, ParseStream, Result},
    parse_quote,
    punctuated::Punctuated,
    spanned::Spanned,
};

/// A contract specifies the intended behavior of a function or method.
#[derive(Debug)]
pub struct Contract {
    /// Preconditions: conditions that must hold when the function is called.
    pub requires: Vec<Condition>,
    /// Invariants: conditions that must hold both when the function is called and when it returns.
    pub maintains: Vec<Condition>,
    /// Postconditions: conditions that must hold when the function returns.
    pub ensures: Vec<ConditionClosure>,
}

/// A condition represented by a `bool`-valued expression.
#[derive(Debug)]
pub struct Condition {
    /// The expression.
    pub expr: Expr,
    /// **Static analyzers can safely ignore this field.**
    ///
    /// Build configuration filter to decide whether to add runtime checks.
    /// Passed to a `cfg!()` guard in the instrumented function.
    pub cfg: Option<Meta>,
}

/// A condition represented by a `bool`-valued closure.
#[derive(Debug)]
pub struct ConditionClosure {
    /// The closure.
    pub closure: ExprClosure,
    /// **Static analyzers can safely ignore this field.**
    ///
    /// Build configuration filter to decide whether to add runtime checks.
    /// Passed to a `cfg!()` guard in the instrumented function.
    pub cfg: Option<Meta>,
}

impl Parse for Contract {
    fn parse(input: ParseStream) -> Result<Self> {
        let args = Punctuated::<ContractArg, Token![,]>::parse_terminated(input)?;

        let mut last_arg_order: Option<ArgOrder> = None;
        let mut requires: Vec<Condition> = vec![];
        let mut maintains: Vec<Condition> = vec![];
        let mut binds_pattern: Option<Pat> = None;
        let mut ensures_exprs: Vec<Condition> = vec![];

        for arg in args {
            let current_arg_order = arg.get_order();
            if let Some(last_order) = last_arg_order {
                if current_arg_order < last_order {
                    return Err(syn::Error::new(
                        arg.get_keyword_span(),
                        "parameters are out of order: their order must be `requires`, `maintains`, `binds`, `ensures`",
                    ));
                }
            }
            last_arg_order = Some(current_arg_order);

            match arg {
                ContractArg::Requires { cfg, expr, .. } => {
                    if let Expr::Array(conditions) = expr {
                        requires.extend(conditions.elems.into_iter().map(|expr| Condition {
                            expr,
                            cfg: cfg.clone(),
                        }));
                    } else {
                        requires.push(Condition { expr, cfg });
                    }
                }
                ContractArg::Maintains { cfg, expr, .. } => {
                    if let Expr::Array(conditions) = expr {
                        maintains.extend(conditions.elems.into_iter().map(|expr| Condition {
                            expr,
                            cfg: cfg.clone(),
                        }));
                    } else {
                        maintains.push(Condition { expr, cfg });
                    }
                }
                ContractArg::Binds { keyword, pattern } => {
                    if binds_pattern.is_some() {
                        return Err(syn::Error::new(
                            keyword.span(),
                            "multiple `binds` parameters are not allowed",
                        ));
                    }
                    binds_pattern = Some(pattern);
                }
                ContractArg::Ensures { cfg, expr, .. } => {
                    if let Expr::Array(conditions) = expr {
                        ensures_exprs.extend(conditions.elems.into_iter().map(|expr| Condition {
                            expr,
                            cfg: cfg.clone(),
                        }));
                    } else {
                        ensures_exprs.push(Condition { expr, cfg });
                    }
                }
            }
        }

        let default_closure_pattern = binds_pattern
            .as_ref()
            .map(|p| p.to_token_stream())
            .unwrap_or_else(|| quote! { output });

        let ensures = ensures_exprs
            .into_iter()
            .map(|condition| {
                let closure: ExprClosure = if let Expr::Closure(closure) = condition.expr {
                    closure
                } else {
                    // Convert "naked" postcondition to closure
                    let inner_condition = condition.expr;
                    parse_quote! { |#default_closure_pattern| #inner_condition }
                };
                Ok(ConditionClosure {
                    closure,
                    cfg: condition.cfg,
                })
            })
            .collect::<Result<Vec<ConditionClosure>>>()?;

        Ok(Contract {
            requires,
            maintains,
            ensures,
        })
    }
}

#[derive(PartialEq, PartialOrd, Clone, Copy, Debug)]
enum ArgOrder {
    Requires,
    Maintains,
    Binds,
    Ensures,
}

/// An intermediate enum to help parse either a condition or a `binds` setting.
enum ContractArg {
    Requires {
        keyword: kw::requires,
        cfg: Option<Meta>,
        expr: Expr,
    },
    Ensures {
        keyword: kw::ensures,
        cfg: Option<Meta>,
        expr: Expr,
    },
    Maintains {
        keyword: kw::maintains,
        cfg: Option<Meta>,
        expr: Expr,
    },
    Binds {
        keyword: kw::binds,
        pattern: Pat,
    },
}

impl ContractArg {
    fn get_order(&self) -> ArgOrder {
        match self {
            ContractArg::Requires { .. } => ArgOrder::Requires,
            ContractArg::Maintains { .. } => ArgOrder::Maintains,
            ContractArg::Binds { .. } => ArgOrder::Binds,
            ContractArg::Ensures { .. } => ArgOrder::Ensures,
        }
    }

    fn get_keyword_span(&self) -> Span {
        match self {
            ContractArg::Requires { keyword, .. } => keyword.span,
            ContractArg::Ensures { keyword, .. } => keyword.span,
            ContractArg::Maintains { keyword, .. } => keyword.span,
            ContractArg::Binds { keyword, .. } => keyword.span,
        }
    }
}

impl Parse for ContractArg {
    fn parse(input: ParseStream) -> Result<Self> {
        let attrs = input.call(Attribute::parse_outer)?;
        let cfg = parse_cfg_attribute(&attrs)?;

        let lookahead = input.lookahead1();
        if lookahead.peek(kw::binds) {
            if cfg.is_some() {
                return Err(syn::Error::new(
                    attrs[0].span(),
                    "`cfg` attribute is not supported on `binds`",
                ));
            }

            // Parse `binds: <pattern>`
            let keyword = input.parse::<kw::binds>()?;
            input.parse::<Token![:]>()?;
            Ok(ContractArg::Binds {
                keyword,
                pattern: Pat::parse_single(input)?,
            })
        } else if lookahead.peek(kw::requires) {
            // Parse `requires: <conditions>`
            let keyword = input.parse::<kw::requires>()?;
            input.parse::<Token![:]>()?;
            Ok(ContractArg::Requires {
                keyword,
                cfg,
                expr: input.parse()?,
            })
        } else if lookahead.peek(kw::maintains) {
            // Parse `maintains: <conditions>`
            let keyword = input.parse::<kw::maintains>()?;
            input.parse::<Token![:]>()?;
            Ok(ContractArg::Maintains {
                keyword,
                cfg,
                expr: input.parse()?,
            })
        } else if lookahead.peek(kw::ensures) {
            // Parse `ensures: <conditions>`
            let keyword = input.parse::<kw::ensures>()?;
            input.parse::<Token![:]>()?;
            Ok(ContractArg::Ensures {
                keyword,
                cfg,
                expr: input.parse()?,
            })
        } else {
            Err(lookahead.error())
        }
    }
}

fn parse_cfg_attribute(attrs: &[Attribute]) -> Result<Option<Meta>> {
    let mut cfg_attrs: Vec<Meta> = vec![];

    for attr in attrs {
        if attr.path().is_ident("cfg") {
            cfg_attrs.push(attr.parse_args()?);
        } else {
            return Err(syn::Error::new(
                attr.span(),
                "unsupported attribute; only `cfg` is allowed",
            ));
        }
    }

    if cfg_attrs.len() > 1 {
        return Err(syn::Error::new(
            cfg_attrs[1].span(),
            "multiple `cfg` attributes are not supported",
        ));
    }

    Ok(cfg_attrs.pop())
}

/// Custom keywords for parsing. This allows us to use `requires`, `ensures`, etc.,
/// as if they were built-in Rust keywords during parsing.
mod kw {
    syn::custom_keyword!(requires);
    syn::custom_keyword!(maintains);
    syn::custom_keyword!(binds);
    syn::custom_keyword!(ensures);
}

/// Takes the contract and the original body and returns a new instrumented function body.
pub fn instrument_fn_body(
    contract: &Contract,
    original_body: &Block,
    is_async: bool,
) -> Result<Block> {
    // The identifier for the return value binding. It's hygienic to prevent collisions.
    let binding_ident = Ident::new("__anodized_output", Span::mixed_site());

    // --- Generate Precondition Checks ---
    let preconditions = contract
        .requires
        .iter()
        .map(|condition| {
            let expr = &condition.expr;
            let msg = format!("Precondition failed: {}", expr.to_token_stream());
            let assert = quote! { assert!(#expr, #msg); };
            if let Some(cfg) = &condition.cfg {
                quote! { if cfg!(#cfg) { #assert } }
            } else {
                assert
            }
        })
        .chain(contract.maintains.iter().map(|condition| {
            let expr = &condition.expr;
            let msg = format!("Pre-invariant failed: {}", expr.to_token_stream());
            let assert = quote! { assert!(#expr, #msg); };
            if let Some(cfg) = &condition.cfg {
                quote! { if cfg!(#cfg) { #assert } }
            } else {
                assert
            }
        }));

    // --- Generate Postcondition Checks ---
    let postconditions = contract
        .maintains
        .iter()
        .map(|condition| {
            let expr = &condition.expr;
            let msg = format!("Post-invariant failed: {}", expr.to_token_stream());
            let assert = quote! { assert!(#expr, #msg); };
            if let Some(cfg) = &condition.cfg {
                quote! { if cfg!(#cfg) { #assert } }
            } else {
                assert
            }
        })
        .chain(contract.ensures.iter().map(|condition_closure| {
            let closure = &condition_closure.closure;
            let msg = format!("Postcondition failed: {}", closure.to_token_stream());
            let assert = quote! { assert!((#closure)(#binding_ident), #msg); };
            if let Some(cfg) = &condition_closure.cfg {
                quote! { if cfg!(#cfg) { #assert } }
            } else {
                assert
            }
        }));

    // --- Construct the New Body ---
    let body_expr = if is_async {
        quote! { async #original_body.await }
    } else {
        quote! { #original_body }
    };

    Ok(parse_quote! {
        {
            #(#preconditions)*
            let #binding_ident = #body_expr;
            #(#postconditions)*
            #binding_ident
        }
    })
}

#[cfg(test)]
mod test_parse_contract;

#[cfg(test)]
mod test_instrument_fn;

#[cfg(test)]
mod test_util;
