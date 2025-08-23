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
fn test_instrument_simple() {
    let contract: Contract = parse_quote! {
        requires: self.is_valid(),
    };
    let body = make_fn_body();
    let is_async = false;

    let expected: Block = parse_quote! {
        {
            assert!(self.is_valid(), "Precondition failed: self.is_valid()");
            let __anodized_output = #body;
            __anodized_output
        }
    };

    let observed = instrument_fn_body(&contract, &body, is_async).unwrap();
    assert_block_eq(&observed, &expected);
}
