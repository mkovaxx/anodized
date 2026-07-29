use crate::{instrument::CheckSettings, qualifiers::FnQualifiers, test_util::assert_tokens_eq};

use super::*;
use proc_macro2::TokenStream;
use syn::{ItemImpl, parse_quote};

#[test]
fn embed_spec_item_impl() {
    let impl_spec = DataSpec::empty();
    let item_impl: ItemImpl = parse_quote! {
        impl IMPL_TYPE {
            #[spec(
                requires: COND_1,
                maintains: COND_2,
                inspects: PAT_1,
                ensures: COND_3,
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
                let __anodized_clause_1 = (|| -> bool { COND_1 })();
                let __anodized_clause_2 = (|| -> bool { COND_2 })();
                __anodized_clause_1 && __anodized_clause_2
            }

            #[doc(hidden)]
            #[allow(warnings)]
            fn __anodized_fn_ensures_FUNC(&self, PARAM_1: TYPE_1, PARAM_2: TYPE_2, __anodized_output: RET_TYPE) -> bool {
                let __anodized_clause_1 = (|| -> bool { COND_2 })();
                let (PAT_1) = (__anodized_output);
                let __anodized_clause_2 = (|| -> bool { COND_3 })();
                __anodized_clause_1 && __anodized_clause_2
            }

            fn FUNC(&self, PARAM_1: TYPE_1, PARAM_2: TYPE_2) -> RET_TYPE {
                BODY
            }
        }
    };

    let observed = Mode::EmbedSpecs
        .instrument_item_impl(impl_spec, item_impl)
        .unwrap();
    assert_tokens_eq(&observed, &expected);
}

#[test]
fn default_instrument_item_impl() {
    let impl_spec = DataSpec::empty();
    let item_impl: ItemImpl = parse_quote! {
        impl IMPL_TYPE {
            #[spec(
                requires: COND_1,
                maintains: COND_2,
                inspects: PAT_1,
                ensures: COND_3,
            )]
            fn FUNC(&self, PARAM_1: TYPE_1, PARAM_2: TYPE_2) -> RET_TYPE {
                BODY
            }
        }
    };

    let expected: TokenStream = parse_quote! {
        impl IMPL_TYPE {
            fn FUNC(&self, PARAM_1: TYPE_1, PARAM_2: TYPE_2) -> RET_TYPE {
                if false {
                    fn __anodized_eval_pre(c: impl Fn() -> bool) -> bool { c() }
                    let mut __anodized_errors = ::std::string::String::new();
                    let __anodized_precond = __anodized_eval_pre(|| -> bool { COND_1 })
                        & __anodized_eval_pre(|| -> bool { COND_2 });
                    if !__anodized_precond {}
                }
                let (__anodized_output) = ((|| -> RET_TYPE { BODY })());
                if false {
                    fn __anodized_eval_post(c: impl Fn() -> bool) -> bool { c() }
                    let mut __anodized_errors = ::std::string::String::new();
                    let PAT_1 = __anodized_output;
                    let __anodized_postcond = __anodized_eval_post(|| -> bool { COND_2 })
                        & __anodized_eval_post(|| -> bool { COND_3 });
                    if !__anodized_postcond {}
                }
                __anodized_output
            }
        }
    };

    let observed = Mode::DEFAULT
        .instrument_item_impl(impl_spec, item_impl)
        .unwrap();
    assert_tokens_eq(&observed, &expected);
}

#[test]
fn emit_try_fn_instrument_item_impl() {
    let impl_spec = DataSpec::empty();
    let item_impl: ItemImpl = parse_quote! {
        impl IMPL_TYPE {
            #[spec(
                requires: COND_1,
                maintains: COND_2,
                inspects: PAT_1,
                ensures: COND_3,
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
                        ::anodized::result::Error::Pre(errors)
                    ) => panic!("precondition failed:{errors}"),
                    ::anodized::result::Result::Err(
                        ::anodized::result::Error::Post(_, errors)
                    ) => panic!("postcondition failed:{errors}"),
                }
            }

            #[doc(hidden)]
            #[inline]
            fn __anodized_fn_try_FUNC(PARAM_1: TYPE_1, PARAM_2: TYPE_2)
                -> ::anodized::result::Result<RET_TYPE>
            {
                if true {
                    fn __anodized_eval_pre(c: impl Fn() -> bool) -> bool { c() }
                    let mut __anodized_errors = ::std::string::String::new();
                    let __anodized_precond = (__anodized_eval_pre(|| -> bool { COND_1 })
                            || __anodized_errors.push_str("\n    COND_1") != ())
                        & (__anodized_eval_pre(|| -> bool { COND_2 })
                            || __anodized_errors.push_str("\n    COND_2") != ());
                    if !__anodized_precond {
                        return ::anodized::result::pre_err(__anodized_errors);
                    }
                }
                let (__anodized_output) = ((|| -> RET_TYPE { BODY })());
                if true {
                    fn __anodized_eval_post(c: impl Fn() -> bool) -> bool { c() }
                    let mut __anodized_errors = ::std::string::String::new();
                    let PAT_1 = __anodized_output;
                    let __anodized_postcond = (__anodized_eval_post(|| -> bool { COND_2 })
                            || __anodized_errors.push_str("\n    COND_2") != ())
                        & (__anodized_eval_post(|| -> bool { COND_3 })
                            || __anodized_errors.push_str("\n    COND_3") != ());
                    if !__anodized_postcond {
                        return ::anodized::result::post_err(__anodized_output, __anodized_errors);
                    }
                }
                Ok(__anodized_output)
            }
        }
    };

    let observed = Mode::InjectChecks(CheckSettings::PRINT_AND_TRY)
        .instrument_item_impl(impl_spec, item_impl)
        .unwrap();
    assert_tokens_eq(&observed, &expected);
}
