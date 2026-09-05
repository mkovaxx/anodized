use crate::{
    Capture, Condition, DataSpec, EmptySpec, FnSpec, PostCondition, annotate::Specified,
    instrument::patterns::TamePat,
};
use pretty_assertions::assert_eq;
use quote::{ToTokens, quote};
use syn::parse::{Parse, ParseStream};

/// Specified `fn`.
pub type SpecItemFn = NodeWithSpec<FnSpec, syn::ItemFn>;

/// Specified `impl`.
pub type SpecItemImpl = NodeWithSpec<EmptySpec, syn::ItemImpl>;

/// Specified `trait`.
pub type SpecItemTrait = NodeWithSpec<EmptySpec, syn::ItemTrait>;

/// Specified `struct`.
pub type SpecItemStruct = NodeWithSpec<DataSpec, syn::ItemStruct>;

/// Specified `enum`.
pub type SpecItemEnum = NodeWithSpec<DataSpec, syn::ItemEnum>;

/// An AST node with a spec attached.
pub struct NodeWithSpec<Spec, AstNode> {
    pub spec: Spec,
    pub node: AstNode,
}

impl<AstNode: Parse + Specified> Parse for NodeWithSpec<AstNode::Spec, AstNode> {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut node: AstNode = input.parse()?;
        let spec = node.parse_spec_from_attrs()?;
        Ok(Self { spec, node })
    }
}

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

pub fn assert_spec_eq(left: &FnSpec, right: &FnSpec) {
    // Destructure to ensure we handle all fields - compilation will fail if fields are added
    let FnSpec {
        qualifiers: left_qualifiers,
        input_spec_flags: left_input_specs,
        output_spec_flag: left_output_spec_on_exit,
        requires: left_requires,
        maintains: left_maintains,
        captures: left_captures,
        ensures: left_ensures,
        span: _,
    } = left;

    let FnSpec {
        qualifiers: right_qualifiers,
        input_spec_flags: right_input_specs,
        output_spec_flag: right_output_spec_on_exit,
        requires: right_requires,
        maintains: right_maintains,
        captures: right_captures,
        ensures: right_ensures,
        span: _,
    } = right;

    assert_eq!(
        left_qualifiers, right_qualifiers,
        "qualifiers do not match: {left_qualifiers:?} vs {right_qualifiers:?}"
    );
    assert_eq!(
        left_output_spec_on_exit, right_output_spec_on_exit,
        "output spec on exit does not match"
    );
    assert_slice_eq(
        left_input_specs,
        right_input_specs,
        "input specs",
        |left, right, message| {
            assert_eq!(
                left.on_entry, right.on_entry,
                "{message} entry flags do not match"
            );
            assert_eq!(
                left.on_exit, right.on_exit,
                "{message} exit flags do not match"
            );
        },
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
    assert_slice_eq(
        left_ensures,
        right_ensures,
        "ensures",
        assert_postcondition_eq,
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
    let PostCondition {
        pat: left_pat,
        expr: left_expr,
        cfg: left_cfg,
    } = left;
    let PostCondition {
        pat: right_pat,
        expr: right_expr,
        cfg: right_cfg,
    } = right;

    assert_eq!(left_pat, right_pat, "{}`pat` does not match", msg_prefix);
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
        pat: left_alias,
        expr: left_expr,
    } = left;

    let Capture {
        pat: right_alias,
        expr: right_expr,
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

pub fn assert_tame_pat_eq(left: &syn::Result<TamePat>, right: &syn::Result<TamePat>) {
    assert_eq!(
        stringify_tame_pat(left),
        stringify_tame_pat(right),
        "tame patterns do not match"
    );
}

fn stringify_tame_pat(tame_pat: &syn::Result<TamePat>) -> String {
    let tokens = match tame_pat {
        Ok(TamePat::Borrowing(brw_pat)) => quote! { brw #brw_pat },
        Ok(TamePat::Invertible(inv_pat, inv_expr)) => quote! { inv #inv_pat => #inv_expr },
        Err(err) => {
            let msg = err.to_string();
            quote! { err #msg }
        }
    };
    tokens.to_string()
}
