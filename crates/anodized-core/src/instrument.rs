use proc_macro2::TokenStream;
use quote::{ToTokens, quote};
use syn::{
    Attribute, Block, FnArg, Ident, ItemConst, ItemFn, ItemImpl, ItemTrait, Result, ReturnType,
    Signature, parse_quote,
};

use crate::{DataSpec, Spec};

pub mod data;
pub mod fns;
pub mod impls;
pub mod loops;
pub mod patterns;
pub mod traits;

#[derive(Debug, Clone)]
pub enum Mode {
    /// Make no changes to the code.
    ChangeNothing,
    /// Inject code to enable compile-time and/or runtime checks.
    InjectChecks(CheckSettings),
    /// Embed spec elements as new items without changing existing code.
    EmbedSpecs,
}

#[derive(Debug, Clone)]
pub struct CheckSettings {
    /// Print errors about violated clauses.
    pub does_print: bool,
    /// Panic on a violated pre/postcondition or invariant.
    pub does_panic: Option<PanicSettings>,
}

#[derive(Debug, Clone)]
pub struct PanicSettings {
    /// Generate an entry point that defers the panic. Used for fuzzing, PBT, etc.
    pub has_try_fn: bool,
}

impl Mode {
    pub fn changes_anything(&self) -> bool {
        !matches!(self, Mode::ChangeNothing)
    }

    pub fn emits_try_fn(&self) -> bool {
        if let Self::InjectChecks(check_settings) = self
            && let Some(panic_settings) = &check_settings.does_panic
        {
            panic_settings.has_try_fn
        } else {
            false
        }
    }

    pub fn with_try_fn(&self, value: bool) -> Self {
        match self {
            Mode::ChangeNothing => Mode::ChangeNothing,
            Mode::InjectChecks(check_settings) => {
                let mut check_settings = check_settings.clone();
                if let Some(panic_settings) = &mut check_settings.does_panic {
                    panic_settings.has_try_fn = value;
                };
                Mode::InjectChecks(check_settings)
            }
            Mode::EmbedSpecs => Mode::EmbedSpecs,
        }
    }

