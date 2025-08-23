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
    let contract: Contract = parse_quote! {
        requires: CONDITION_1,
    };
    let body = make_fn_body();
    let is_async = false;

    let expected: Block = parse_quote! {
        {
            assert!(CONDITION_1, "Precondition failed: CONDITION_1");
            let __anodized_output = #body;
            __anodized_output
        }
    };

    let observed = instrument_fn_body(&contract, &body, is_async).unwrap();
    assert_block_eq(&observed, &expected);
}

#[test]
fn test_instrument_simple_maintains() {
    let contract: Contract = parse_quote! {
        maintains: CONDITION_1,
    };
    let body = make_fn_body();
    let is_async = false;

    let expected: Block = parse_quote! {
        {
            assert!(CONDITION_1, "Pre-invariant failed: CONDITION_1");
            let __anodized_output = #body;
            assert!(CONDITION_1, "Post-invariant failed: CONDITION_1");
            __anodized_output
        }
    };

    let observed = instrument_fn_body(&contract, &body, is_async).unwrap();
    assert_block_eq(&observed, &expected);
}

#[test]
fn test_instrument_simple_ensures() {
    let contract: Contract = parse_quote! {
        ensures: CONDITION_1,
    };
    let body = make_fn_body();
    let is_async = false;

    let expected: Block = parse_quote! {
        {
            let __anodized_output = #body;
            assert!(
                (|output| CONDITION_1)(__anodized_output),
                "Postcondition failed: | output | CONDITION_1"
            );
            __anodized_output
        }
    };

    let observed = instrument_fn_body(&contract, &body, is_async).unwrap();
    assert_block_eq(&observed, &expected);
}

#[test]
fn test_instrument_simple_requires_and_maintains() {
    let contract: Contract = parse_quote! {
        requires: CONDITION_1,
        maintains: CONDITION_2,
    };
    let body = make_fn_body();
    let is_async = false;

    let expected: Block = parse_quote! {
        {
            assert!(CONDITION_1, "Precondition failed: CONDITION_1");
            assert!(CONDITION_2, "Pre-invariant failed: CONDITION_2");
            let __anodized_output = #body;
            assert!(CONDITION_2, "Post-invariant failed: CONDITION_2");
            __anodized_output
        }
    };

    let observed = instrument_fn_body(&contract, &body, is_async).unwrap();
    assert_block_eq(&observed, &expected);
}

#[test]
fn test_instrument_simple_requires_and_ensures() {
    let contract: Contract = parse_quote! {
        requires: CONDITION_1,
        ensures: CONDITION_2,
    };
    let body = make_fn_body();
    let is_async = false;

    let expected: Block = parse_quote! {
        {
            assert!(CONDITION_1, "Precondition failed: CONDITION_1");
            let __anodized_output = #body;
            assert!(
                (|output| CONDITION_2)(__anodized_output),
                "Postcondition failed: | output | CONDITION_2"
            );
            __anodized_output
        }
    };

    let observed = instrument_fn_body(&contract, &body, is_async).unwrap();
    assert_block_eq(&observed, &expected);
}

#[test]
fn test_instrument_simple_maintains_and_ensures() {
    let contract: Contract = parse_quote! {
        maintains: CONDITION_1,
        ensures: CONDITION_2,
    };
    let body = make_fn_body();
    let is_async = false;

    let expected: Block = parse_quote! {
        {
            assert!(CONDITION_1, "Pre-invariant failed: CONDITION_1");
            let __anodized_output = #body;
            assert!(CONDITION_1, "Post-invariant failed: CONDITION_1");
            assert!(
                (|output| CONDITION_2)(__anodized_output),
                "Postcondition failed: | output | CONDITION_2"
            );
            __anodized_output
        }
    };

    let observed = instrument_fn_body(&contract, &body, is_async).unwrap();
    assert_block_eq(&observed, &expected);
}

#[test]
fn test_instrument_simple_requires_maintains_and_ensures() {
    let contract: Contract = parse_quote! {
        requires: CONDITION_1,
        maintains: CONDITION_2,
        ensures: CONDITION_3,
    };
    let body = make_fn_body();
    let is_async = false;

    let expected: Block = parse_quote! {
        {
            assert!(CONDITION_1, "Precondition failed: CONDITION_1");
            assert!(CONDITION_2, "Pre-invariant failed: CONDITION_2");
            let __anodized_output = #body;
            assert!(CONDITION_2, "Post-invariant failed: CONDITION_2");
            assert!(
                (|output| CONDITION_3)(__anodized_output),
                "Postcondition failed: | output | CONDITION_3"
            );
            __anodized_output
        }
    };

    let observed = instrument_fn_body(&contract, &body, is_async).unwrap();
    assert_block_eq(&observed, &expected);
}

#[test]
fn test_instrument_simple_async_requires_maintains_and_ensures() {
    let contract: Contract = parse_quote! {
        requires: CONDITION_1,
        maintains: CONDITION_2,
        ensures: CONDITION_3,
    };
    let body = make_fn_body();
    let is_async = true;

    let expected: Block = parse_quote! {
        {
            assert!(CONDITION_1, "Precondition failed: CONDITION_1");
            assert!(CONDITION_2, "Pre-invariant failed: CONDITION_2");
            let __anodized_output = async #body.await;
            assert!(CONDITION_2, "Post-invariant failed: CONDITION_2");
            assert!(
                (|output| CONDITION_3)(__anodized_output),
                "Postcondition failed: | output | CONDITION_3"
            );
            __anodized_output
        }
    };

    let observed = instrument_fn_body(&contract, &body, is_async).unwrap();
    assert_block_eq(&observed, &expected);
}

#[test]
fn test_instrument_multiple_conditions_in_clauses() {
    let contract: Contract = parse_quote! {
        requires: [CONDITION_1, CONDITION_2],
        maintains: [CONDITION_3, CONDITION_4],
        ensures: [CONDITION_5, CONDITION_6],
    };
    let body = make_fn_body();
    let is_async = false;

    let expected: Block = parse_quote! {
        {
            assert!(CONDITION_1, "Precondition failed: CONDITION_1");
            assert!(CONDITION_2, "Precondition failed: CONDITION_2");
            assert!(CONDITION_3, "Pre-invariant failed: CONDITION_3");
            assert!(CONDITION_4, "Pre-invariant failed: CONDITION_4");
            let __anodized_output = #body;
            assert!(CONDITION_3, "Post-invariant failed: CONDITION_3");
            assert!(CONDITION_4, "Post-invariant failed: CONDITION_4");
            assert!(
                (|output| CONDITION_5)(__anodized_output),
                "Postcondition failed: | output | CONDITION_5"
            );
            assert!(
                (|output| CONDITION_6)(__anodized_output),
                "Postcondition failed: | output | CONDITION_6"
            );
            __anodized_output
        }
    };

    let observed = instrument_fn_body(&contract, &body, is_async).unwrap();
    assert_block_eq(&observed, &expected);
}
