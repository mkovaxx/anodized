#![doc = include_str!("../README.md")]
use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{
    parse_macro_input,
    ItemFn,
    spanned::Spanned,
};
use proc_macro2::Span;
use syn::Ident;

use crate::syntax::ContractArgs;

mod syntax;

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

    let contract = match crate::syntax::Contract::try_from(contract_args) {
        Ok(contract) => contract,
        Err(e) => return e.to_compile_error().into(),
    };

    // Generate the new, instrumented function body.
    let new_body = match instrument_body(&func, &contract) {
        Ok(body) => body,
        Err(e) => return e.to_compile_error().into(),
    };

    // Replace the old function body with the new one.
    *func.block = syn::parse2(new_body).expect("Failed to parse new function body");

    // Return the modified function.
    func.into_token_stream().into()
}

/// Takes the original function and contract, and returns a new
/// token stream for the instrumented function body.
fn instrument_body(
    func: &ItemFn,
    contract: &crate::syntax::Contract,
) -> Result<TokenStream2, syn::Error> {
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
