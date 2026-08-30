use crate::{instrument::CheckSettings, qualifiers::FnQualifiers, test_util::assert_tokens_eq};

use super::*;
use proc_macro2::TokenStream;
use syn::{ItemImpl, ItemTrait, parse_quote};

#[test]
fn embed_spec_item_trait() {
    let trait_spec = DataSpec::empty();
    let item_trait: ItemTrait = parse_quote! {
        trait TRAIT {
            #[spec(
                requires: COND_1,
                maintains: COND_2,
                ensures: |PAT_1| COND_3,
            )]
            fn FUNC(&self, PARAM_1: TYPE_1, PARAM_2: TYPE_2) -> RET_TYPE {
                BODY
            }
        }
    };

    let qualifier_bits = FnQualifiers::empty().bits();
    let expected: TokenStream = parse_quote! {
        trait TRAIT {
            #[doc(hidden)]
            #[allow(warnings)]
            fn __anodized_fn_requires_FUNC(&self, PARAM_1: TYPE_1, PARAM_2: TYPE_2) -> bool {
                let __anodized_clause_1 = ::anodized::__::eval::<bool>(|| COND_1);
                let __anodized_clause_2 = ::anodized::__::eval::<bool>(|| COND_2);
                __anodized_clause_1 && __anodized_clause_2
            }

            #[doc(hidden)]
            #[allow(warnings)]
            fn __anodized_fn_ensures_FUNC(&self, PARAM_1: TYPE_1, PARAM_2: TYPE_2, __anodized_output: RET_TYPE) -> bool {
                let __anodized_clause_1 = ::anodized::__::eval::<bool>(|| COND_2);
                let () = ();
                let __anodized_clause_2 = ::anodized::__::eval::<bool>(|| { let PAT_1 = __anodized_output; COND_3 });
                __anodized_clause_1 && __anodized_clause_2
            }

            #[doc(hidden)]
            #[allow(warnings)]
            const __anodized_fn_qualifiers_trait_FUNC: u32 = #qualifier_bits;

            #[doc(hidden)]
            #[allow(warnings)]
            const __anodized_fn_qualifiers_FUNC: u32 = #qualifier_bits;

            fn FUNC(&self, PARAM_1: TYPE_1, PARAM_2: TYPE_2) -> RET_TYPE {
                BODY
            }
        }
    };

    let observed = Mode::EmbedSpecs
        .instrument_item_trait(trait_spec, item_trait)
        .unwrap();
    assert_tokens_eq(&observed, &expected);
}

#[test]
fn default_instrument_item_trait() {
    let trait_spec = DataSpec::empty();
    let item_trait: ItemTrait = parse_quote! {
        trait TRAIT {
            #[spec(
                requires: COND_1,
                maintains: COND_2,
                ensures: |PAT_1| COND_3,
            )]
            fn FUNC(&self, PARAM_1: TYPE_1, PARAM_2: TYPE_2) -> RET_TYPE {
                BODY
            }
        }
    };

    let qualifier_bits = FnQualifiers::empty().bits();
    let expected: TokenStream = parse_quote! {
        trait TRAIT {
            #[doc(hidden)]
            #[allow(warnings)]
            const __anodized_fn_qualifiers_trait_FUNC: u32 = #qualifier_bits;

            #[doc(hidden)]
            #[allow(warnings)]
            const __anodized_fn_qualifiers_FUNC: u32 = #qualifier_bits;

            #[doc(hidden)]
            fn __anodized_FUNC(&self, PARAM_1: TYPE_1, PARAM_2: TYPE_2) -> RET_TYPE {
                BODY
            }

            fn FUNC(&self, PARAM_1: TYPE_1, PARAM_2: TYPE_2) -> RET_TYPE {
                let __anodized_pre = true;
                let __anodized_pre = __anodized_pre & (true || ::anodized::__::eval::<bool>(|| COND_1));
                let __anodized_pre = __anodized_pre & (true || ::anodized::__::eval::<bool>(|| COND_2));
                if !__anodized_pre {}
                let (__anodized_output) = (::anodized::__::eval_once(|| -> RET_TYPE { Self::__anodized_FUNC(self, PARAM_1, PARAM_2) }));
                let __anodized_post = true;
                let __anodized_post = __anodized_post & (true || ::anodized::__::eval::<bool>(|| COND_2));
                let (__anodized_post, __anodized_output) = ::anodized::__::apply_keep(
                    |PAT_1| (__anodized_post & (true || ::anodized::__::eval::<bool>(|| COND_3)), PAT_1),
                    __anodized_output,
                );
                if !__anodized_post {}
                __anodized_output
            }
        }
    };

    let observed = Mode::DEFAULT
        .instrument_item_trait(trait_spec, item_trait)
        .unwrap();
    assert_tokens_eq(&observed, &expected);
}

