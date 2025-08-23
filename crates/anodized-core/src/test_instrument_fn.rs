use super::*;
use quote::ToTokens;
use syn::{Block, ItemFn, parse_quote};

fn assert_body_eq(actual: &Block, expected: &Block) {
    let actual_str = actual.to_token_stream().to_string();
    let expected_str = expected.to_token_stream().to_string();
    assert_eq!(actual_str, expected_str);
}

#[test]
fn test_instrument_simple() {
    let contract: Contract = parse_quote! {
        requires: x > 0,
        ensures: output > x,
    };
    let body: Block = parse_quote! {
        {
            this_is_the_function_body()
        }
    };
    let is_async = false;
    let expected: Block = parse_quote! {
        {
            assert!(x > 0, "Precondition failed: x > 0");
            let __anodized_output = {
                this_is_the_function_body()
            };
            assert!((|output| output > x)(__anodized_output), "Postcondition failed: | output | output > x");
            __anodized_output
        }
    };

    let observed = instrument_function_body(&contract, &body, is_async).unwrap();
    assert_body_eq(&observed, &expected);
}
