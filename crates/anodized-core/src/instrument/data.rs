#[cfg(test)]
#[path = "data_tests.rs"]
mod data_tests;

use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::{ItemEnum, ItemImpl, ItemStruct, Result, parse_quote};

use crate::{DataSpec, instrument::Mode};

impl Mode {
    pub fn instrument_item_struct(
        &self,
        spec: DataSpec,
        item_struct: ItemStruct,
    ) -> Result<TokenStream> {
        let mut tokens = TokenStream::new();

        item_struct.to_tokens(&mut tokens);

        if self.changes_anything() {
            let ident = &item_struct.ident;
            let (impl_generics, ty_generics, where_clause) = item_struct.generics.split_for_impl();
            let statements = Self::build_precondition_fn_body(&[], &spec.maintains).stmts;

            let spec_impl: ItemImpl = parse_quote! {
                #[doc(hidden)]
                #[allow(warnings)]
                impl #impl_generics ::anodized::types::Refine for #ident #ty_generics
                    #where_clause
                {
                    fn predicate(&self) -> bool {
                        #(#statements)*
                    }
                }
            };
            spec_impl.to_tokens(&mut tokens);
        }

        Ok(tokens)
    }

    pub fn instrument_item_enum(&self, spec: DataSpec, item_enum: ItemEnum) -> Result<TokenStream> {
        let mut tokens = TokenStream::new();

        let ident = &item_enum.ident;
        let (impl_generics, ty_generics, where_clause) = item_enum.generics.split_for_impl();
        let statements = Self::build_precondition_fn_body(&[], &spec.maintains).stmts;

        item_enum.to_tokens(&mut tokens);

        if let Mode::EmbedSpecs = self {
            let spec_impl: ItemImpl = parse_quote! {
                #[doc(hidden)]
                #[allow(warnings)]
                impl #impl_generics ::anodized::types::Refine for #ident #ty_generics
                    #where_clause
                {
                    fn predicate(&self) -> bool {
                        // Bring all variants into scope for convenience.
                        use #ident::*;
                        #(#statements)*
                    }
                }
            };
            spec_impl.to_tokens(&mut tokens);
        }

        Ok(tokens)
    }
}
