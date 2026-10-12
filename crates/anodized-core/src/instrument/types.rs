#[cfg(test)]
#[path = "types_tests.rs"]
mod types_tests;

use proc_macro2::{Span, TokenStream};
use quote::ToTokens;
use syn::{
    Arm, Expr, Fields, Ident, ItemEnum, ItemImpl, ItemStruct, Pat, Result, Stmt, parse_quote,
};

use crate::{
    Condition, TypeSpec,
    instrument::{Mode, build_cond_eval},
};

impl Mode {
    pub fn instrument_item_struct(
        &self,
        spec: TypeSpec,
        item_struct: ItemStruct,
    ) -> Result<TokenStream> {
        let mut tokens = TokenStream::new();

        item_struct.to_tokens(&mut tokens);

        if self.changes_anything() {
            let inductive_predicate = Self::build_enum_inductive_predicate(
                std::iter::once((&item_struct.ident, &item_struct.fields)),
                &spec.field_spec_flags,
            );
            let statements = Self::build_refinement_stmts(&inductive_predicate, &spec.maintains);

            let ident = &item_struct.ident;
            let (impl_generics, ty_generics, where_clause) = item_struct.generics.split_for_impl();

            let spec_impl: ItemImpl = parse_quote! {
                #[doc(hidden)]
                #[allow(warnings)]
                impl #impl_generics ::anodized::types::Spec for #ident #ty_generics #where_clause {
                    fn predicate(&self) -> bool {
                        #(#statements)*
                        __anodized_inv
                    }
                }
            };
            spec_impl.to_tokens(&mut tokens);
        }

        Ok(tokens)
    }

    pub fn instrument_item_enum(&self, spec: TypeSpec, item_enum: ItemEnum) -> Result<TokenStream> {
        let mut tokens = TokenStream::new();

        item_enum.to_tokens(&mut tokens);

        if self.changes_anything() {
            let inductive_predicate = Self::build_enum_inductive_predicate(
                item_enum
                    .variants
                    .iter()
                    .map(|variant| (&variant.ident, &variant.fields)),
                &spec.field_spec_flags,
            );
            let statements = Self::build_refinement_stmts(&inductive_predicate, &spec.maintains);

            let ident = &item_enum.ident;
            let (impl_generics, ty_generics, where_clause) = item_enum.generics.split_for_impl();

            let spec_impl: ItemImpl = parse_quote! {
                #[doc(hidden)]
                #[allow(warnings)]
                impl #impl_generics ::anodized::types::Spec for #ident #ty_generics #where_clause {
                    fn predicate(&self) -> bool {
                        // Bring all variants into scope for convenience.
                        use #ident::*;
                        #(#statements)*
                        __anodized_inv
                    }
                }
            };
            spec_impl.to_tokens(&mut tokens);
        }

        Ok(tokens)
    }

    fn build_refinement_stmts(inductive_predicate: &Expr, maintains: &[Condition]) -> Vec<Stmt> {
        let mut statements = vec![parse_quote! {
            let __anodized_inv = #inductive_predicate;
        }];

        for invariant in maintains {
            let eval = build_cond_eval(&invariant.expr);
            let check = parse_quote! {
                let __anodized_inv = __anodized_inv & #eval;
            };
            statements.push(check);
        }

        statements
    }

    fn build_enum_inductive_predicate<'a>(
        variants: impl Iterator<Item = (&'a Ident, &'a Fields)>,
        variant_spec_flags: &[Vec<bool>],
    ) -> Expr {
        let arms = variants
            .zip(variant_spec_flags)
            .map(|((ident, fields), field_spec_flags)| {
                Self::build_variant_inductive_predicate(ident, fields, field_spec_flags)
            });

        parse_quote! {
            match self {
                #(#arms),*
            }
        }
    }

    fn build_variant_inductive_predicate(
        ident: &Ident,
        fields: &Fields,
        field_spec_flags: &[bool],
    ) -> Arm {
        let mut field_predicate_calls: Vec<Expr> = vec![];

        let variant_pattern: Pat = match fields {
            Fields::Named(fields) => {
                let field_names = fields.named.iter().zip(field_spec_flags).flat_map(
                    |(field, spec_flag)| -> Option<&Ident> {
                        let Some(field_name) = &field.ident else {
                            unreachable!("named field with no ident");
                        };
                        if *spec_flag {
                            field_predicate_calls.push(parse_quote! {
                                ::anodized::__::eval_type_spec(#field_name)
                            });
                            Some(field_name)
                        } else {
                            None
                        }
                    },
                );

                if field_spec_flags.iter().all(|flag| *flag) {
                    parse_quote! {
                        #ident { #(#field_names),* }
                    }
                } else {
                    parse_quote! {
                        #ident { #(#field_names,)* .. }
                    }
                }
            }
            Fields::Unnamed(fields) => {
                let field_patterns = fields.unnamed.iter().zip(field_spec_flags).enumerate().map(
                    |(index, (_, spec_flag))| -> Pat {
                        if *spec_flag {
                            let field_name =
                                Ident::new(&format!("field_{index}"), Span::call_site());
                            field_predicate_calls.push(parse_quote! {
                                ::anodized::__::eval_type_spec(#field_name)
                            });
                            parse_quote! { #field_name }
                        } else {
                            parse_quote! { _ }
                        }
                    },
                );

                parse_quote! {
                    #ident(#(#field_patterns),*)
                }
            }
            Fields::Unit => parse_quote! { #ident },
        };

        match field_predicate_calls.as_slice() {
            [] => parse_quote! {
                #variant_pattern => true
            },
            [single] => parse_quote! {
                #variant_pattern => #single
            },
            many => parse_quote! {
                #variant_pattern => {
                    #(#many)&*
                }
            },
        }
    }
}
