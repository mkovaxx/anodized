use crate::{Condition, ConditionClosure, Contract};
use quote::ToTokens;
use syn::{
    Block,
    parse::{Parse, ParseStream, Result},
};

impl Parse for Condition {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(Condition {
            expr: input.parse()?,
            cfg: None,
        })
    }
}

impl Parse for ConditionClosure {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(ConditionClosure {
            closure: input.parse()?,
            cfg: None,
        })
    }
}

pub fn assert_block_eq(left: &Block, right: &Block) {
    let left_str = left.to_token_stream().to_string();
    let right_str = right.to_token_stream().to_string();
    assert_eq!(left_str, right_str);
}

pub fn assert_contract_eq(left: &Contract, right: &Contract) {
    assert_slice_eq(
        &left.requires,
        &right.requires,
        "requires",
        &assert_condition_eq,
    );
    assert_slice_eq(
        &left.maintains,
        &right.maintains,
        "maintains",
        &assert_condition_eq,
    );
    assert_slice_eq(
        &left.ensures,
        &right.ensures,
        "ensures",
        &assert_condition_closure_eq,
    );
}

fn assert_slice_eq<T, F>(left: &[T], right: &[T], item_name: &str, assert_item_eq: F)
where
    F: Fn(&T, &T, &str),
{
    assert_eq!(
        left.len(),
        right.len(),
        "number of `{}` items do not match",
        item_name
    );

    for (i, (left_item, right_item)) in left.iter().zip(right.iter()).enumerate() {
        let msg_prefix = format!("`{}` items at index {}, ", item_name, i);
        assert_item_eq(left_item, right_item, &msg_prefix);
    }
}

fn assert_condition_eq(left: &Condition, right: &Condition, msg_prefix: &str) {
    assert_eq!(
        left.expr.to_token_stream().to_string(),
        right.expr.to_token_stream().to_string(),
        "{}`expr` does not match",
        msg_prefix
    );

    assert_eq!(
        left.cfg.to_token_stream().to_string(),
        right.cfg.to_token_stream().to_string(),
        "{}`cfg` does not match",
        msg_prefix
    );
}

fn assert_condition_closure_eq(
    left: &ConditionClosure,
    right: &ConditionClosure,
    msg_prefix: &str,
) {
    assert_eq!(
        left.closure.to_token_stream().to_string(),
        right.closure.to_token_stream().to_string(),
        "{}`closure` does not match",
        msg_prefix
    );

    assert_eq!(
        left.cfg.to_token_stream().to_string(),
        right.cfg.to_token_stream().to_string(),
        "{}`cfg` does not match",
        msg_prefix
    );
}
