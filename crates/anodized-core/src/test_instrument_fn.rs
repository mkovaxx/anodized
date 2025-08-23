use crate::test_util::assert_block_eq;

use super::*;
use syn::{Block, parse_quote};

#[test]
fn test_instrument_simple() {
    let contract: Contract = parse_quote! {
        requires: x > 0,
        ensures: output > x,
    };
    let body: Block = parse_quote! {
        {
            this_is_the_body()
        }
    };
    let is_async = false;

    let expected: Block = parse_quote! {
        {
            assert!(x > 0, "Precondition failed: x > 0");
            let __anodized_output = {
                this_is_the_body()
            };
            assert!((|output| output > x)(__anodized_output), "Postcondition failed: | output | output > x");
            __anodized_output
        }
    };

    let observed = instrument_fn_body(&contract, &body, is_async).unwrap();
    assert_block_eq(&observed, &expected);
}

#[test]
fn test_instrument_async_simple() {
    let contract: Contract = parse_quote! {
        requires: x > 0,
        ensures: output > x,
    };
    let body: Block = parse_quote! {
        {
            this_is_the_async_body()
        }
    };
    let is_async = true;

    let expected: Block = parse_quote! {
        {
            assert!(x > 0, "Precondition failed: x > 0");
            let __anodized_output = async {
                this_is_the_async_body()
            }
            .await;
            assert!((|output| output > x)(__anodized_output), "Postcondition failed: | output | output > x");
            __anodized_output
        }
    };

    let observed = instrument_fn_body(&contract, &body, is_async).unwrap();
    assert_block_eq(&observed, &expected);
}
