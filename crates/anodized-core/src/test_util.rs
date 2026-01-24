use crate::{Capture, PostCondition, PreCondition, Spec};
use quote::ToTokens;

pub fn assert_tokens_eq(left: &impl ToTokens, right: &impl ToTokens) {
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
        span: _,
    } = left;

    let Spec {
        requires: right_requires,
        maintains: right_maintains,
        captures: right_captures,
        ensures: right_ensures,
        span: _,
    } = right;

    assert_slice_eq(
        left_requires,
        right_requires,
        "requires",
        &assert_precondition_eq,
    );
    assert_slice_eq(
        left_maintains,
        right_maintains,
        "maintains",
        &assert_precondition_eq,
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

fn assert_precondition_eq(left: &PreCondition, right: &PreCondition, msg_prefix: &str) {
    // Destructure to ensure we handle all fields
    let PreCondition {
        closure: left_expr,
        cfg: left_cfg,
    } = left;

    let PreCondition {
        closure: right_expr,
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
