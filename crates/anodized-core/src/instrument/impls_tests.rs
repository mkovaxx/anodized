use crate::{
    instrument::CheckSettings,
    qualifiers::FnQualifiers,
    test_util::{SpecItemImpl, assert_tokens_eq},
};

use super::*;
use proc_macro2::TokenStream;
use syn::parse_quote;

#[test]
fn embed_spec_item_impl() {
    let spec_item_impl: SpecItemImpl = parse_quote! {
        #[spec]
        impl IMPL_TYPE {
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
        impl IMPL_TYPE {
            #[doc(hidden)]
            #[allow(warnings)]
            const __anodized_fn_qualifiers_FUNC: u32 = #qualifier_bits;

            #[doc(hidden)]
            #[allow(warnings)]
            fn __anodized_fn_requires_FUNC(&self, PARAM_1: TYPE_1, PARAM_2: TYPE_2) -> bool {
                let __anodized_pre = true;
                let __anodized_pre = __anodized_pre & ::anodized::__::eval::<bool>(|| COND_1);
                let __anodized_pre = __anodized_pre & ::anodized::__::eval::<bool>(|| COND_2);
                __anodized_pre
            }

            #[doc(hidden)]
            #[allow(warnings)]
            fn __anodized_fn_ensures_FUNC(&self, PARAM_1: TYPE_1, PARAM_2: TYPE_2, __anodized_output: RET_TYPE) -> bool {
                let __anodized_post = true;
                let __anodized_post = ::anodized::__::eval::<bool>(|| COND_2);
                let () = ();
                let __anodized_post = __anodized_post & ::anodized::__::eval::<bool>(|| { let PAT_1 = __anodized_output; COND_3 });
                __anodized_post
            }

            fn FUNC(&self, PARAM_1: TYPE_1, PARAM_2: TYPE_2) -> RET_TYPE {
                BODY
            }
        }
    };

    let observed = Mode::EmbedSpecs
        .instrument_item_impl(spec_item_impl.spec, spec_item_impl.node)
        .unwrap();
    assert_tokens_eq(&observed, &expected);
}

#[test]
fn default_instrument_item_impl() {
    let spec_item_impl: SpecItemImpl = parse_quote! {
        #[spec]
        impl IMPL_TYPE {
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

    let expected: TokenStream = parse_quote! {
        impl IMPL_TYPE {
            fn FUNC(&self, PARAM_1: TYPE_1, PARAM_2: TYPE_2) -> RET_TYPE {
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
        .instrument_item_impl(spec_item_impl.spec, spec_item_impl.node)
        .unwrap();
    assert_tokens_eq(&observed, &expected);
}

#[test]
fn emit_try_fn_instrument_item_impl() {
    let spec_item_impl: SpecItemImpl = parse_quote! {
        #[spec]
        impl IMPL_TYPE {
            #[spec(
                requires: COND_1,
                maintains: COND_2,
                ensures: |PAT_1| COND_3,
            )]
            // An associated `fn` (no receiver) is syntactically identical to a free-standing `fn`.
            fn FUNC(PARAM_1: TYPE_1, PARAM_2: TYPE_2) -> RET_TYPE {
                BODY
            }
        }
    };

    let expected: TokenStream = parse_quote! {
        impl IMPL_TYPE {
            fn FUNC(input_0: TYPE_1, input_1: TYPE_2) -> RET_TYPE {
                match Self::__anodized_fn_try_FUNC(input_0, input_1) {
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
            fn __anodized_fn_try_FUNC(PARAM_1: TYPE_1, PARAM_2: TYPE_2)
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
                    return ::anodized::result::post_err(__anodized_output);
                }
                Ok(__anodized_output)
            }
        }
    };

    let observed = Mode::InjectChecks(CheckSettings::PRINT_AND_TRY)
        .instrument_item_impl(spec_item_impl.spec, spec_item_impl.node)
        .unwrap();
    assert_tokens_eq(&observed, &expected);
}
