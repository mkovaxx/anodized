use proc_macro::TokenStream;
use proc_macro2::{Ident, Span};
use quote::{ToTokens, quote};
use syn::{
    Expr, ItemFn, Pat, Result, ReturnType, Token,
    parse::{Parse, ParseStream},
    parse_macro_input,
    punctuated::Punctuated,
    spanned::Spanned,
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
    clauses: Vec<Clause>,
    // Optional global rename for the return value.
    returns_ident: Option<Ident>,
}

/// Represents a single clause, e.g., `requires: x > 0`.
struct Clause {
    flavor: ClauseFlavor,
    predicate: Expr,
    // Optional per-clause rename for the return value (only for `ensures`).
    output_binding: Option<Ident>,
}

/// The "flavor" of a clause: precondition, postcondition, or invariant.
#[derive(Clone, Copy, PartialEq, Eq)]
enum ClauseFlavor {
    Requires,
    Ensures,
    Maintains,
}

impl Parse for ContractArgs {
    /// Custom parser for the contents of `#[contract(...)]`.
    fn parse(input: ParseStream) -> Result<Self> {
        let mut clauses = Vec::new();
        let mut returns_ident = None;

        // The arguments are a comma-separated list of clauses or a `returns` key.
        let items = Punctuated::<ContractArgItem, Token![,]>::parse_terminated(input)?;

        for item in items {
            match item {
                ContractArgItem::Clause(clause) => clauses.push(clause),
                ContractArgItem::Returns(ident) => {
                    if returns_ident.is_some() {
                        return Err(syn::Error::new(ident.span(), "duplicate `returns` key"));
                    }
                    returns_ident = Some(ident);
                }
            }
        }

        Ok(ContractArgs {
            clauses,
            returns_ident,
        })
    }
}

/// An intermediate enum to help parse either a clause or a `returns` key.
enum ContractArgItem {
    Clause(Clause),
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
            // Parse a clause like `requires: predicate` or `ensures: |val| predicate`
            Ok(ContractArgItem::Clause(input.parse()?))
        } else {
            Err(lookahead.error())
        }
    }
}

impl Parse for Clause {
    /// Parses a single clause.
    fn parse(input: ParseStream) -> Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(kw::requires) {
            input.parse::<kw::requires>()?;
            input.parse::<Token![:]>()?;
            Ok(Clause {
                flavor: ClauseFlavor::Requires,
                predicate: input.parse()?,
                output_binding: None,
            })
        } else if lookahead.peek(kw::ensures) {
            input.parse::<kw::ensures>()?;
            input.parse::<Token![:]>()?;
            let mut output_binding = None;
            // Check for the optional `|name|` syntax.
            if input.peek(Token![|]) {
                input.parse::<Token![|]>()?;
                // FIX: Use `Pat::parse_single` instead of `input.parse()`.
                let pat = Pat::parse_single(input)?;
                if let Pat::Ident(pat_ident) = pat {
                    output_binding = Some(pat_ident.ident);
                } else {
                    return Err(syn::Error::new(
                        pat.span(),
                        "expected a simple identifier for the return value binding",
                    ));
                }
                input.parse::<Token![|]>()?;
            }
            Ok(Clause {
                flavor: ClauseFlavor::Ensures,
                predicate: input.parse()?,
                output_binding,
            })
        } else if lookahead.peek(kw::maintains) {
            input.parse::<kw::maintains>()?;
            input.parse::<Token![:]>()?;
            Ok(Clause {
                flavor: ClauseFlavor::Maintains,
                predicate: input.parse()?,
                output_binding: None,
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
    let preconditions = args.clauses.iter().filter_map(|c| {
        if c.flavor == ClauseFlavor::Requires || c.flavor == ClauseFlavor::Maintains {
            let pred = &c.predicate;
            let msg = format!("Precondition failed: {}", pred.to_token_stream());
            Some(quote! { assert!(#pred, #msg); })
        } else {
            None
        }
    });

    // --- Generate Postcondition Checks ---
    let postconditions = args.clauses.iter().filter_map(|c| {
        if c.flavor == ClauseFlavor::Ensures || c.flavor == ClauseFlavor::Maintains {
            let pred = &c.predicate;
            let msg = format!("Postcondition failed: {}", pred.to_token_stream());

            // If the clause has a per-clause `|name|` binding, we create a new
            // variable with that name that references the global output variable.
            let maybe_rename = if let Some(per_clause_ident) = &c.output_binding {
                quote! { let #per_clause_ident = &#global_output_ident; }
            } else {
                quote! {}
            };

            Some(quote! {
                // This block ensures that if we rename the output, it's only for this assertion.
                {
                    #maybe_rename
                    assert!(#pred, #msg);
                }
            })
        } else {
            None
        }
    });

    // --- Construct the New Body ---
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
