#![doc = include_str!("../README.md")]

use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::{ToTokens, quote};
use syn::{
    Expr, ExprClosure, Ident, ItemFn, Pat, Token,
    parse::{Parse, ParseStream, Result},
    punctuated::Punctuated,
    spanned::Spanned,
};

/// A contract specifies the intended behavior of a function or method.
#[derive(Debug, Default)]
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
        let mut requires: Vec<Expr> = vec![];
        let mut maintains: Vec<Expr> = vec![];
        let mut ensures: Vec<ExprClosure> = vec![];

        // The default pattern for `ensures` conditions. It must be resolvable at the call site.
        let default_output_pat = args
            .binds_pat
            .clone()
            .map(|p| p.to_token_stream())
            .unwrap_or_else(|| quote! { output });

        for condition in args.conditions {
            match condition {
                Condition::Requires { predicate } => requires.push(predicate),
                Condition::Maintains { predicate } => maintains.push(predicate),
                Condition::Ensures { predicate } => {
                    // Convert a simple expression into a closure.
                    let closure: ExprClosure =
                        syn::parse_quote! { |#default_output_pat| #predicate };
                    ensures.push(closure);
                }
                Condition::EnsuresClosure { closure } => ensures.push(closure),
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
    pub conditions: Vec<Condition>,
    pub binds_pat: Option<Pat>,
}

/// Represents a single contract condition, e.g., `requires: x > 0`.
pub enum Condition {
    Requires { predicate: Expr },
    Ensures { predicate: Expr },
    EnsuresClosure { closure: ExprClosure },
    Maintains { predicate: Expr },
}

impl Parse for ContractArgs {
    /// Custom parser for the contents of `#[contract(...)]`.
    fn parse(input: ParseStream) -> Result<Self> {
        let mut conditions = Vec::new();
        let mut binds_pat = None;

        // The arguments are a comma-separated list of conditions or a `binds` setting.
        let items = Punctuated::<ContractArgItem, Token![,]>::parse_terminated(input)?;

        for item in items {
            match item {
                ContractArgItem::Condition(condition) => conditions.push(condition),
                ContractArgItem::Binds(pat) => {
                    if binds_pat.is_some() {
                        return Err(syn::Error::new(pat.span(), "duplicate `binds` setting"));
                    }
                    binds_pat = Some(pat);
                }
            }
        }

        Ok(ContractArgs {
            conditions,
            binds_pat,
        })
    }
}

/// An intermediate enum to help parse either a condition or a `binds` setting.
pub enum ContractArgItem {
    Condition(Condition),
    Binds(Pat),
}

impl Parse for ContractArgItem {
    fn parse(input: ParseStream) -> Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(kw::binds) {
            // Parse `binds: pat`
            input.parse::<kw::binds>()?;
            input.parse::<Token![:]>()?;
            let pat = Pat::parse_single(input)?;
            Ok(ContractArgItem::Binds(pat))
        } else if lookahead.peek(kw::requires)
            || lookahead.peek(kw::ensures)
            || lookahead.peek(kw::maintains)
        {
            // Parse a condition like `requires: predicate` or `ensures: |val| predicate`
            Ok(ContractArgItem::Condition(input.parse()?))
        } else {
            Err(lookahead.error())
        }
    }
}

impl Parse for Condition {
    /// Parses a single condition.
    fn parse(input: ParseStream) -> Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(kw::requires) {
            input.parse::<kw::requires>()?;
            input.parse::<Token![:]>()?;
            Ok(Condition::Requires {
                predicate: input.parse()?,
            })
        } else if lookahead.peek(kw::ensures) {
            input.parse::<kw::ensures>()?;
            input.parse::<Token![:]>()?;
            let predicate: Expr = input.parse()?;
            if let Expr::Closure(closure) = predicate {
                Ok(Condition::EnsuresClosure { closure })
            } else {
                Ok(Condition::Ensures { predicate })
            }
        } else if lookahead.peek(kw::maintains) {
            input.parse::<kw::maintains>()?;
            input.parse::<Token![:]>()?;
            Ok(Condition::Maintains {
                predicate: input.parse()?,
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
    syn::custom_keyword!(ensures);
    syn::custom_keyword!(maintains);
    syn::custom_keyword!(binds);
}

/// Takes the original function and contract, and returns a new
/// token stream for the instrumented function body.
pub fn instrument_body(func: &ItemFn, contract: &Contract) -> Result<TokenStream2> {
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

    Ok(quote! {
        {
            #(#preconditions)*
            let #binding_ident = #body_expr;
            #(#postconditions)*
            #binding_ident
        }
    })
}
