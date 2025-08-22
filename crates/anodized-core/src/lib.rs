#![doc = include_str!("../README.md")]

use proc_macro2::Span;
use quote::{ToTokens, quote};
use syn::{
    Block, Expr, ExprClosure, Ident, ItemFn, Meta, Pat, Token, parenthesized,
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

/// A Condition represented by a `bool`-valued expression.
#[derive(Debug)]
pub struct Condition {
    /// The expression.
    pub expr: Expr,
    /// A setting to control when the condition should be present via a `#[cfg]` annotation.
    pub cfg: Option<Meta>,
}

/// A Condition represented by a `bool`-valued closure.
#[derive(Debug)]
pub struct ConditionClosure {
    /// The closure.
    pub closure: ExprClosure,
    /// A setting to control when the condition should be present via a `#[cfg]` annotation.
    pub cfg: Option<Meta>,
}

impl Condition {
    fn from_expr(expr: Expr) -> Self {
        Self { expr, cfg: None }
    }
}

impl ConditionClosure {
    fn from_closure(closure: ExprClosure) -> Self {
        Self { closure, cfg: None }
    }
}

#[derive(PartialEq, PartialOrd, Clone, Copy, Debug)]
enum ArgOrder {
    Requires,
    Maintains,
    Binds,
    Ensures,
}

impl TryFrom<ContractArgs> for Contract {
    type Error = syn::Error;

    fn try_from(args: ContractArgs) -> Result<Self> {
        let mut last_arg_order: Option<ArgOrder> = None;
        let mut binds_pattern: Option<Pat> = None;
        let mut requires: Vec<Condition> = vec![];
        let mut maintains: Vec<Condition> = vec![];
        let mut ensures_exprs: Vec<Condition> = vec![];

        for arg in args.items {
            let current_arg_order = arg.get_order();
            if let Some(last_order) = last_arg_order {
                if current_arg_order < last_order {
                    return Err(syn::Error::new(
                        arg.span(),
                        "parameters are out of order: their order must be `requires`, `maintains`, `binds`, `ensures`",
                    ));
                }
            }
            last_arg_order = Some(current_arg_order);

            match arg {
                ContractArg::Requires { cfg, expr } => {
                    if let Expr::Array(conditions) = expr {
                        requires.extend(conditions.elems.into_iter().map(|expr| Condition {
                            expr,
                            cfg: cfg.clone(),
                        }));
                    } else {
                        requires.push(Condition { expr, cfg });
                    }
                }
                ContractArg::Maintains { cfg, expr } => {
                    if let Expr::Array(conditions) = expr {
                        maintains.extend(conditions.elems.into_iter().map(|expr| Condition {
                            expr,
                            cfg: cfg.clone(),
                        }));
                    } else {
                        maintains.push(Condition { expr, cfg });
                    }
                }
                ContractArg::Binds { pattern } => {
                    if binds_pattern.is_some() {
                        return Err(syn::Error::new(
                            pattern.span(),
                            "multiple `binds` parameters are not allowed",
                        ));
                    }
                    binds_pattern = Some(pattern);
                }
                ContractArg::Ensures { cfg, expr } => {
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

/// A container for all parsed arguments from the `#[contract]` attribute.
pub struct ContractArgs {
    pub items: Vec<ContractArg>,
}

impl Parse for ContractArgs {
    /// Custom parser for the contents of `#[contract(...)]`.
    fn parse(input: ParseStream) -> Result<Self> {
        // The arguments are a comma-separated list of conditions or a `binds` setting.
        let items = Punctuated::<ContractArg, Token![,]>::parse_terminated(input)?;

        Ok(ContractArgs {
            items: items.into_iter().collect(),
        })
    }
}

/// An intermediate enum to help parse either a condition or a `binds` setting.
pub enum ContractArg {
    Requires { cfg: Option<Meta>, expr: Expr },
    Ensures { cfg: Option<Meta>, expr: Expr },
    Maintains { cfg: Option<Meta>, expr: Expr },
    Binds { pattern: Pat },
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

    fn span(&self) -> Span {
        match self {
            ContractArg::Requires { expr, .. } => expr.span(),
            ContractArg::Ensures { expr, .. } => expr.span(),
            ContractArg::Maintains { expr, .. } => expr.span(),
            ContractArg::Binds { pattern } => pattern.span(),
        }
    }
}

impl Parse for ContractArg {
    fn parse(input: ParseStream) -> Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(kw::binds) {
            // Parse `binds: <pattern>`
            input.parse::<kw::binds>()?;
            input.parse::<Token![:]>()?;
            Ok(ContractArg::Binds {
                pattern: Pat::parse_single(input)?,
            })
        } else if lookahead.peek(kw::requires) {
            // Parse `requires: <expr>`
            input.parse::<kw::requires>()?;
            let cfg = if input.peek(syn::token::Paren) {
                let content;
                parenthesized!(content in input);
                Some(content.parse()?)
            } else {
                None
            };
            input.parse::<Token![:]>()?;
            Ok(ContractArg::Requires {
                cfg,
                expr: input.parse()?,
            })
        } else if lookahead.peek(kw::maintains) {
            // Parse `maintains: <expr>`
            input.parse::<kw::maintains>()?;
            let cfg = if input.peek(syn::token::Paren) {
                let content;
                parenthesized!(content in input);
                Some(content.parse()?)
            } else {
                None
            };
            input.parse::<Token![:]>()?;
            Ok(ContractArg::Maintains {
                cfg,
                expr: input.parse()?,
            })
        } else if lookahead.peek(kw::ensures) {
            // Parse `ensures: <expr>`
            input.parse::<kw::ensures>()?;
            let cfg = if input.peek(syn::token::Paren) {
                let content;
                parenthesized!(content in input);
                Some(content.parse()?)
            } else {
                None
            };
            input.parse::<Token![:]>()?;
            Ok(ContractArg::Ensures {
                cfg,
                expr: input.parse()?,
            })
        } else {
            Err(lookahead.error())
        }
    }
}

// Custom keywords for parsing. This allows us to use `requires`, `ensures`, etc.,
// as if they were built-in Rust keywords during parsing.
mod kw {
    syn::custom_keyword!(requires);
    syn::custom_keyword!(maintains);
    syn::custom_keyword!(binds);
    syn::custom_keyword!(ensures);
}

/// Takes the contract and the function, and returns a new instrumented function body.
pub fn instrument_function_body(contract: &Contract, func: &ItemFn) -> Result<Block> {
    let original_body = &func.block;
    let is_async = func.sig.asyncness.is_some();

    // The identifier for the return value binding. It's hygienic to prevent collisions.
    let binding_ident = Ident::new("__anodized_output", Span::mixed_site());

    // --- Generate Precondition Checks ---
    let preconditions = contract
        .requires
        .iter()
        .map(|condition| {
            let predicate = &condition.expr;
            let msg = format!("Precondition failed: {}", predicate.to_token_stream());
            let cfg = condition.cfg.as_ref().map(|c| quote! { #[cfg(#c)] });
            quote! { #cfg assert!(#predicate, #msg); }
        })
        .chain(contract.maintains.iter().map(|condition| {
            let predicate = &condition.expr;
            let msg = format!("Pre-invariant failed: {}", predicate.to_token_stream());
            let cfg = condition.cfg.as_ref().map(|c| quote! { #[cfg(#c)] });
            quote! { #cfg assert!(#predicate, #msg); }
        }));

    // --- Generate Postcondition Checks ---
    let postconditions = contract
        .maintains
        .iter()
        .map(|condition| {
            let predicate = &condition.expr;
            let msg = format!("Post-invariant failed: {}", predicate.to_token_stream());
            let cfg = condition.cfg.as_ref().map(|c| quote! { #[cfg(#c)] });
            quote! { #cfg assert!(#predicate, #msg); }
        })
        .chain(contract.ensures.iter().map(|condition_closure| {
            let closure = &condition_closure.closure;
            let msg = format!("Postcondition failed: {}", closure.to_token_stream());
            let cfg = condition_closure
                .cfg
                .as_ref()
                .map(|c| quote! { #[cfg(#c)] });
            quote! { #cfg assert!((#closure)(#binding_ident), #msg); }
        }));

    // --- Construct the New Body ---
    let body_expr = if is_async {
        quote! { async { #original_body }.await }
    } else {
        quote! { { #original_body } }
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
mod test;

#[cfg(test)]
mod test_util;