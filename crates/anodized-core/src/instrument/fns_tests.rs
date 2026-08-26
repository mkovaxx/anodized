use crate::test_util::assert_tokens_eq;

use super::*;
use proc_macro2::TokenStream;
use syn::{Block, ItemFn, parse_quote};

fn make_complex_spec() -> Spec {
    parse_quote! {
        requires: COND_1,
        #[cfg(META_1)]
        requires: [COND_2, COND_3],
        maintains: [COND_4, COND_5],
        #[cfg(META_2)]
        maintains: COND_6,
        captures: [
            ALIAS_1 = EXPR_1,
            (ALIAS_2, ALIAS_3) = EXPR_2,
        ],
        ensures: |PAT_1| COND_7,
        #[cfg(META_3)]
        ensures: |PAT_1| [
            COND_8,
            COND_9,
        ],
    }
}

#[test]
fn embed_spec_item_fn() {
    let fn_spec = make_complex_spec();
    let item_fn: ItemFn = parse_quote! {
        fn FUNC(&self, PARAM_1: TYPE_1, PARAM_2: TYPE_2) -> RET_TYPE {
            BODY
        }
    };

    let qualifier_bits = FnQualifiers::empty().bits();
    let expected: TokenStream = parse_quote! {
        #[doc(hidden)]
        #[allow(warnings)]
        const __anodized_fn_qualifiers_FUNC: u32 = #qualifier_bits;

        #[doc(hidden)]
        #[allow(warnings)]
        fn __anodized_fn_requires_FUNC(&self, PARAM_1: TYPE_1, PARAM_2: TYPE_2) -> bool {
            let __anodized_clause_1 = ::anodized::__::eval::<bool>(|| COND_1);
            let __anodized_clause_2 = ::anodized::__::eval::<bool>(|| COND_2);
            let __anodized_clause_3 = ::anodized::__::eval::<bool>(|| COND_3);
            let __anodized_clause_4 = ::anodized::__::eval::<bool>(|| COND_4);
            let __anodized_clause_5 = ::anodized::__::eval::<bool>(|| COND_5);
            let __anodized_clause_6 = ::anodized::__::eval::<bool>(|| COND_6);
            __anodized_clause_1 && __anodized_clause_2 && __anodized_clause_3
                && __anodized_clause_4 && __anodized_clause_5 && __anodized_clause_6
        }

        #[doc(hidden)]
        #[allow(warnings)]
        fn __anodized_fn_ensures_FUNC(&self, PARAM_1: TYPE_1, PARAM_2: TYPE_2, __anodized_output: RET_TYPE) -> bool {
            let __anodized_clause_1 = ::anodized::__::eval::<bool>(|| COND_4);
            let __anodized_clause_2 = ::anodized::__::eval::<bool>(|| COND_5);
            let __anodized_clause_3 = ::anodized::__::eval::<bool>(|| COND_6);
            let (ALIAS_1, (ALIAS_2, ALIAS_3)) = (
                ::anodized::__::eval(|| EXPR_1),
                ::anodized::__::eval(|| EXPR_2),
            );
            let __anodized_clause_4 = ::anodized::__::eval::<bool>(|| { let PAT_1 = __anodized_output; COND_7 });
            let __anodized_clause_5 = ::anodized::__::eval::<bool>(|| { let PAT_1 = __anodized_output; COND_8 });
            let __anodized_clause_6 = ::anodized::__::eval::<bool>(|| { let PAT_1 = __anodized_output; COND_9 });
            __anodized_clause_1 && __anodized_clause_2 && __anodized_clause_3
                && __anodized_clause_4 && __anodized_clause_5 && __anodized_clause_6
        }

        fn FUNC(&self, PARAM_1: TYPE_1, PARAM_2: TYPE_2) -> RET_TYPE {
            BODY
        }
    };

    let observed = Mode::EmbedSpecs
        .instrument_item_fn(fn_spec, item_fn)
        .unwrap();
    assert_tokens_eq(&observed, &expected);
}

#[test]
fn default_instrument_item_fn() {
    let fn_spec = make_complex_spec();
    let item_fn: ItemFn = parse_quote! {
        fn FUNC(&self, PARAM_1: TYPE_1, PARAM_2: TYPE_2) -> RET_TYPE {
            BODY
        }
    };

    let expected: TokenStream = parse_quote! {
        fn FUNC(&self, PARAM_1: TYPE_1, PARAM_2: TYPE_2) -> RET_TYPE {
            if false {
                let __anodized_pre = true;
                let __anodized_pre = __anodized_pre & ::anodized::__::eval::<bool>(|| COND_1);
                let __anodized_pre = __anodized_pre & (!cfg!(META_1) || ::anodized::__::eval::<bool>(|| COND_2));
                let __anodized_pre = __anodized_pre & (!cfg!(META_1) || ::anodized::__::eval::<bool>(|| COND_3));
                let __anodized_pre = __anodized_pre & ::anodized::__::eval::<bool>(|| COND_4);
                let __anodized_pre = __anodized_pre & ::anodized::__::eval::<bool>(|| COND_5);
                let __anodized_pre = __anodized_pre & (!cfg!(META_2) || ::anodized::__::eval::<bool>(|| COND_6));
                if !__anodized_pre {}
            }
            let (ALIAS_1, (ALIAS_2, ALIAS_3), __anodized_output) = (
                ::anodized::__::eval(|| EXPR_1),
                ::anodized::__::eval(|| EXPR_2),
                (|| -> RET_TYPE { BODY })(),
            );
            if false {
                let __anodized_post = true;
                let __anodized_post = __anodized_post & ::anodized::__::eval::<bool>(|| COND_4);
                let __anodized_post = __anodized_post & ::anodized::__::eval::<bool>(|| COND_5);
                let __anodized_post = __anodized_post & (!cfg!(META_2) || ::anodized::__::eval::<bool>(|| COND_6));
                let __anodized_post = __anodized_post & ::anodized::__::eval::<bool>(|| { let PAT_1 = __anodized_output; COND_7 });
                let __anodized_post = __anodized_post & (!cfg!(META_3) || ::anodized::__::eval::<bool>(|| { let PAT_1 = __anodized_output; COND_8 }));
                let __anodized_post = __anodized_post & (!cfg!(META_3) || ::anodized::__::eval::<bool>(|| { let PAT_1 = __anodized_output; COND_9 }));
                if !__anodized_post {}
            }
            __anodized_output
        }
    };

    let observed = Mode::DEFAULT.instrument_item_fn(fn_spec, item_fn).unwrap();
    assert_tokens_eq(&observed, &expected);
}