#[test]
fn emit_try_fn_instrument_item_trait() {
    let trait_spec = DataSpec::empty();
    let item_trait: ItemTrait = parse_quote! {
        trait TRAIT {
            #[spec(
                requires: COND_1,
                maintains: COND_2,
                ensures: |PAT_1| COND_3,
            )]
            fn FUNC(&self, PARAM_1: TYPE_1, PARAM_2: TYPE_2) -> RET_TYPE {
                BODY
            }
        }
    };

    let qualifier_bits = FnQualifiers::empty().bits();
    let expected: TokenStream = parse_quote! {
        trait TRAIT {
            #[doc(hidden)]
            #[allow(warnings)]
            const __anodized_fn_qualifiers_trait_FUNC: u32 = #qualifier_bits;

            #[doc(hidden)]
            #[allow(warnings)]
            const __anodized_fn_qualifiers_FUNC: u32 = #qualifier_bits;

            #[doc(hidden)]
            fn __anodized_FUNC(&self, PARAM_1: TYPE_1, PARAM_2: TYPE_2) -> RET_TYPE {
                BODY
            }

            fn FUNC(&self, input_1: TYPE_1, input_2: TYPE_2) -> RET_TYPE {
                match Self::__anodized_fn_try_FUNC(self, input_1, input_2) {
                    ::anodized::result::Result::Ok(output) => output,
                    ::anodized::result::Result::Err(
                        ::anodized::result::Error::Pre
                    ) => panic!("precondition failed"),
                    ::anodized::result::Result::Err(
                        ::anodized::result::Error::Post(_)
                    ) => panic!("postcondition failed"),
                }
            }

            #[doc(hidden)]
            #[inline]
            fn __anodized_fn_try_FUNC(&self, PARAM_1: TYPE_1, PARAM_2: TYPE_2)
                -> ::anodized::result::Result<RET_TYPE>
            {
                let __anodized_pre = true;
                let __anodized_pre = __anodized_pre & (::anodized::__::eval::<bool>(|| COND_1)
                    || eprintln!("precondition failed: {}", "COND_1") != ());
                let __anodized_pre = __anodized_pre & (::anodized::__::eval::<bool>(|| COND_2)
                    || eprintln!("preinvariant failed: {}", "COND_2") != ());
                if !__anodized_pre {
                    return ::anodized::result::pre_err();
                }
                let (__anodized_output) = (::anodized::__::eval_once(|| -> RET_TYPE { Self::__anodized_FUNC(self, PARAM_1, PARAM_2) }));
                let __anodized_post = true;
                let __anodized_post = __anodized_post & (::anodized::__::eval::<bool>(|| COND_2)
                        || eprintln!("postinvariant failed: {}", "COND_2") != ());
                let (__anodized_post, __anodized_output) = ::anodized::__::apply_keep(
                    |PAT_1| (__anodized_post & (::anodized::__::eval::<bool>(|| COND_3)
                        || eprintln!("postcondition failed: {}", "COND_3") != ()), PAT_1),
                    __anodized_output,
                );
                if !__anodized_post {
                    return ::anodized::result::post_err(__anodized_output);
                }
                Ok(__anodized_output)
            }
        }
    };

    let observed = Mode::InjectChecks(CheckSettings::PRINT_AND_TRY)
        .instrument_item_trait(trait_spec, item_trait)
        .unwrap();
    assert_tokens_eq(&observed, &expected);
}

