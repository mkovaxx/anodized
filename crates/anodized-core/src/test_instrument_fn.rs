use super::*;
use quote::ToTokens;
use syn::{Block, parse_quote};

fn assert_block_eq(left: &Block, right: &Block) {
    let left_str = left.to_token_stream().to_string();
    let right_str = right.to_token_stream().to_string();
    assert_eq!(left_str, right_str);
}

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
