use crate::{Capture, Condition, PostCondition, Spec};
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

pub fn assert_block_eq(left: &Block, right: &Block) {
    let left_str = left.to_token_stream().to_string();
    let right_str = right.to_token_stream().to_string();
    assert_eq!(left_str, right_str);
}

pub fn assert_spec_eq(left: &Spec, right: &Spec) {
    // Destructure to ensure we handle all fields - compilation will fail if fields are added
    let Spec {
        requires: left_requires,
        maintains: left_maintains,
        captures: left_captures,
        ensures: left_ensures,
    } = left;

    let Spec {
        requires: right_requires,
        maintains: right_maintains,
        captures: right_captures,
        ensures: right_ensures,
    } = right;

    assert_slice_eq(
        left_requires,
        right_requires,
        "requires",
        &assert_condition_eq,
    );
    assert_slice_eq(
        left_maintains,
        right_maintains,
        "maintains",
        &assert_condition_eq,
    );
    assert_slice_eq(
        left_captures,
        right_captures,
        "captures",
        &assert_capture_eq,
    );
    assert_slice_eq(
        left_ensures,
        right_ensures,
        "ensures",
        &assert_postcondition_eq,
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
    // Destructure to ensure we handle all fields
    let Condition {
        expr: left_expr,
        cfg: left_cfg,
    } = left;

    let Condition {
        expr: right_expr,
        cfg: right_cfg,
    } = right;

    assert_eq!(
        left_expr.to_token_stream().to_string(),
        right_expr.to_token_stream().to_string(),
        "{}`expr` does not match",
        msg_prefix
    );

    assert_eq!(
        left_cfg.to_token_stream().to_string(),
        right_cfg.to_token_stream().to_string(),
        "{}`cfg` does not match",
        msg_prefix
    );
}

fn assert_postcondition_eq(left: &PostCondition, right: &PostCondition, msg_prefix: &str) {
    // Destructure to ensure we handle all fields
    let PostCondition {
        closure: left_closure,
        cfg: left_cfg,
    } = left;

    let PostCondition {
        closure: right_closure,
        cfg: right_cfg,
    } = right;

    assert_eq!(
        left_closure.to_token_stream().to_string(),
        right_closure.to_token_stream().to_string(),
        "{}`closure` does not match",
        msg_prefix
    );

    assert_eq!(
        left_cfg.to_token_stream().to_string(),
        right_cfg.to_token_stream().to_string(),
        "{}`cfg` does not match",
        msg_prefix
    );
}

fn assert_capture_eq(left: &Capture, right: &Capture, msg_prefix: &str) {
    // Destructure to ensure we handle all fields
    let Capture {
        expr: left_expr,
        alias: left_alias,
    } = left;

    let Capture {
        expr: right_expr,
        alias: right_alias,
    } = right;

    assert_eq!(
        left_expr.to_token_stream().to_string(),
        right_expr.to_token_stream().to_string(),
        "{}`expr` does not match",
        msg_prefix
    );

    assert_eq!(
        left_alias.to_token_stream().to_string(),
        right_alias.to_token_stream().to_string(),
        "{}`alias` does not match",
        msg_prefix
    );
}
