use proc_macro::TokenStream;
use proc_macro2::{Ident, Span};
use quote::{ToTokens, quote};
use syn::{
    Expr, ExprClosure, ItemFn, Result, ReturnType, Token,
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
    // Optional global rename for the return value.
    returns_ident: Option<Ident>,
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
        let mut returns_ident = None;

        // The arguments are a comma-separated list of conditions or a `returns` key.
        let items = Punctuated::<ContractArgItem, Token![,]>::parse_terminated(input)?;

        for item in items {
            match item {
                ContractArgItem::Condition(condition) => conditions.push(condition),
                ContractArgItem::Returns(ident) => {
                    if returns_ident.is_some() {
                        return Err(syn::Error::new(ident.span(), "duplicate `returns` key"));
                    }
                    returns_ident = Some(ident);
                }
            }
        }

        Ok(ContractArgs {
            conditions,
            returns_ident,
        })
    }
}

/// An intermediate enum to help parse either a condition or a `returns` key.
enum ContractArgItem {
    Condition(Condition),
    Returns(Ident),
}

impl Parse for ContractArgItem {
    fn parse(input: ParseStream) -> Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(kw::returns) {
            // Parse `returns: new_name`
            input.parse::<kw::returns>()?;
            input.parse::<Token![:]>()?;
            let ident = input.parse::<Ident>()?;
            Ok(ContractArgItem::Returns(ident))
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

    // Determine the name of the variable that will hold the function's return value.
    // This is either the global `returns:` override or the default "output".
    let global_output_ident = args
        .returns_ident
        .clone()
        .unwrap_or_else(|| Ident::new("output", Span::call_site()));

    // --- Generate Precondition Checks ---
    let preconditions = args.conditions.iter().filter_map(|c| match c {
        Condition::Requires { predicate } | Condition::Maintains { predicate } => {
            let msg = format!("Precondition failed: {}", predicate.to_token_stream());
            Some(quote! { assert!(#predicate, #msg); })
        }
        _ => None,
    });

    // --- Generate Postcondition Checks ---
    let returns_nothing = match &func.sig.output {
        ReturnType::Default => true,
        ReturnType::Type(_, ty) => {
            if let syn::Type::Tuple(tuple) = &**ty {
                tuple.elems.is_empty()
            } else {
                false
            }
        }
    };

    let postconditions = args.conditions.iter().filter_map(|c| match c {
        Condition::Maintains { predicate } => {
            let msg = format!("Postcondition failed: {}", predicate.to_token_stream());
            Some(quote! { assert!(#predicate, #msg); })
        }
        Condition::Ensures { predicate } => {
            if returns_nothing {
                return None;
            }
            let msg = format!("Postcondition failed: {}", predicate.to_token_stream());
            Some(quote! { assert!((|#global_output_ident| #predicate)(#global_output_ident), #msg); })
        }
        Condition::EnsuresClosure { closure } => {
            if returns_nothing {
                return None;
            }
            let msg = format!("Postcondition failed: {}", closure.to_token_stream());
            Some(quote! { assert!((#closure)(#global_output_ident), #msg); })
        }
        _ => None, // Ignore `requires`
    });

    // --- Construct the New Body ---

    if returns_nothing {
        // Case 1: Function returns `()` or nothing.
        if is_async {
            Ok(quote! {
                {
                    #(#preconditions)*
                    let result = async { #original_body }.await;
                    #(#postconditions)*
                    result
                }
            })
        } else {
            Ok(quote! {
                {
                    #(#preconditions)*
                    #original_body
                    #(#postconditions)*
                }
            })
        }
    } else {
        // Case 2: Function returns a value.
        if is_async {
            Ok(quote! {
                {
                    #(#preconditions)*
                    let #global_output_ident = async { #original_body }.await;
                    #(#postconditions)*
                    #global_output_ident
                }
            })
        } else {
            Ok(quote! {
                {
                    #(#preconditions)*
                    let #global_output_ident = #original_body;
                    #(#postconditions)*
                    #global_output_ident
                }
            })
        }
    }
}