#[test]
fn check_data_instrument_item_fn() {
    let fn_spec: Spec = parse_quote! {
        requires: COND_1,
        maintains: COND_2,
        ensures: |OUT_PAT| COND_3,
    };
    let item_fn: ItemFn = parse_quote! {
        fn FUNC(IN_PAT_1: TYPE_1, IN_PAT_2: TYPE_2) -> RET_TYPE {
            BODY
        }
    };

    let expected: TokenStream = parse_quote! {
        fn FUNC(IN_PAT_1: TYPE_1, IN_PAT_2: TYPE_2) -> RET_TYPE {
            if false {
                let __anodized_pre = true;
                let __anodized_pre = __anodized_pre &
                    <TYPE_1 as ::anodized::refine::Refined>::predicate(&IN_PAT_1);
                let __anodized_pre = __anodized_pre &
                    <TYPE_2 as ::anodized::refine::Refined>::predicate(&IN_PAT_2);
                let __anodized_pre = __anodized_pre & ::anodized::__::eval::<bool>(|| COND_1);
                let __anodized_pre = __anodized_pre & ::anodized::__::eval::<bool>(|| COND_2);
                if !__anodized_pre {}
            }
            let (__anodized_output) = ((|| -> RET_TYPE { BODY })());
            if false {
                let __anodized_post = true;
                let __anodized_post = __anodized_post &
                    <TYPE_1 as ::anodized::refine::Refined>::predicate(&IN_PAT_1);
                let __anodized_post = __anodized_post &
                    <TYPE_2 as ::anodized::refine::Refined>::predicate(&IN_PAT_2);
                let __anodized_post = __anodized_post &
                    <RET_TYPE as ::anodized::refine::Refined>::predicate(&__anodized_output);
                let __anodized_post = __anodized_post & ::anodized::__::eval::<bool>(|| COND_2);
                let __anodized_post = __anodized_post &
                    ::anodized::__::eval::<bool>(|| { let OUT_PAT = __anodized_output; COND_3 });
                if !__anodized_post {}
            }
            __anodized_output
        }
    };

    let observed = Mode::InjectChecks(CheckSettings::CHECK_DATA)
        .instrument_item_fn(fn_spec, item_fn)
        .unwrap();
    assert_tokens_eq(&observed, &expected);
}

