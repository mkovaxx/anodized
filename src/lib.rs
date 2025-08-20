use proc_macro::TokenStream;
use proc_macro2::{Ident, Span};
use quote::{ToTokens, quote};
use syn::spanned::Spanned;
use syn::{
    Expr, ExprClosure, ItemFn, Pat, Result, Token,
    parse::{Parse, ParseStream},
    parse_macro_input,
    punctuated::Punctuated,
};

/// The main procedural macro for defining contracts on functions.
///
/// This macro parses contract annotations and injects `assert!` statements
/// into the function body to perform runtime checks in debug builds.
#[proc_macro_attribute]
pub fn contract(args: TokenStream, input: TokenStream) -> TokenStream {
    // Parse the contract arguments from the attribute, e.g., `requires: x > 0, ...`
    let contract_args = parse_macro_input!(args as ContractArgs);
    // Parse the function to which the attribute is attached.
    let mut func = parse_macro_input!(input as ItemFn);

    // Generate the new, instrumented function body.
    let new_body = match instrument_body(&func, &contract_args) {
        Ok(body) => body,
        Err(e) => return e.to_compile_error().into(),
    };

    // Replace the old function body with the new one.
    *func.block = syn::parse2(new_body).expect("Failed to parse new function body");

    // Return the modified function.
    func.into_token_stream().into()
}

/// A container for all parsed arguments from the `#[contract]` attribute.
struct ContractArgs {
    conditions: Vec<Condition>,
    returns_pat: Option<Pat>,
}

/// Represents a single contract condition, e.g., `requires: x > 0`.
enum Condition {
    Requires { predicate: Expr },
    Ensures { predicate: Expr },
    EnsuresClosure { closure: ExprClosure },
    Maintains { predicate: Expr },
}

impl Parse for ContractArgs {
    /// Custom parser for the contents of `#[contract(...)]`.
    fn parse(input: ParseStream) -> Result<Self> {
        let mut conditions = Vec::new();
        let mut returns_pat = None;

        // The arguments are a comma-separated list of conditions or a `returns` key.
        let items = Punctuated::<ContractArgItem, Token![,]>::parse_terminated(input)?;

        for item in items {
            match item {
                ContractArgItem::Condition(condition) => conditions.push(condition),
                ContractArgItem::Returns(pat) => {
                    if returns_pat.is_some() {
                        return Err(syn::Error::new(pat.span(), "duplicate `returns` key"));
                    }
                    returns_pat = Some(pat);
                }
            }
        }

        Ok(ContractArgs {
            conditions,
            returns_pat,
        })
    }
}

/// An intermediate enum to help parse either a condition or a `returns` key.
enum ContractArgItem {
    Condition(Condition),
    Returns(Pat),
}

impl Parse for ContractArgItem {
    fn parse(input: ParseStream) -> Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(kw::returns) {
            // Parse `returns: pat`
            input.parse::<kw::returns>()?;
            input.parse::<Token![:]>()?;
            let pat = Pat::parse_single(input)?;
            Ok(ContractArgItem::Returns(pat))
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
    syn::custom_keyword!(returns);
}

/// Takes the original function and contract arguments, and returns a new
/// token stream for the instrumented function body.
fn instrument_body(func: &ItemFn, args: &ContractArgs) -> Result<proc_macro2::TokenStream> {
    let original_body = &func.block;
    let is_async = func.sig.asyncness.is_some();

    // The identifier for the return value binding. It's hygienic to prevent collisions.
    let binding_ident = Ident::new("__anodized_output", Span::mixed_site());

    // The pattern for the `ensures` predicate. It must be resolvable at the call site.
    let default_output_pat = args
        .returns_pat
        .clone()
        .map(|p| p.to_token_stream())
        .unwrap_or_else(|| quote! { output });

    // --- Generate Precondition Checks ---
    let preconditions = args.conditions.iter().filter_map(|c| match c {
        Condition::Requires { predicate } => {
            let msg = format!("Precondition failed: {}", predicate.to_token_stream());
            Some(quote! { assert!(#predicate, #msg); })
        }
        Condition::Maintains { predicate } => {
            let msg = format!("Pre-invariant failed: {}", predicate.to_token_stream());
            Some(quote! { assert!(#predicate, #msg); })
        }
        _ => None,
    });

    // --- Generate Postcondition Checks ---
    let postconditions = args.conditions.iter().filter_map(|c| match c {
        Condition::Maintains { predicate } => {
            let msg = format!("Post-invariant failed: {}", predicate.to_token_stream());
            Some(quote! { assert!(#predicate, #msg); })
        }
        Condition::Ensures { predicate } => {
            let msg = format!("Postcondition failed: {}", predicate.to_token_stream());
            Some(quote! { assert!((|#default_output_pat| #predicate)(#binding_ident), #msg); })
        }
        Condition::EnsuresClosure { closure } => {
            let msg = format!("Postcondition failed: {}", closure.to_token_stream());
            Some(quote! { assert!((#closure)(#binding_ident), #msg); })
        }
        _ => None, // Ignore `requires`
    });

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
