#![doc = include_str!("../../../README.md")]

use proc_macro::TokenStream;
use quote::ToTokens;
use syn::{ItemFn, parse_macro_input};

use anodized_core::{Contract, instrument_function_body};

/// The main procedural macro for defining contracts on functions.
///
/// This macro parses contract annotations and injects `assert!` statements
/// into the function body to perform runtime checks in debug builds.
#[proc_macro_attribute]
pub fn contract(args: TokenStream, input: TokenStream) -> TokenStream {
    // Parse the contract arguments from the attribute, e.g., `requires: x > 0, ...`
    let contract = parse_macro_input!(args as Contract);
    // Parse the function to which the attribute is attached.
    let mut func = parse_macro_input!(input as ItemFn);
    let is_async = func.sig.asyncness.is_some();

    // Generate the new, instrumented function body.
    let new_body = match instrument_function_body(&contract, &func.block, is_async) {
        Ok(body) => body,
        Err(e) => return e.to_compile_error().into(),
    };

    // Replace the old function body with the new one.
    *func.block = new_body;

    // Return the modified function.
    func.into_token_stream().into()
}
