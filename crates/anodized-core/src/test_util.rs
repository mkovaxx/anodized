use crate::{CloneBinding, Condition, PostCondition, Spec};
use quote::ToTokens;
use proc_macro2::TokenStream;
use syn::{
    Arm, Block, Expr,
    parse::{Parse, ParseStream, Result},
    parse_quote,
};

impl Parse for Condition {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(Condition {
            expr: input.parse()?,
            cfg: None,
        })
    }
}

impl Parse for PostCondition {
    fn parse(input: ParseStream) -> Result<Self> {
        // First parse as an expression
        let expr: Expr = input.parse()?;
        
        // Try to reparse it as a match arm (pattern => expr)
        let tokens = expr.to_token_stream();
        if let Ok(arm) = syn::parse2::<Arm>(tokens) {
            if arm.guard.is_none() {
                return Ok(PostCondition {
                    pattern: arm.pat,
                    expr: *arm.body,
                    cfg: None,
                });
            } else {
                return Err(syn::Error::new_spanned(
                    arm.guard.unwrap().1,
                    "guards are not supported in postcondition patterns",
                ));
            }
        }
        
        // Otherwise it's a naked expression with default pattern
        Ok(PostCondition {
            pattern: parse_quote! { output },
            expr,
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
        clones: left_clones,
        ensures: left_ensures,
    } = left;

    let Spec {
        requires: right_requires,
        maintains: right_maintains,
        clones: right_clones,
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
        left_clones,
        right_clones,
        "clones",
        &assert_clone_binding_eq,
    );
    assert_slice_eq(
        left_ensures,
        right_ensures,
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

fn assert_condition_closure_eq(left: &PostCondition, right: &PostCondition, msg_prefix: &str) {
    // Destructure to ensure we handle all fields
    let PostCondition {
        pattern: left_pattern,
        expr: left_expr,
        cfg: left_cfg,
    } = left;

    let PostCondition {
        pattern: right_pattern,
        expr: right_expr,
        cfg: right_cfg,
    } = right;

    assert_eq!(
        left_pattern.to_token_stream().to_string(),
        right_pattern.to_token_stream().to_string(),
        "{}`pattern` does not match",
        msg_prefix
    );

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

fn assert_clone_binding_eq(left: &CloneBinding, right: &CloneBinding, msg_prefix: &str) {
    // Destructure to ensure we handle all fields
    let CloneBinding {
        expr: left_expr,
        alias: left_alias,
    } = left;

    let CloneBinding {
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
