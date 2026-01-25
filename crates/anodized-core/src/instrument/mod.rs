use proc_macro2::TokenStream;
use quote::{ToTokens, quote};
use syn::Meta;

pub mod fns;
pub mod traits;

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

pub fn make_item_error<T: ToTokens>(tokens: &T, item_descr: &str) -> syn::Error {
    let msg = format!(
        r#"The #[spec] attribute doesn't yet support this item: {}.
If this is a problem for your use case, please open a feature
request at https://github.com/mkovaxx/anodized/issues/new"#,
        item_descr
    );
    syn::Error::new_spanned(tokens, msg)
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