    pub fn instrument_item_fn(&self, spec: Spec, mut item_fn: ItemFn) -> Result<TokenStream> {
        let mut tokens = TokenStream::new();

        if item_fn.sig.ident.to_string().starts_with("__anodized_") {
            return Err(syn::Error::new_spanned(
                item_fn.sig.ident,
                r#"An item with the `__anodized_` prefix is internal. Do not implement it directly.
Instead, you likely need to place a `#[spec]` attribute on an enclosing trait or impl block."#,
            ));
        }

        if let Self::EmbedSpecs = self {
            // Embed `spec` elements as `__anodized_fn_*` items.
            let attrs: [Attribute; 2] = [
                parse_quote!(#[doc(hidden)]),
                parse_quote!(#[allow(warnings)]),
            ];

            let spec_qualifiers_const: ItemConst = Self::build_qualifier_const_item(
                &attrs,
                "__anodized_fn_qualifiers",
                spec.qualifiers,
                &item_fn.sig.ident,
            );
            let spec_requires_fn = ItemFn {
                attrs: attrs.to_vec(),
                vis: syn::Visibility::Inherited,
                sig: Self::build_precondition_fn_sig("__anodized_fn_requires", &item_fn.sig),
                block: Box::new(Self::build_precondition_fn_body(
                    &spec.requires,
                    &spec.maintains,
                )),
            };
            let spec_ensures_fn = ItemFn {
                attrs: attrs.to_vec(),
                vis: syn::Visibility::Inherited,
                sig: Self::build_postcondition_fn_sig("__anodized_fn_ensures", &item_fn.sig),
                block: Box::new(Self::build_postcondition_fn_body(
                    &spec.maintains,
                    &spec.captures,
                    &spec.ensures,
                )?),
            };

            spec_qualifiers_const.to_tokens(&mut tokens);
            spec_requires_fn.to_tokens(&mut tokens);
            spec_ensures_fn.to_tokens(&mut tokens);
        }

        // Instrument function body.
        self.instrument_fn(&spec, &item_fn.sig, &mut item_fn.block)?;

        if let Self::InjectChecks(check_settings) = self
            && let Some(ref panic_settings) = check_settings.does_panic
            && panic_settings.has_try_fn
        {
            // Build a wrapper that forwards to the "try_fn" entry point.
            let mut wrapper_fn = item_fn.clone();
            let mangled_ident =
                Self::build_try_fn_wrapper(false, &mut wrapper_fn.sig, wrapper_fn.block.as_mut());
            wrapper_fn.to_tokens(&mut tokens);

            // Create the "try_fn" entry point for e.g. fuzzing and PBT.
            item_fn.sig.ident = mangled_ident;
            item_fn.sig.output = match item_fn.sig.output {
                ReturnType::Default => parse_quote!(-> ::anodized::result::Result<()>),
                ReturnType::Type(ra, ty) => {
                    parse_quote!(#ra ::anodized::result::Result<#ty>)
                }
            };
            item_fn.attrs = vec![parse_quote!(#[doc(hidden)]), parse_quote!(#[inline])];
        }

        item_fn.to_tokens(&mut tokens);
        Ok(tokens)
    }

    fn build_try_fn_wrapper(is_impl: bool, sig: &mut Signature, body: &mut Block) -> Ident {
        let mangled_ident = fns::make_try_fn_ident(&sig.ident);

        Self::build_wrapper_fn_signature(sig);

        let args = sig.inputs.iter().map(|arg| match arg {
            FnArg::Receiver(receiver) => receiver.self_token.to_token_stream(),
            FnArg::Typed(pat_type) => pat_type.pat.to_token_stream(),
        });

        let maybe_self = match is_impl {
            true => quote!(Self::),
            false => quote!(),
        };

        let maybe_await = match &sig.asyncness {
            Some(_) => quote!(.await),
            None => quote!(),
        };

        *body = parse_quote! {
            {
                match #maybe_self #mangled_ident(#(#args),*) #maybe_await {
                    ::anodized::result::Result::Ok(output) => output,
                    ::anodized::result::Result::Err(
                        ::anodized::result::Error::Pre
                    ) => panic!("precondition failed"),
                    ::anodized::result::Result::Err(
                        ::anodized::result::Error::Post(_)
                    ) => panic!("postcondition failed"),
                }
            }
        };

        mangled_ident
    }

    fn build_wrapper_fn_signature(sig: &mut Signature) {
        use syn::spanned::Spanned;
        for (i, arg) in sig.inputs.iter_mut().enumerate() {
            match arg {
                FnArg::Receiver(_) => {}
                FnArg::Typed(pat_type) => {
                    let name = Ident::new(&format!("input_{i}"), pat_type.span());
                    pat_type.pat = parse_quote!(#name);
                }
            }
        }
    }

    pub fn instrument_item_impl(&self, spec: DataSpec, item_impl: ItemImpl) -> Result<TokenStream> {
        let new_impl = self.instrument_impl(spec, item_impl)?;
        Ok(new_impl.to_token_stream())
    }

    pub fn instrument_item_trait(
        &self,
        spec: DataSpec,
        item_trait: ItemTrait,
    ) -> Result<TokenStream> {
        let new_trait = self.instrument_trait(spec, item_trait)?;
        Ok(new_trait.to_token_stream())
    }

    pub fn instrument_item_trait_impl(
        &self,
        spec: DataSpec,
        item_impl: ItemImpl,
    ) -> Result<TokenStream> {
        let new_trait_impl = self.instrument_trait_impl(spec, item_impl)?;
        Ok(new_trait_impl.to_token_stream())
    }
}

#[cfg(test)]
impl Mode {
    pub(crate) const DEFAULT: Self = Mode::InjectChecks(CheckSettings::DEFAULT);
}

#[cfg(test)]
impl CheckSettings {
    pub(crate) const DEFAULT: Self = Self {
        does_print: false,
        does_panic: None,
    };

    pub(crate) const PRINT: Self = Self {
        does_print: true,
        does_panic: None,
    };

    pub(crate) const PRINT_AND_PANIC: Self = Self {
        does_print: true,
        does_panic: Some(PanicSettings { has_try_fn: false }),
    };

    pub(crate) const PRINT_AND_TRY: Self = Self {
        does_print: true,
        does_panic: Some(PanicSettings { has_try_fn: true }),
    };
}

/// Make an error message to say that some item is unsupported.
pub fn make_item_error<T: ToTokens>(tokens: &T, item_descr: &str) -> syn::Error {
    let msg = format!(
        r#"The #[spec] attribute doesn't yet support this item: {}.
If this is a problem for your use case, please open a feature
request at https://github.com/anodized-rs/anodized/issues/new"#,
        item_descr
    );
    syn::Error::new_spanned(tokens, msg)
}
