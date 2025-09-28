use proc_macro2::TokenStream;
use quote::quote;
pub mod function;

pub struct Backend {
    pub disable_runtime_checks: bool,
    pub build_check: fn(&TokenStream, &str, &TokenStream) -> TokenStream,
}

impl Backend {
    pub const CHECK_AND_PANIC: Backend = Backend {
        disable_runtime_checks: false,
        build_check: build_assert,
    };

    pub const CHECK_AND_PRINT: Backend = Backend {
        disable_runtime_checks: false,
        build_check: build_eprint,
    };

    pub const NO_CHECK: Backend = Backend {
        disable_runtime_checks: true,
        build_check: build_assert,
    };
}

fn build_assert(expr: &TokenStream, message: &str, repr: &TokenStream) -> TokenStream {
    let repr_str = repr.to_string();
    quote! { assert!(#expr, #message, #repr_str); }
}

fn build_eprint(expr: &TokenStream, message: &str, repr: &TokenStream) -> TokenStream {
    let repr_str = repr.to_string();
    quote! {
        if !(#expr) {
            eprintln!(#message, #repr_str);
        }
    }
}
