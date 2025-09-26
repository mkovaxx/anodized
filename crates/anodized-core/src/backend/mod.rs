use proc_macro2::TokenStream;
use quote::quote;
pub mod function;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct Backend {
    pub disable_runtime_checks: bool,
    pub build_check: fn(&TokenStream, &str, &TokenStream) -> TokenStream,
}

impl Backend {
    pub const DEFAULT: Backend = Backend {
        disable_runtime_checks: false,
        build_check: build_assert,
    };

    pub const NO_CHECKS: Backend = Backend {
        disable_runtime_checks: true,
        build_check: build_assert,
    };
}

fn build_assert(expr: &TokenStream, message: &str, repr: &TokenStream) -> TokenStream {
    let repr_str = repr.to_string();
    quote! { assert!(#expr, #message, #repr_str); }
}
