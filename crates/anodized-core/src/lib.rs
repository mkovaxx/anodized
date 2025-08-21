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

impl TryFrom<ContractArgs> for Contract {
    type Error = syn::Error;

    fn try_from(args: ContractArgs) -> Result<Self> {
        let mut binds_pattern: Option<&Pat> = None;
        let mut requires: Vec<Expr> = vec![];
        let mut maintains: Vec<Expr> = vec![];
        let mut ensures: Vec<ExprClosure> = vec![];

        for arg in &args.items {
            if let ContractArg::Binds { pattern } = arg {
                if binds_pattern.is_some() {
                    return Err(syn::Error::new(
                        pattern.span(),
                        "duplicate `binds` parameter",
                    ));
                }
                binds_pattern = Some(pattern);
            }
        }

        // The default pattern for `ensures` conditions.
        let default_closure_pattern = binds_pattern
            .map(|p| p.to_token_stream())
            .unwrap_or_else(|| quote! { output });

        for arg in args.items {
            match arg {
                ContractArg::Requires { expr: predicate } => requires.push(predicate),
                ContractArg::Maintains { expr: predicate } => maintains.push(predicate),
                ContractArg::Ensures { expr: predicate } => {
                    // Convert a simple expression into a closure.
                    let closure: ExprClosure =
                        syn::parse_quote! { |#default_closure_pattern| #predicate };
                    ensures.push(closure);
                }
                ContractArg::EnsuresClosure { closure } => ensures.push(closure),
                ContractArg::Binds { .. } => {}
            }
        }

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
    EnsuresClosure { closure: ExprClosure },
    Maintains { expr: Expr },
    Binds { pattern: Pat },
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
            let predicate: Expr = input.parse()?;
            if let Expr::Closure(closure) = predicate {
                Ok(ContractArg::EnsuresClosure { closure })
            } else {
                Ok(ContractArg::Ensures { expr: predicate })
            }
        } else {
            Err(lookahead.error())
        }
    }
}

// Custom keywords for parsing. This allows us to use `requires`, `ensures`, etc.,
// as if they were built-in Rust keywords during parsing.
mod kw {
    syn::custom_keyword!(binds);
    syn::custom_keyword!(requires);
    syn::custom_keyword!(maintains);
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