#[test]
fn emit_try_fn_instrument_item_fn() {
    let fn_spec = make_complex_spec();
    let item_fn: ItemFn = parse_quote! {
        fn FUNC(&self, PARAM_1: TYPE_1, PARAM_2: TYPE_2) -> RET_TYPE {
            BODY
        }
    };

    let expected: TokenStream = parse_quote! {
        fn FUNC(&self, input_1: TYPE_1, input_2: TYPE_2) -> RET_TYPE {
            match __anodized_fn_try_FUNC(self, input_1, input_2) {
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
            if true {
                let __anodized_pre = true;
                let __anodized_pre = __anodized_pre & (::anodized::__::eval::<bool>(|| COND_1)
                        || eprintln!("precondition failed: {}", "COND_1") != ());
                let __anodized_pre = __anodized_pre & (!cfg!(META_1) || ::anodized::__::eval::<bool>(|| COND_2)
                        || eprintln!("precondition failed: {}", "COND_2") != ());
                let __anodized_pre = __anodized_pre & (!cfg!(META_1) || ::anodized::__::eval::<bool>(|| COND_3)
                        || eprintln!("precondition failed: {}", "COND_3") != ());
                let __anodized_pre = __anodized_pre & (::anodized::__::eval::<bool>(|| COND_4)
                        || eprintln!("precondition failed: {}", "COND_4") != ());
                let __anodized_pre = __anodized_pre & (::anodized::__::eval::<bool>(|| COND_5)
                        || eprintln!("precondition failed: {}", "COND_5") != ());
                let __anodized_pre = __anodized_pre & (!cfg!(META_2) || ::anodized::__::eval::<bool>(|| COND_6)
                        || eprintln!("precondition failed: {}", "COND_6") != ());
                if !__anodized_pre {
                    return ::anodized::result::pre_err();
                }
            }
            let (ALIAS_1, (ALIAS_2, ALIAS_3), __anodized_output) = (
                ::anodized::__::eval(|| EXPR_1),
                ::anodized::__::eval(|| EXPR_2),
                (|| -> RET_TYPE { BODY })(),
            );
            if true {
                let __anodized_post = true;
                let __anodized_post = __anodized_post & (::anodized::__::eval::<bool>(|| COND_4)
                        || eprintln!("postcondition failed: {}", "COND_4") != ());
                let __anodized_post = __anodized_post & (::anodized::__::eval::<bool>(|| COND_5)
                        || eprintln!("postcondition failed: {}", "COND_5") != ());
                let __anodized_post = __anodized_post & (!cfg!(META_2) || ::anodized::__::eval::<bool>(|| COND_6)
                        || eprintln!("postcondition failed: {}", "COND_6") != ());
                let __anodized_post = __anodized_post & (::anodized::__::eval::<bool>(|| { let PAT_1 = __anodized_output; COND_7 })
                        || eprintln!("postcondition failed: {}", "COND_7") != ());
                let __anodized_post = __anodized_post & (!cfg!(META_3) || ::anodized::__::eval::<bool>(|| { let PAT_1 = __anodized_output; COND_8 })
                        || eprintln!("postcondition failed: {}", "COND_8") != ());
                let __anodized_post = __anodized_post & (!cfg!(META_3) || ::anodized::__::eval::<bool>(|| { let PAT_1 = __anodized_output; COND_9 })
                        || eprintln!("postcondition failed: {}", "COND_9") != ());
                if !__anodized_post {
                    return ::anodized::result::post_err(__anodized_output);
                }
            }
            Ok(__anodized_output)
        }
    };

    let observed = Mode::InjectChecks(CheckSettings::PRINT_AND_TRY)
        .instrument_item_fn(fn_spec, item_fn)
        .unwrap();
    assert_tokens_eq(&observed, &expected);
}

fn make_fn_body() -> Block {
    parse_quote! {
        {
            this_is_the_body()
        }
    }
}

fn make_return_type() -> ReturnType {
    parse_quote! { -> SomeType }
}

#[test]
fn simple_requires() {
    let spec: Spec = parse_quote! {
        requires: CONDITION_1,
    };
    let body = make_fn_body();
    let ret_type = make_return_type();
    let is_async = false;

    let expected: Block = parse_quote! {
        {
            if true {
                let __anodized_pre = true;
                let __anodized_pre = __anodized_pre & (::anodized::__::eval::<bool>(|| CONDITION_1)
                    || eprintln!("precondition failed: {}", "CONDITION_1") != ());
                if !__anodized_pre {
                    panic!("precondition failed");
                }
            }
            let (__anodized_output) = ((|| #ret_type #body)());
            if true {
                let __anodized_post = true;
                if !__anodized_post {
                    panic!("postcondition failed");
                }
            }
            __anodized_output
        }
    };

    let observed = CheckSettings::PRINT_AND_PANIC
        .instrument_fn_body(&spec, &body, is_async, &ret_type)
        .unwrap();
    assert_tokens_eq(&observed, &expected);
}

#[test]
fn requires_disable_runtime_checks() {
    let spec: Spec = parse_quote! {
        requires: CONDITION_1,
    };
    let body = make_fn_body();
    let ret_type = make_return_type();
    let is_async = false;

    let observed = CheckSettings::DEFAULT
        .instrument_fn_body(&spec, &body, is_async, &ret_type)
        .unwrap();
    let expected: Block = parse_quote! {
        {
            if false {
                let __anodized_pre = true;
                let __anodized_pre = __anodized_pre & ::anodized::__::eval::<bool>(|| CONDITION_1);
                if !__anodized_pre {}
            }
            let (__anodized_output) = ((|| #ret_type #body)());
            if false {
                let __anodized_post = true;
                if !__anodized_post {}
            }
            __anodized_output
        }
    };
    assert_tokens_eq(&observed, &expected);
}

#[test]
fn requires_no_panic_runtime() {
    let spec: Spec = parse_quote! {
        requires: CONDITION_1,
    };
    let body = make_fn_body();
    let ret_type = make_return_type();
    let is_async = false;

    let expected: Block = parse_quote! {
        {
            if true {
                let __anodized_pre = true;
                let __anodized_pre = __anodized_pre & (::anodized::__::eval::<bool>(|| CONDITION_1)
                    || eprintln!("precondition failed: {}", "CONDITION_1") != ());
                if !__anodized_pre {}
            }
            let (__anodized_output) = ((|| #ret_type #body)());
            if true {
                let __anodized_post = true;
                if !__anodized_post {}
            }
            __anodized_output
        }
    };

    let observed = CheckSettings::PRINT
        .instrument_fn_body(&spec, &body, is_async, &ret_type)
        .unwrap();
    assert_tokens_eq(&observed, &expected);
}

#[test]
fn simple_maintains() {
    let spec: Spec = parse_quote! {
        maintains: CONDITION_1,
    };
    let body = make_fn_body();
    let ret_type = make_return_type();
    let is_async = false;

    let expected: Block = parse_quote! {
        {
            if true {
                let __anodized_pre = true;
                let __anodized_pre = __anodized_pre & (::anodized::__::eval::<bool>(|| CONDITION_1)
                    || eprintln!("precondition failed: {}", "CONDITION_1") != ());
                if !__anodized_pre {
                    panic!("precondition failed");
                }
            }
            let (__anodized_output) = ((|| #ret_type #body)());
            if true {
                let __anodized_post = true;
                let __anodized_post = __anodized_post & (::anodized::__::eval::<bool>(|| CONDITION_1)
                    || eprintln!("postcondition failed: {}", "CONDITION_1") != ());
                if !__anodized_post {
                    panic!("postcondition failed");
                }
            }
            __anodized_output
        }
    };

    let observed = CheckSettings::PRINT_AND_PANIC
        .instrument_fn_body(&spec, &body, is_async, &ret_type)
        .unwrap();
    assert_tokens_eq(&observed, &expected);
}

#[test]
fn simple_ensures() {
    let spec: Spec = parse_quote! {
        ensures: CONDITION_1,
    };
    let body = make_fn_body();
    let ret_type = make_return_type();
    let is_async = false;

    let expected: Block = parse_quote! {
        {
            if true {
                let __anodized_pre = true;
                if !__anodized_pre {
                    panic!("precondition failed");
                }
            }
            let (__anodized_output) = ((|| #ret_type #body)());
            if true {
                let __anodized_post = true;
                let __anodized_post = __anodized_post & (::anodized::__::eval::<bool>(|| CONDITION_1)
                    || eprintln!("postcondition failed: {}", "CONDITION_1") != ());
                if !__anodized_post {
                    panic!("postcondition failed");
                }
            }
            __anodized_output
        }
    };

    let observed = CheckSettings::PRINT_AND_PANIC
        .instrument_fn_body(&spec, &body, is_async, &ret_type)
        .unwrap();
    assert_tokens_eq(&observed, &expected);
}

#[test]
fn simple_requires_and_maintains() {
    let spec: Spec = parse_quote! {
        requires: CONDITION_1,
        maintains: CONDITION_2,
    };
    let body = make_fn_body();
    let ret_type = make_return_type();
    let is_async = false;

    let expected: Block = parse_quote! {
        {
            if true {
                let __anodized_pre = true;
                let __anodized_pre = __anodized_pre & (::anodized::__::eval::<bool>(|| CONDITION_1)
                    || eprintln!("precondition failed: {}", "CONDITION_1") != ());
                let __anodized_pre = __anodized_pre & (::anodized::__::eval::<bool>(|| CONDITION_2)
                        || eprintln!("precondition failed: {}", "CONDITION_2") != ());
                if !__anodized_pre {
                    panic!("precondition failed");
                }
            }
            let (__anodized_output) = ((|| #ret_type #body)());
            if true {
                let __anodized_post = true;
                let __anodized_post = __anodized_post & (::anodized::__::eval::<bool>(|| CONDITION_2)
                    || eprintln!("postcondition failed: {}", "CONDITION_2") != ());
                if !__anodized_post {
                    panic!("postcondition failed");
                }
            }
            __anodized_output
        }
    };

    let observed = CheckSettings::PRINT_AND_PANIC
        .instrument_fn_body(&spec, &body, is_async, &ret_type)
        .unwrap();
    assert_tokens_eq(&observed, &expected);
}

#[test]
fn simple_requires_and_ensures() {
    let spec: Spec = parse_quote! {
        requires: CONDITION_1,
        ensures: CONDITION_2,
    };
    let body = make_fn_body();
    let ret_type = make_return_type();
    let is_async = false;

    let expected: Block = parse_quote! {
        {
            if true {
                let __anodized_pre = true;
                let __anodized_pre = __anodized_pre & (::anodized::__::eval::<bool>(|| CONDITION_1)
                    || eprintln!("precondition failed: {}", "CONDITION_1") != ());
                if !__anodized_pre {
                    panic!("precondition failed");
                }
            }
            let (__anodized_output) = ((|| #ret_type #body)());
            if true {
                let __anodized_post = true;
                let __anodized_post = __anodized_post & (::anodized::__::eval::<bool>(|| CONDITION_2)
                    || eprintln!("postcondition failed: {}", "CONDITION_2") != ());
                if !__anodized_post {
                    panic!("postcondition failed");
                }
            }
            __anodized_output
        }
    };

    let observed = CheckSettings::PRINT_AND_PANIC
        .instrument_fn_body(&spec, &body, is_async, &ret_type)
        .unwrap();
    assert_tokens_eq(&observed, &expected);
}

#[test]
fn simple_maintains_and_ensures() {
    let spec: Spec = parse_quote! {
        maintains: CONDITION_1,
        ensures: CONDITION_2,
    };
    let body = make_fn_body();
    let ret_type = make_return_type();
    let is_async = false;

    let expected: Block = parse_quote! {
        {
            if true {
                let __anodized_pre = true;
                let __anodized_pre = __anodized_pre & (::anodized::__::eval::<bool>(|| CONDITION_1)
                    || eprintln!("precondition failed: {}", "CONDITION_1") != ());
                if !__anodized_pre {
                    panic!("precondition failed");
                }
            }
            let (__anodized_output) = ((|| #ret_type #body)());
            if true {
                let __anodized_post = true;
                let __anodized_post = __anodized_post & (::anodized::__::eval::<bool>(|| CONDITION_1)
                    || eprintln!("postcondition failed: {}", "CONDITION_1") != ());
                let __anodized_post = __anodized_post & (::anodized::__::eval::<bool>(|| CONDITION_2)
                        || eprintln!("postcondition failed: {}", "CONDITION_2") != ());
                if !__anodized_post {
                    panic!("postcondition failed");
                }
            }
            __anodized_output
        }
    };

    let observed = CheckSettings::PRINT_AND_PANIC
        .instrument_fn_body(&spec, &body, is_async, &ret_type)
        .unwrap();
    assert_tokens_eq(&observed, &expected);
}

#[test]
fn simple_requires_maintains_and_ensures() {
    let spec: Spec = parse_quote! {
        requires: CONDITION_1,
        maintains: CONDITION_2,
        ensures: CONDITION_3,
    };
    let body = make_fn_body();
    let ret_type = make_return_type();
    let is_async = false;

    let expected: Block = parse_quote! {
        {
            if true {
                let __anodized_pre = true;
                let __anodized_pre = __anodized_pre & (::anodized::__::eval::<bool>(|| CONDITION_1)
                    || eprintln!("precondition failed: {}", "CONDITION_1") != ());
                let __anodized_pre = __anodized_pre & (::anodized::__::eval::<bool>(|| CONDITION_2)
                        || eprintln!("precondition failed: {}", "CONDITION_2") != ());
                if !__anodized_pre {
                    panic!("precondition failed");
                }
            }
            let (__anodized_output) = ((|| #ret_type #body)());
            if true {
                let __anodized_post = true;
                let __anodized_post = __anodized_post & (::anodized::__::eval::<bool>(|| CONDITION_2)
                    || eprintln!("postcondition failed: {}", "CONDITION_2") != ());
                let __anodized_post = __anodized_post & (::anodized::__::eval::<bool>(|| CONDITION_3)
                        || eprintln!("postcondition failed: {}", "CONDITION_3") != ());
                if !__anodized_post {
                    panic!("postcondition failed");
                }
            }
            __anodized_output
        }
    };

    let observed = CheckSettings::PRINT_AND_PANIC
        .instrument_fn_body(&spec, &body, is_async, &ret_type)
        .unwrap();
    assert_tokens_eq(&observed, &expected);
}

#[test]
fn simple_async_requires_maintains_and_ensures() {
    let spec: Spec = parse_quote! {
        requires: CONDITION_1,
        maintains: CONDITION_2,
        ensures: CONDITION_3,
    };
    let body = make_fn_body();
    let ret_type = make_return_type();
    let is_async = true;

    let expected: Block = parse_quote! {
        {
            if true {
                let __anodized_pre = true;
                let __anodized_pre = __anodized_pre & (::anodized::__::eval::<bool>(|| CONDITION_1)
                    || eprintln!("precondition failed: {}", "CONDITION_1") != ());
                let __anodized_pre = __anodized_pre & (::anodized::__::eval::<bool>(|| CONDITION_2)
                        || eprintln!("precondition failed: {}", "CONDITION_2") != ());
                if !__anodized_pre {
                    panic!("precondition failed");
                }
            }
            let (__anodized_output) = ((async || #ret_type #body)().await);
            if true {
                let __anodized_post = true;
                let __anodized_post = __anodized_post & (::anodized::__::eval::<bool>(|| CONDITION_2)
                    || eprintln!("postcondition failed: {}", "CONDITION_2") != ());
                let __anodized_post = __anodized_post & (::anodized::__::eval::<bool>(|| CONDITION_3)
                        || eprintln!("postcondition failed: {}", "CONDITION_3") != ());
                if !__anodized_post {
                    panic!("postcondition failed");
                }
            }
            __anodized_output
        }
    };

    let observed = CheckSettings::PRINT_AND_PANIC
        .instrument_fn_body(&spec, &body, is_async, &ret_type)
        .unwrap();
    assert_tokens_eq(&observed, &expected);
}

#[test]
fn multiple_conditions_in_clauses() {
    let spec: Spec = parse_quote! {
        requires: [CONDITION_1, CONDITION_2],
        maintains: [CONDITION_3, CONDITION_4],
        ensures: [CONDITION_5, CONDITION_6],
    };
    let body = make_fn_body();
    let ret_type = make_return_type();
    let is_async = false;

    let expected: Block = parse_quote! {
        {
            if true {
                let __anodized_pre = true;
                let __anodized_pre = __anodized_pre & (::anodized::__::eval::<bool>(|| CONDITION_1)
                    || eprintln!("precondition failed: {}", "CONDITION_1") != ());
                let __anodized_pre = __anodized_pre & (::anodized::__::eval::<bool>(|| CONDITION_2)
                        || eprintln!("precondition failed: {}", "CONDITION_2") != ());
                let __anodized_pre = __anodized_pre & (::anodized::__::eval::<bool>(|| CONDITION_3)
                        || eprintln!("precondition failed: {}", "CONDITION_3") != ());
                let __anodized_pre = __anodized_pre & (::anodized::__::eval::<bool>(|| CONDITION_4)
                        || eprintln!("precondition failed: {}", "CONDITION_4") != ());
                if !__anodized_pre {
                    panic!("precondition failed");
                }
            }
            let (__anodized_output) = ((|| #ret_type #body)());
            if true {
                let __anodized_post = true;
                let __anodized_post = __anodized_post & (::anodized::__::eval::<bool>(|| CONDITION_3)
                    || eprintln!("postcondition failed: {}", "CONDITION_3") != ());
                let __anodized_post = __anodized_post & (::anodized::__::eval::<bool>(|| CONDITION_4)
                        || eprintln!("postcondition failed: {}", "CONDITION_4") != ());
                let __anodized_post = __anodized_post & (::anodized::__::eval::<bool>(|| CONDITION_5)
                        || eprintln!("postcondition failed: {}", "CONDITION_5") != ());
                let __anodized_post = __anodized_post & (::anodized::__::eval::<bool>(|| CONDITION_6)
                        || eprintln!("postcondition failed: {}", "CONDITION_6") != ());
                if !__anodized_post {
                    panic!("postcondition failed");
                }
            }
            __anodized_output
        }
    };

    let observed = CheckSettings::PRINT_AND_PANIC
        .instrument_fn_body(&spec, &body, is_async, &ret_type)
        .unwrap();
    assert_tokens_eq(&observed, &expected);
}

#[test]
fn postcond_closure_form() {
    let spec: Spec = parse_quote! {
        ensures: |OUTPUT_PATTERN| CONDITION_1,
    };
    let body = make_fn_body();
    let ret_type = make_return_type();
    let is_async = false;

    let expected: Block = parse_quote! {
        {
            if true {
                let __anodized_pre = true;
                if !__anodized_pre {
                    panic!("precondition failed");
                }
            }
            let (__anodized_output) = ((|| #ret_type #body)());
            if true {
                let __anodized_post = true;
                let __anodized_post = __anodized_post & (::anodized::__::eval::<bool>(|| { let OUTPUT_PATTERN = __anodized_output; CONDITION_1 })
                    || eprintln!("postcondition failed: {}", "CONDITION_1") != ());
                if !__anodized_post {
                    panic!("postcondition failed");
                }
            }
            __anodized_output
        }
    };

    let observed = CheckSettings::PRINT_AND_PANIC
        .instrument_fn_body(&spec, &body, is_async, &ret_type)
        .unwrap();
    assert_tokens_eq(&observed, &expected);
}

#[test]
fn ensures_with_mixed_conditions() {
    let spec: Spec = parse_quote! {
        ensures: [
            CONDITION_1,
            CONDITION_2,
            CONDITION_3,
            CONDITION_4
        ],
    };
    let body = make_fn_body();
    let ret_type = make_return_type();
    let is_async = false;

    let expected: Block = parse_quote! {
        {
            if true {
                let __anodized_pre = true;
                if !__anodized_pre {
                    panic!("precondition failed");
                }
            }
            let (__anodized_output) = ((|| #ret_type #body)());
            if true {
                let __anodized_post = true;
                let __anodized_post = __anodized_post & (::anodized::__::eval::<bool>(|| CONDITION_1)
                    || eprintln!("postcondition failed: {}", "CONDITION_1") != ());
                let __anodized_post = __anodized_post & (::anodized::__::eval::<bool>(|| CONDITION_2)
                        || eprintln!("postcondition failed: {}", "CONDITION_2") != ());
                let __anodized_post = __anodized_post & (::anodized::__::eval::<bool>(|| CONDITION_3)
                        || eprintln!("postcondition failed: {}", "CONDITION_3") != ());
                let __anodized_post = __anodized_post & (::anodized::__::eval::<bool>(|| CONDITION_4)
                        || eprintln!("postcondition failed: {}", "CONDITION_4") != ());
                if !__anodized_post {
                    panic!("postcondition failed");
                }
            }
            __anodized_output
        }
    };

    let observed = CheckSettings::PRINT_AND_PANIC
        .instrument_fn_body(&spec, &body, is_async, &ret_type)
        .unwrap();
    assert_tokens_eq(&observed, &expected);
}

#[test]
fn cfg_attributes() {
    let spec: Spec = parse_quote! {
        #[cfg(SETTING_1)]
        requires: CONDITION_1,
        #[cfg(SETTING_2)]
        maintains: CONDITION_2,
        #[cfg(SETTING_3)]
        ensures: CONDITION_3,
    };
    let body = make_fn_body();
    let ret_type = make_return_type();
    let is_async = false;

    let expected: Block = parse_quote! {
        {
            if true {
                let __anodized_pre = true;
                let __anodized_pre = __anodized_pre & (!cfg!(SETTING_1) || ::anodized::__::eval::<bool>(|| CONDITION_1)
                    || eprintln!("precondition failed: {}", "CONDITION_1") != ());
                let __anodized_pre = __anodized_pre & (!cfg!(SETTING_2) || ::anodized::__::eval::<bool>(|| CONDITION_2)
                        || eprintln!("precondition failed: {}", "CONDITION_2") != ());
                if !__anodized_pre {
                    panic!("precondition failed");
                }
            }
            let (__anodized_output) = ((|| #ret_type #body)());
            if true {
                let __anodized_post = true;
                let __anodized_post = __anodized_post & (!cfg!(SETTING_2) || ::anodized::__::eval::<bool>(|| CONDITION_2)
                    || eprintln!("postcondition failed: {}", "CONDITION_2") != ());
                let __anodized_post = __anodized_post & (!cfg!(SETTING_3) || ::anodized::__::eval::<bool>(|| CONDITION_3)
                        || eprintln!("postcondition failed: {}", "CONDITION_3") != ());
                if !__anodized_post {
                    panic!("postcondition failed");
                }
            }
            __anodized_output
        }
    };

    let observed = CheckSettings::PRINT_AND_PANIC
        .instrument_fn_body(&spec, &body, is_async, &ret_type)
        .unwrap();
    assert_tokens_eq(&observed, &expected);
}

#[test]
fn cfg_on_single_and_list_conditions() {
    let spec: Spec = parse_quote! {
        #[cfg(SETTING_1)]
        requires: CONDITION_1,
        maintains: [CONDITION_2, CONDITION_3],
        #[cfg(SETTING_2)]
        ensures: [CONDITION_4, CONDITION_5],
    };
    let body = make_fn_body();
    let ret_type = make_return_type();
    let is_async = false;

    let expected: Block = parse_quote! {
        {
            if true {
                let __anodized_pre = true;
                let __anodized_pre = __anodized_pre & (!cfg!(SETTING_1) || ::anodized::__::eval::<bool>(|| CONDITION_1)
                    || eprintln!("precondition failed: {}", "CONDITION_1") != ());
                let __anodized_pre = __anodized_pre & (::anodized::__::eval::<bool>(|| CONDITION_2)
                        || eprintln!("precondition failed: {}", "CONDITION_2") != ());
                let __anodized_pre = __anodized_pre & (::anodized::__::eval::<bool>(|| CONDITION_3)
                        || eprintln!("precondition failed: {}", "CONDITION_3") != ());
                if !__anodized_pre {
                    panic!("precondition failed");
                }
            }
            let (__anodized_output) = ((|| #ret_type #body)());
            if true {
                let __anodized_post = true;
                let __anodized_post = __anodized_post & (::anodized::__::eval::<bool>(|| CONDITION_2)
                    || eprintln!("postcondition failed: {}", "CONDITION_2") != ());
                let __anodized_post = __anodized_post & (::anodized::__::eval::<bool>(|| CONDITION_3)
                        || eprintln!("postcondition failed: {}", "CONDITION_3") != ());
                let __anodized_post = __anodized_post & (!cfg!(SETTING_2) || ::anodized::__::eval::<bool>(|| CONDITION_4)
                        || eprintln!("postcondition failed: {}", "CONDITION_4") != ());
                let __anodized_post = __anodized_post & (!cfg!(SETTING_2) || ::anodized::__::eval::<bool>(|| CONDITION_5)
                        || eprintln!("postcondition failed: {}", "CONDITION_5") != ());
                if !__anodized_post {
                    panic!("postcondition failed");
                }
            }
            __anodized_output
        }
    };

    let observed = CheckSettings::PRINT_AND_PANIC
        .instrument_fn_body(&spec, &body, is_async, &ret_type)
        .unwrap();
    assert_tokens_eq(&observed, &expected);
}

#[test]
fn complex_mixed_conditions() {
    let spec: Spec = parse_quote! {
        requires: CONDITION_1,
        #[cfg(SETTING_1)]
        requires: [CONDITION_2, CONDITION_3],
        maintains: [CONDITION_4, CONDITION_5],
        #[cfg(SETTING_2)]
        maintains: CONDITION_6,
        ensures: CONDITION_7,
        #[cfg(SETTING_3)]
        ensures: [CONDITION_8, CONDITION_9],
    };
    let body = make_fn_body();
    let ret_type = make_return_type();
    let is_async = false;

    let expected: Block = parse_quote! {
        {
            if true {
                let __anodized_pre = true;
                let __anodized_pre = __anodized_pre & (::anodized::__::eval::<bool>(|| CONDITION_1)
                    || eprintln!("precondition failed: {}", "CONDITION_1") != ());
                let __anodized_pre = __anodized_pre & (!cfg!(SETTING_1) || ::anodized::__::eval::<bool>(|| CONDITION_2)
                        || eprintln!("precondition failed: {}", "CONDITION_2") != ());
                let __anodized_pre = __anodized_pre & (!cfg!(SETTING_1) || ::anodized::__::eval::<bool>(|| CONDITION_3)
                        || eprintln!("precondition failed: {}", "CONDITION_3") != ());
                let __anodized_pre = __anodized_pre & (::anodized::__::eval::<bool>(|| CONDITION_4)
                        || eprintln!("precondition failed: {}", "CONDITION_4") != ());
                let __anodized_pre = __anodized_pre & (::anodized::__::eval::<bool>(|| CONDITION_5)
                        || eprintln!("precondition failed: {}", "CONDITION_5") != ());
                let __anodized_pre = __anodized_pre & (!cfg!(SETTING_2) || ::anodized::__::eval::<bool>(|| CONDITION_6)
                        || eprintln!("precondition failed: {}", "CONDITION_6") != ());
                if !__anodized_pre {
                    panic!("precondition failed");
                }
            }
            let (__anodized_output) = ((|| #ret_type #body)());
            if true {
                let __anodized_post = true;
                let __anodized_post = __anodized_post & (::anodized::__::eval::<bool>(|| CONDITION_4)
                    || eprintln!("postcondition failed: {}", "CONDITION_4") != ());
                let __anodized_post = __anodized_post & (::anodized::__::eval::<bool>(|| CONDITION_5)
                        || eprintln!("postcondition failed: {}", "CONDITION_5") != ());
                let __anodized_post = __anodized_post & (!cfg!(SETTING_2) || ::anodized::__::eval::<bool>(|| CONDITION_6)
                        || eprintln!("postcondition failed: {}", "CONDITION_6") != ());
                let __anodized_post = __anodized_post & (::anodized::__::eval::<bool>(|| CONDITION_7)
                        || eprintln!("postcondition failed: {}", "CONDITION_7") != ());
                let __anodized_post = __anodized_post & (!cfg!(SETTING_3) || ::anodized::__::eval::<bool>(|| CONDITION_8)
                        || eprintln!("postcondition failed: {}", "CONDITION_8") != ());
                let __anodized_post = __anodized_post & (!cfg!(SETTING_3) || ::anodized::__::eval::<bool>(|| CONDITION_9)
                        || eprintln!("postcondition failed: {}", "CONDITION_9") != ());
                if !__anodized_post {
                    panic!("postcondition failed");
                }
            }
            __anodized_output
        }
    };

    let observed = CheckSettings::PRINT_AND_PANIC
        .instrument_fn_body(&spec, &body, is_async, &ret_type)
        .unwrap();
    assert_tokens_eq(&observed, &expected);
}

#[test]
fn captures() {
    let spec: Spec = parse_quote! {
        requires: CONDITION_1,
        captures: [
            ALIAS_1 = EXPR_1,
            ALIAS_2 = EXPR_2,
        ],
        ensures: [
            CONDITION_2,
            CONDITION_3,
        ],
    };
    let body = make_fn_body();
    let ret_type = make_return_type();
    let is_async = false;

    let expected: Block = parse_quote! {
        {
            if true {
                let __anodized_pre = true;
                let __anodized_pre = __anodized_pre & (::anodized::__::eval::<bool>(|| CONDITION_1)
                    || eprintln!("precondition failed: {}", "CONDITION_1") != ());
                if !__anodized_pre {
                    panic!("precondition failed");
                }
            }
            let (ALIAS_1, ALIAS_2, __anodized_output) = (
                ::anodized::__::eval(|| EXPR_1),
                ::anodized::__::eval(|| EXPR_2),
                (|| #ret_type #body)(),
            );
            if true {
                let __anodized_post = true;
                let __anodized_post = __anodized_post & (::anodized::__::eval::<bool>(|| CONDITION_2)
                    || eprintln!("postcondition failed: {}", "CONDITION_2") != ());
                let __anodized_post = __anodized_post & (::anodized::__::eval::<bool>(|| CONDITION_3)
                        || eprintln!("postcondition failed: {}", "CONDITION_3") != ());
                if !__anodized_post {
                    panic!("postcondition failed");
                }
            }
            __anodized_output
        }
    };

    let observed = CheckSettings::PRINT_AND_PANIC
        .instrument_fn_body(&spec, &body, is_async, &ret_type)
        .unwrap();
    assert_tokens_eq(&observed, &expected);
}

#[test]
fn try_call_free_fn() {
    let input: Expr = parse_quote! {
        module::FUNC(arg_1, arg_2)
    };

    let expected: Expr = parse_quote! {
        module::__anodized_fn_try_FUNC(arg_1, arg_2)
    };

    let observed = make_try_call(input).expect("tryify");
    assert_eq!(expected, observed);
}

#[test]
fn try_call_method() {
    let input: Expr = parse_quote! {
        receiver.METHOD(arg_1, arg_2)
    };

    let expected: Expr = parse_quote! {
        receiver.__anodized_fn_try_METHOD(arg_1, arg_2)
    };

    let observed = make_try_call(input).expect("tryify");
    assert_eq!(expected, observed);
}

#[test]
fn try_call_associated_fn() {
    let input: Expr = parse_quote! {
        Type::FUNC(arg_1, arg_2)
    };

    let expected: Expr = parse_quote! {
        Type::__anodized_fn_try_FUNC(arg_1, arg_2)
    };

    let observed = make_try_call(input).expect("tryify");
    assert_eq!(expected, observed);
}

#[test]
fn try_call_turbofish_associated_fn() {
    let input: Expr = parse_quote! {
        <Type>::FUNC(arg_1, arg_2)
    };

    let expected: Expr = parse_quote! {
        <Type>::__anodized_fn_try_FUNC(arg_1, arg_2)
    };

    let observed = make_try_call(input).expect("tryify");
    assert_eq!(expected, observed);
}

#[test]
fn try_call_trait_fn() {
    let input: Expr = parse_quote! {
        <Type as Trait>::FUNC(arg_1, arg_2)
    };

    let expected: Expr = parse_quote! {
        <Type as Trait>::__anodized_fn_try_FUNC(arg_1, arg_2)
    };

    let observed = make_try_call(input).expect("tryify");
    assert_eq!(expected, observed);
}

#[test]
fn try_call_invalid() {
    let input = parse_quote! {
        free_fn(value)
    };
    let error = make_try_call(input).expect_err("invalid input");
    assert_eq!(
        error.to_string(),
        "must be a method call or a qualified function call",
    );
}