#[test]
fn embed_spec_item_impl_trait() {
    let trait_spec = DataSpec::empty();
    let item_impl: ItemImpl = parse_quote! {
        impl TRAIT for IMPL_TYPE {
            #[spec(
                requires: COND_1,
                maintains: COND_2,
                ensures: |PAT_1| COND_3,
            )]
            fn FUNC(&self, PARAM_1: TYPE_1, PARAM_2: TYPE_2) -> RET_TYPE {
                BODY
            }
        }
    };

    let qualifier_bits = FnQualifiers::empty().bits();
    let expected: TokenStream = parse_quote! {
        impl TRAIT for IMPL_TYPE {
            #[doc(hidden)]
            #[allow(warnings)]
            fn __anodized_fn_requires_FUNC(&self, PARAM_1: TYPE_1, PARAM_2: TYPE_2) -> bool {
                let __anodized_clause_1 = ::anodized::__::eval::<bool>(|| COND_1);
                let __anodized_clause_2 = ::anodized::__::eval::<bool>(|| COND_2);
                __anodized_clause_1 && __anodized_clause_2
            }

            #[doc(hidden)]
            #[allow(warnings)]
            fn __anodized_fn_ensures_FUNC(&self, PARAM_1: TYPE_1, PARAM_2: TYPE_2, __anodized_output: RET_TYPE) -> bool {
                let __anodized_clause_1 = ::anodized::__::eval::<bool>(|| COND_2);
                let () = ();
                let __anodized_clause_2 = ::anodized::__::eval::<bool>(|| { let PAT_1 = __anodized_output; COND_3 });
                __anodized_clause_1 && __anodized_clause_2
            }

            #[doc(hidden)]
            #[allow(warnings)]
            const __anodized_fn_qualifiers_FUNC: u32 = #qualifier_bits;

            fn FUNC(&self, PARAM_1: TYPE_1, PARAM_2: TYPE_2) -> RET_TYPE {
                BODY
            }
        }
    };

    let observed = Mode::EmbedSpecs
        .instrument_item_trait_impl(trait_spec, item_impl)
        .unwrap();
    assert_tokens_eq(&observed, &expected);
}

#[test]
fn default_instrument_item_impl_trait() {
    let trait_spec = DataSpec::empty();
    let item_impl: ItemImpl = parse_quote! {
        impl TRAIT for IMPL_TYPE {
            #[spec(
                requires: COND_1,
                maintains: COND_2,
                ensures: |PAT_1| COND_3,
            )]
            fn FUNC(&self, PARAM_1: TYPE_1, PARAM_2: TYPE_2) -> RET_TYPE {
                BODY
            }
        }
    };

    let qualifier_bits = FnQualifiers::empty().bits();
    let expected: TokenStream = parse_quote! {
        impl TRAIT for IMPL_TYPE {
            #[doc(hidden)]
            #[allow(warnings)]
            const __anodized_fn_qualifiers_FUNC: u32 = #qualifier_bits;

            #[inline]
            fn __anodized_FUNC(&self, PARAM_1: TYPE_1, PARAM_2: TYPE_2) -> RET_TYPE {
                const {
                    assert!(
                        Self::__anodized_fn_qualifiers_FUNC ==
                            Self::__anodized_fn_qualifiers_trait_FUNC |
                            Self::__anodized_fn_qualifiers_FUNC,
                        "the qualifiers on the impl `IMPL_TYPE::FUNC` cannot be weaker than the qualifiers on the trait `TRAIT::FUNC`",
                    );
                };
                let __anodized_pre = true;
                let __anodized_pre = __anodized_pre & (true || ::anodized::__::eval::<bool>(|| COND_1));
                let __anodized_pre = __anodized_pre & (true || ::anodized::__::eval::<bool>(|| COND_2));
                if !__anodized_pre {}
                let (__anodized_output) = (::anodized::__::eval_once(|| -> RET_TYPE { BODY }));
                let __anodized_post = true;
                let __anodized_post = __anodized_post & (true || ::anodized::__::eval::<bool>(|| COND_2));
                let (__anodized_post, __anodized_output) = ::anodized::__::apply_keep(
                    |PAT_1| (__anodized_post & (true || ::anodized::__::eval::<bool>(|| COND_3)), PAT_1),
                    __anodized_output,
                );
                if !__anodized_post {}
                __anodized_output
            }
        }
    };

    let observed = Mode::DEFAULT
        .instrument_item_trait_impl(trait_spec, item_impl)
        .unwrap();
    assert_tokens_eq(&observed, &expected);
}

