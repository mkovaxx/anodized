#![doc = include_str!("../README.md")]

use proc_macro2::Span;
use quote::{ToTokens, quote};
use syn::{
    Block, Expr, ExprClosure, Ident, ItemFn, Pat, Token,
    parse::{Parse, ParseStream, Result},
    parse_quote,
    punctuated::Punctuated,
    spanned::Spanned,
};

/// A contract specifies the intended behavior of a function or method.
#[derive(Debug)]
pub struct Contract {
    /// Preconditions: conditions that must hold when the function is called.
    pub requires: Vec<Expr>,
    /// Invariants: conditions that must hold both when the function is called and when it returns.
    pub maintains: Vec<Expr>,
    /// Postconditions: conditions that must hold when the function returns.
    pub ensures: Vec<ExprClosure>,
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
        let mut requires: Vec<Expr> = vec![];
        let mut maintains: Vec<Expr> = vec![];
        let mut ensures_exprs: Vec<Expr> = vec![];

        for arg in args.items {
            let current_arg_order = arg.get_order();
            if let Some(last_order) = last_arg_order {
                if current_arg_order < last_order {
                    return Err(syn::Error::new(
                        arg.span(),
                        "parameters are out of order: it must be `requires`, `maintains`, `binds`, `ensures`",
                    ));
                }
            }
            last_arg_order = Some(current_arg_order);

            match arg {
                ContractArg::Requires { expr } => {
                    if let Expr::Array(conditions) = expr {
                        requires.extend(conditions.elems);
                    } else {
                        requires.push(expr);
                    }
                }
                ContractArg::Maintains { expr } => {
                    if let Expr::Array(conditions) = expr {
                        maintains.extend(conditions.elems);
                    } else {
                        maintains.push(expr);
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
                ContractArg::Ensures { expr } => {
                    if let Expr::Array(conditions) = expr {
                        ensures_exprs.extend(conditions.elems);
                    } else {
                        ensures_exprs.push(expr);
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
                if let Expr::Closure(closure) = condition {
                    Ok(closure)
                } else {
                    let closure: ExprClosure =
                        parse_quote! { |#default_closure_pattern| #condition };
                    Ok(closure)
                }
            })
            .collect::<Result<Vec<ExprClosure>>>()?;

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
            items: items.into_iter().map(|x| x).collect(),
        })
    }
}

/// An intermediate enum to help parse either a condition or a `binds` setting.
pub enum ContractArg {
    Requires { expr: Expr },
    Ensures { expr: Expr },
    Maintains { expr: Expr },
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
            ContractArg::Requires { expr } => expr.span(),
            ContractArg::Ensures { expr } => expr.span(),
            ContractArg::Maintains { expr } => expr.span(),
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
            input.parse::<Token![:]>()?;
            Ok(ContractArg::Requires {
                expr: input.parse()?,
            })
        } else if lookahead.peek(kw::maintains) {
            // Parse `maintains: <expr>`
            input.parse::<kw::maintains>()?;
            input.parse::<Token![:]>()?;
            Ok(ContractArg::Maintains {
                expr: input.parse()?,
            })
        } else if lookahead.peek(kw::ensures) {
            // Parse `ensures: <expr>`
            input.parse::<kw::ensures>()?;
            input.parse::<Token![:]>()?;
            Ok(ContractArg::Ensures {
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
        .map(|predicate| {
            let msg = format!("Precondition failed: {}", predicate.to_token_stream());
            quote! { assert!(#predicate, #msg); }
        })
        .chain(contract.maintains.iter().map(|predicate| {
            let msg = format!("Pre-invariant failed: {}", predicate.to_token_stream());
            quote! { assert!(#predicate, #msg); }
        }));

    // --- Generate Postcondition Checks ---
    let postconditions = contract
        .maintains
        .iter()
        .map(|predicate| {
            let msg = format!("Post-invariant failed: {}", predicate.to_token_stream());
            quote! { assert!(#predicate, #msg); }
        })
        .chain(contract.ensures.iter().map(|closure| {
            let msg = format!("Postcondition failed: {}", closure.to_token_stream());
            quote! { assert!((#closure)(#binding_ident), #msg); }
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
mod tests {
    use super::*;
    use syn::parse2;

    #[test]
    fn test_parse_simple_contract() {
        let tokens = quote! {
            requires: x > 0,
            ensures: output > x
        };
        let args: ContractArgs = parse2(tokens.into()).unwrap();
        let contract = Contract::try_from(args).unwrap();

        assert_eq!(contract.requires.len(), 1);
        assert_eq!(contract.maintains.len(), 0);
        assert_eq!(contract.ensures.len(), 1);
    }

    #[test]
    fn test_parse_all_clauses() {
        let tokens = quote! {
            requires: x > 0,
            maintains: y.is_valid(),
            binds: z,
            ensures: z > x
        };
        let args: ContractArgs = parse2(tokens.into()).unwrap();
        let contract = Contract::try_from(args).unwrap();

        assert_eq!(contract.requires.len(), 1);
        assert_eq!(contract.maintains.len(), 1);
        assert_eq!(contract.ensures.len(), 1);
    }

    #[test]
    fn test_parse_out_of_order() {
        let tokens = quote! {
            ensures: output > x,
            requires: x > 0
        };
        let args: ContractArgs = parse2(tokens.into()).unwrap();
        let result = Contract::try_from(args);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_multiple_binds() {
        let tokens = quote! {
            binds: y,
            binds: z
        };
        let args: ContractArgs = parse2(tokens.into()).unwrap();
        let result = Contract::try_from(args);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_array_of_conditions() {
        let tokens = quote! {
            requires: [x > 0, y > 0],
            ensures: [output > x, output > y]
        };
        let args: ContractArgs = parse2(tokens.into()).unwrap();
        let contract = Contract::try_from(args).unwrap();

        assert_eq!(contract.requires.len(), 2);
        assert_eq!(contract.maintains.len(), 0);
        assert_eq!(contract.ensures.len(), 2);
    }
}