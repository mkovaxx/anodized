use crate::test_util::assert_block_eq;

use super::*;
use syn::{Block, parse_quote};

fn make_fn_body() -> Block {
    parse_quote! {
        {
            this_is_the_body()
        }
    }
}

#[test]
fn test_instrument_simple_requires() {
    let spec: Spec = parse_quote! {
        requires: CONDITION_1,
    };
    let body = make_fn_body();
    let is_async = false;

    let expected: Block = parse_quote! {
        {
            assert!(CONDITION_1, "Precondition failed: {}", "CONDITION_1");
            let (__anodized_output) = (#body);
            __anodized_output
        }
    };

    let observed = instrument_fn_body(&spec, &body, is_async).unwrap();
    assert_block_eq(&observed, &expected);
}

#[test]
fn test_instrument_simple_maintains() {
    let spec: Spec = parse_quote! {
        maintains: CONDITION_1,
    };
    let body = make_fn_body();
    let is_async = false;

    let expected: Block = parse_quote! {
        {
            assert!(CONDITION_1, "Pre-invariant failed: {}", "CONDITION_1");
            let (__anodized_output) = (#body);
            assert!(CONDITION_1, "Post-invariant failed: {}", "CONDITION_1");
            __anodized_output
        }
    };

    let observed = instrument_fn_body(&spec, &body, is_async).unwrap();
    assert_block_eq(&observed, &expected);
}

#[test]
fn test_instrument_simple_ensures() {
    let spec: Spec = parse_quote! {
        ensures: CONDITION_1,
    };
    let body = make_fn_body();
    let is_async = false;

    let expected: Block = parse_quote! {
        {
            let (__anodized_output) = (#body);
            assert!(
                (|output| CONDITION_1)(__anodized_output),
                "Postcondition failed: {}", "| output | CONDITION_1"
            );
            __anodized_output
        }
    };

    let observed = instrument_fn_body(&spec, &body, is_async).unwrap();
    assert_block_eq(&observed, &expected);
}

#[test]
fn test_instrument_simple_requires_and_maintains() {
    let spec: Spec = parse_quote! {
        requires: CONDITION_1,
        maintains: CONDITION_2,
    };
    let body = make_fn_body();
    let is_async = false;

    let expected: Block = parse_quote! {
        {
            assert!(CONDITION_1, "Precondition failed: {}", "CONDITION_1");
            assert!(CONDITION_2, "Pre-invariant failed: {}", "CONDITION_2");
            let (__anodized_output) = (#body);
            assert!(CONDITION_2, "Post-invariant failed: {}", "CONDITION_2");
            __anodized_output
        }
    };

    let observed = instrument_fn_body(&spec, &body, is_async).unwrap();
    assert_block_eq(&observed, &expected);
}

#[test]
fn test_instrument_simple_requires_and_ensures() {
    let spec: Spec = parse_quote! {
        requires: CONDITION_1,
        ensures: CONDITION_2,
    };
    let body = make_fn_body();
    let is_async = false;

    let expected: Block = parse_quote! {
        {
            assert!(CONDITION_1, "Precondition failed: {}", "CONDITION_1");
            let (__anodized_output) = (#body);
            assert!(
                (|output| CONDITION_2)(__anodized_output),
                "Postcondition failed: {}", "| output | CONDITION_2"
            );
            __anodized_output
        }
    };

    let observed = instrument_fn_body(&spec, &body, is_async).unwrap();
    assert_block_eq(&observed, &expected);
}

#[test]
fn test_instrument_simple_maintains_and_ensures() {
    let spec: Spec = parse_quote! {
        maintains: CONDITION_1,
        ensures: CONDITION_2,
    };
    let body = make_fn_body();
    let is_async = false;

    let expected: Block = parse_quote! {
        {
            assert!(CONDITION_1, "Pre-invariant failed: {}", "CONDITION_1");
            let (__anodized_output) = (#body);
            assert!(CONDITION_1, "Post-invariant failed: {}", "CONDITION_1");
            assert!(
                (|output| CONDITION_2)(__anodized_output),
                "Postcondition failed: {}", "| output | CONDITION_2"
            );
            __anodized_output
        }
    };

    let observed = instrument_fn_body(&spec, &body, is_async).unwrap();
    assert_block_eq(&observed, &expected);
}

#[test]
fn test_instrument_simple_requires_maintains_and_ensures() {
    let spec: Spec = parse_quote! {
        requires: CONDITION_1,
        maintains: CONDITION_2,
        ensures: CONDITION_3,
    };
    let body = make_fn_body();
    let is_async = false;

    let expected: Block = parse_quote! {
        {
            assert!(CONDITION_1, "Precondition failed: {}", "CONDITION_1");
            assert!(CONDITION_2, "Pre-invariant failed: {}", "CONDITION_2");
            let (__anodized_output) = (#body);
            assert!(CONDITION_2, "Post-invariant failed: {}", "CONDITION_2");
            assert!(
                (|output| CONDITION_3)(__anodized_output),
                "Postcondition failed: {}", "| output | CONDITION_3"
            );
            __anodized_output
        }
    };

    let observed = instrument_fn_body(&spec, &body, is_async).unwrap();
    assert_block_eq(&observed, &expected);
}

#[test]
fn test_instrument_simple_async_requires_maintains_and_ensures() {
    let spec: Spec = parse_quote! {
        requires: CONDITION_1,
        maintains: CONDITION_2,
        ensures: CONDITION_3,
    };
    let body = make_fn_body();
    let is_async = true;

    let expected: Block = parse_quote! {
        {
            assert!(CONDITION_1, "Precondition failed: {}", "CONDITION_1");
            assert!(CONDITION_2, "Pre-invariant failed: {}", "CONDITION_2");
            let (__anodized_output) = (async #body.await);
            assert!(CONDITION_2, "Post-invariant failed: {}", "CONDITION_2");
            assert!(
                (|output| CONDITION_3)(__anodized_output),
                "Postcondition failed: {}", "| output | CONDITION_3"
            );
            __anodized_output
        }
    };

    let observed = instrument_fn_body(&spec, &body, is_async).unwrap();
    assert_block_eq(&observed, &expected);
}

#[test]
fn test_instrument_multiple_conditions_in_clauses() {
    let spec: Spec = parse_quote! {
        requires: [CONDITION_1, CONDITION_2],
        maintains: [CONDITION_3, CONDITION_4],
        ensures: [CONDITION_5, CONDITION_6],
    };
    let body = make_fn_body();
    let is_async = false;

    let expected: Block = parse_quote! {
        {
            assert!(CONDITION_1, "Precondition failed: {}", "CONDITION_1");
            assert!(CONDITION_2, "Precondition failed: {}", "CONDITION_2");
            assert!(CONDITION_3, "Pre-invariant failed: {}", "CONDITION_3");
            assert!(CONDITION_4, "Pre-invariant failed: {}", "CONDITION_4");
            let (__anodized_output) = (#body);
            assert!(CONDITION_3, "Post-invariant failed: {}", "CONDITION_3");
            assert!(CONDITION_4, "Post-invariant failed: {}", "CONDITION_4");
            assert!(
                (|output| CONDITION_5)(__anodized_output),
                "Postcondition failed: {}", "| output | CONDITION_5"
            );
            assert!(
                (|output| CONDITION_6)(__anodized_output),
                "Postcondition failed: {}", "| output | CONDITION_6"
            );
            __anodized_output
        }
    };

    let observed = instrument_fn_body(&spec, &body, is_async).unwrap();
    assert_block_eq(&observed, &expected);
}

#[test]
fn test_instrument_with_binds_parameter() {
    let spec: Spec = parse_quote! {
        binds: OUTPUT_PATTERN,
        ensures: CONDITION_1,
    };
    let body = make_fn_body();
    let is_async = false;

    let expected: Block = parse_quote! {
        {
            let (__anodized_output) = (#body);
            assert!(
                (|OUTPUT_PATTERN| CONDITION_1)(__anodized_output),
                "Postcondition failed: {}", "| OUTPUT_PATTERN | CONDITION_1"
            );
            __anodized_output
        }
    };

    let observed = instrument_fn_body(&spec, &body, is_async).unwrap();
    assert_block_eq(&observed, &expected);
}

#[test]
fn test_instrument_ensures_with_mixed_conditions() {
    let spec: Spec = parse_quote! {
        ensures: [
            CONDITION_1,
            |PATTERN_1| CONDITION_2,
            CONDITION_3,
            |PATTERN_2| CONDITION_4
        ],
    };
    let body = make_fn_body();
    let is_async = false;

    let expected: Block = parse_quote! {
        {
            let (__anodized_output) = (#body);
            assert!(
                (|output| CONDITION_1)(__anodized_output),
                "Postcondition failed: {}", "| output | CONDITION_1"
            );
            assert!(
                (|PATTERN_1| CONDITION_2)(__anodized_output),
                "Postcondition failed: {}", "| PATTERN_1 | CONDITION_2"
            );
            assert!(
                (|output| CONDITION_3)(__anodized_output),
                "Postcondition failed: {}", "| output | CONDITION_3"
            );
            assert!(
                (|PATTERN_2| CONDITION_4)(__anodized_output),
                "Postcondition failed: {}", "| PATTERN_2 | CONDITION_4"
            );
            __anodized_output
        }
    };

    let observed = instrument_fn_body(&spec, &body, is_async).unwrap();
    assert_block_eq(&observed, &expected);
}

#[test]
fn test_instrument_with_cfg_attributes() {
    let spec: Spec = parse_quote! {
        #[cfg(SETTING_1)]
        requires: CONDITION_1,
        #[cfg(SETTING_2)]
        maintains: CONDITION_2,
        #[cfg(SETTING_3)]
        ensures: CONDITION_3,
    };
    let body = make_fn_body();
    let is_async = false;

    let expected: Block = parse_quote! {
        {
            if cfg!(SETTING_1) {
                assert!(CONDITION_1, "Precondition failed: {}", "CONDITION_1");
            }
            if cfg!(SETTING_2) {
                assert!(CONDITION_2, "Pre-invariant failed: {}", "CONDITION_2");
            }
            let (__anodized_output) = (#body);
            if cfg!(SETTING_2) {
                assert!(CONDITION_2, "Post-invariant failed: {}", "CONDITION_2");
            }
            if cfg!(SETTING_3) {
                assert!(
                    (|output| CONDITION_3)(__anodized_output),
                    "Postcondition failed: {}", "| output | CONDITION_3"
                );
            }
            __anodized_output
        }
    };

    let observed = instrument_fn_body(&spec, &body, is_async).unwrap();
    assert_block_eq(&observed, &expected);
}

#[test]
fn test_instrument_with_cfg_on_single_and_list_conditions() {
    let spec: Spec = parse_quote! {
        #[cfg(SETTING_1)]
        requires: CONDITION_1,
        maintains: [CONDITION_2, CONDITION_3],
        #[cfg(SETTING_2)]
        ensures: [CONDITION_4, CONDITION_5],
    };
    let body = make_fn_body();
    let is_async = false;

    let expected: Block = parse_quote! {
        {
            if cfg!(SETTING_1) {
                assert!(CONDITION_1, "Precondition failed: {}", "CONDITION_1");
            }
            assert!(CONDITION_2, "Pre-invariant failed: {}", "CONDITION_2");
            assert!(CONDITION_3, "Pre-invariant failed: {}", "CONDITION_3");
            let (__anodized_output) = (#body);
            assert!(CONDITION_2, "Post-invariant failed: {}", "CONDITION_2");
            assert!(CONDITION_3, "Post-invariant failed: {}", "CONDITION_3");
            if cfg!(SETTING_2) {
                assert!(
                    (|output| CONDITION_4)(__anodized_output),
                    "Postcondition failed: {}", "| output | CONDITION_4"
                );
            }
            if cfg!(SETTING_2) {
                assert!(
                    (|output| CONDITION_5)(__anodized_output),
                    "Postcondition failed: {}", "| output | CONDITION_5"
                );
            }
            __anodized_output
        }
    };

    let observed = instrument_fn_body(&spec, &body, is_async).unwrap();
    assert_block_eq(&observed, &expected);
}

#[test]
fn test_instrument_with_complex_mixed_conditions() {
    let spec: Spec = parse_quote! {
        requires: CONDITION_1,
        #[cfg(SETTING_1)]
        requires: [CONDITION_2, CONDITION_3],
        maintains: [CONDITION_4, CONDITION_5],
        #[cfg(SETTING_2)]
        maintains: CONDITION_6,
        ensures: CONDITION_7,
        #[cfg(SETTING_3)]
        ensures: [CONDITION_8, |PATTERN_1| CONDITION_9],
    };
    let body = make_fn_body();
    let is_async = false;

    let expected: Block = parse_quote! {
        {
            assert!(CONDITION_1, "Precondition failed: {}", "CONDITION_1");
            if cfg!(SETTING_1) {
                assert!(CONDITION_2, "Precondition failed: {}", "CONDITION_2");
            }
            if cfg!(SETTING_1) {
                assert!(CONDITION_3, "Precondition failed: {}", "CONDITION_3");
            }
            assert!(CONDITION_4, "Pre-invariant failed: {}", "CONDITION_4");
            assert!(CONDITION_5, "Pre-invariant failed: {}", "CONDITION_5");
            if cfg!(SETTING_2) {
                assert!(CONDITION_6, "Pre-invariant failed: {}", "CONDITION_6");
            }
            let (__anodized_output) = (#body);
            assert!(CONDITION_4, "Post-invariant failed: {}", "CONDITION_4");
            assert!(CONDITION_5, "Post-invariant failed: {}", "CONDITION_5");
            if cfg!(SETTING_2) {
                assert!(CONDITION_6, "Post-invariant failed: {}", "CONDITION_6");
            }
            assert!(
                (|output| CONDITION_7)(__anodized_output),
                "Postcondition failed: {}", "| output | CONDITION_7"
            );
            if cfg!(SETTING_3) {
                assert!(
                    (|output| CONDITION_8)(__anodized_output),
                    "Postcondition failed: {}", "| output | CONDITION_8"
                );
            }
            if cfg!(SETTING_3) {
                assert!(
                    (|PATTERN_1| CONDITION_9)(__anodized_output),
                    "Postcondition failed: {}", "| PATTERN_1 | CONDITION_9"
                );
            }
            __anodized_output
        }
    };

    let observed = instrument_fn_body(&spec, &body, is_async).unwrap();
    assert_block_eq(&observed, &expected);
}

#[test]
fn test_instrument_with_clones() {
    let spec: Spec = parse_quote! {
        requires: CONDITION_1,
        captures: [
            EXPR_1 as ALIAS_1,
            EXPR_2 as ALIAS_2,
        ],
        ensures: [
            CONDITION_2,
            CONDITION_3,
        ],
    };
    let body = make_fn_body();
    let is_async = false;

    let expected: Block = parse_quote! {
        {
            assert!(CONDITION_1, "Precondition failed: {}", "CONDITION_1");
            let (ALIAS_1, ALIAS_2, __anodized_output) = ((EXPR_1).clone(), (EXPR_2).clone(), #body);
            assert!(
                (|output| CONDITION_2)(__anodized_output),
                "Postcondition failed: {}", "| output | CONDITION_2"
            );
            assert!(
                (|output| CONDITION_3)(__anodized_output),
                "Postcondition failed: {}", "| output | CONDITION_3"
            );
            __anodized_output
        }
    };

    let observed = instrument_fn_body(&spec, &body, is_async).unwrap();
    assert_block_eq(&observed, &expected);
}