#[test]
fn emit_try_fn_instrument_item_impl_trait() {
    let trait_spec = DataSpec::empty();
    let item_impl: ItemImpl = parse_quote! {
        impl TRAIT for IMPL_TYPE {
            #[spec(
                requires: COND_1,
                maintains: COND_2,
                ensures: |PAT_1| COND_3,
            )]
            fn FUNC(&self, PARAM_1: TYPE_1, PARAM_2: TYPE_2) -> RET_TYPE {
                BODY
            }
        }
    };

    let qualifier_bits = FnQualifiers::empty().bits();
    let expected: TokenStream = parse_quote! {
        impl TRAIT for IMPL_TYPE {
            #[doc(hidden)]
            #[allow(warnings)]
            const __anodized_fn_qualifiers_FUNC: u32 = #qualifier_bits;

            #[inline]
            fn __anodized_FUNC(&self, PARAM_1: TYPE_1, PARAM_2: TYPE_2) -> RET_TYPE {
                const {
                    assert!(
                        Self::__anodized_fn_qualifiers_FUNC ==
                            Self::__anodized_fn_qualifiers_trait_FUNC |
                            Self::__anodized_fn_qualifiers_FUNC,
                        "the qualifiers on the impl `IMPL_TYPE::FUNC` cannot be weaker than the qualifiers on the trait `TRAIT::FUNC`",
                    );
                };
                let __anodized_pre = true;
                let __anodized_pre = __anodized_pre & (::anodized::__::eval::<bool>(|| COND_1)
                    || eprintln!("precondition failed: {}", "COND_1") != ());
                let __anodized_pre = __anodized_pre & (::anodized::__::eval::<bool>(|| COND_2)
                    || eprintln!("preinvariant failed: {}", "COND_2") != ());
                if !__anodized_pre {
                    panic!("precondition failed");
                }
                let (__anodized_output) = (::anodized::__::eval_once(|| -> RET_TYPE { BODY }));
                let __anodized_post = true;
                let __anodized_post = __anodized_post & (::anodized::__::eval::<bool>(|| COND_2)
                        || eprintln!("postinvariant failed: {}", "COND_2") != ());
                let (__anodized_post, __anodized_output) = ::anodized::__::apply_keep(
                    |PAT_1| (__anodized_post & (::anodized::__::eval::<bool>(|| COND_3)
                        || eprintln!("postcondition failed: {}", "COND_3") != ()), PAT_1),
                    __anodized_output,
                );
                if !__anodized_post {
                    panic!("postcondition failed");
                }
                __anodized_output
            }
        }
    };

    let observed = Mode::InjectChecks(CheckSettings::PRINT_AND_TRY)
        .instrument_item_trait_impl(trait_spec, item_impl)
        .unwrap();
    assert_tokens_eq(&observed, &expected);
}
