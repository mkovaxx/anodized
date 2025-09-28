use proc_macro2::TokenStream;
use quote::quote;
use syn::Meta;
pub mod function;

pub struct Backend {
    pub build_check: fn(Option<&Meta>, &TokenStream, &str, &TokenStream) -> TokenStream,
}

impl Backend {
    pub const CHECK_AND_PANIC: Backend = Backend {
        build_check: build_assert,
    };

    pub const CHECK_AND_PRINT: Backend = Backend {
        build_check: build_eprint,
    };

    pub const NO_CHECK: Backend = Backend {
        build_check: build_inert,
    };
}

fn build_assert(
    cfg: Option<&Meta>,
    expr: &TokenStream,
    message: &str,
    repr: &TokenStream,
) -> TokenStream {
    let repr_str = repr.to_string();
    let check = quote! { assert!(#expr, #message, #repr_str); };
    guard_check(cfg, check)
}

fn build_eprint(
    cfg: Option<&Meta>,
    expr: &TokenStream,
    message: &str,
    repr: &TokenStream,
) -> TokenStream {
    let repr_str = repr.to_string();
    let check = quote! {
        if !(#expr) {
            eprintln!(#message, #repr_str);
        }
    };
    guard_check(cfg, check)
}

fn build_inert(
    // The check will not be present at runtime regardless of the `#[cfg]` setting.
    _cfg: Option<&Meta>,
    expr: &TokenStream,
    message: &str,
    repr: &TokenStream,
) -> TokenStream {
    let repr_str = repr.to_string();
    quote! {
        if false {
            assert!(#expr, #message, #repr_str);
        }
    }
}

fn guard_check(cfg: Option<&Meta>, check: TokenStream) -> TokenStream {
    if let Some(cfg) = cfg {
        quote! { if cfg!(#cfg) { #check } }
    } else {
        check
    }
}
