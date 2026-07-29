use crate::{Capture, Condition, Spec};
use pretty_assertions::assert_eq;
use quote::{ToTokens, quote};

pub fn assert_tokens_eq(left: &impl ToTokens, right: &impl ToTokens) {
    let left_str = pretty_print_tokens(left.to_token_stream());
    let right_str = pretty_print_tokens(right.to_token_stream());
    assert_eq!(left_str, right_str);
}

fn pretty_print_tokens(ts: proc_macro2::TokenStream) -> String {
    let file: syn::File = syn::parse2(ts.clone())
        .or_else(|_|
            // Token stream cannot be parsed as top-level items; wrap in function
            syn::parse2(quote! {
                fn main() {
                    #ts
                }
            }))
        .expect("wrap tokens in a file");
    prettyplease::unparse(&file)
}

pub fn assert_spec_eq(left: &Spec, right: &Spec) {
    // Destructure to ensure we handle all fields - compilation will fail if fields are added
    let Spec {
        qualifiers: left_qualifiers,
        requires: left_requires,
        maintains: left_maintains,
        captures: left_captures,
        inspects: left_binds,
        ensures: left_ensures,
        span: _,
    } = left;

    let Spec {
        qualifiers: right_qualifiers,
        requires: right_requires,
        maintains: right_maintains,
        captures: right_captures,
        inspects: right_binds,
        ensures: right_ensures,
        span: _,
    } = right;

    assert_eq!(
        left_qualifiers, right_qualifiers,
        "qualifiers do not match: {left_qualifiers:?} vs {right_qualifiers:?}"
    );

    assert_slice_eq(
        left_requires,
        right_requires,
        "requires",
        assert_condition_eq,
    );
    assert_slice_eq(
        left_maintains,
        right_maintains,
        "maintains",
        assert_condition_eq,
    );
    assert_slice_eq(left_captures, right_captures, "captures", assert_capture_eq);
    assert_eq!(
        left_binds, right_binds,
        "binds patterns do not match: {left_binds:?} vs {right_binds:?}"
    );
    assert_slice_eq(left_ensures, right_ensures, "ensures", assert_condition_eq);
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

fn assert_capture_eq(left: &Capture, right: &Capture, msg_prefix: &str) {
    // Destructure to ensure we handle all fields
    let Capture {
        expr: left_expr,
        pat: left_alias,
    } = left;

    let Capture {
        expr: right_expr,
        pat: right_alias,
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
