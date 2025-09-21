use super::*;

use quote::ToTokens;
use syn::{Attribute, ItemFn, parse_quote};

fn assert_attr_eq(actual: &Attribute, expected: &Attribute) {
    assert_eq!(
        actual.to_token_stream().to_string(),
        expected.to_token_stream().to_string()
    );
}

#[test]
fn requires_clause_emits_contracts_attribute() {
    let spec: Spec = parse_quote! {
        requires: CONDITION,
    };
    let func: ItemFn = parse_quote! {
        fn demo() {}
    };

    let instrumented = instrument_fn(&spec, func).expect("requires should succeed");

    let expected: Attribute = parse_quote! { #[contracts::requires(CONDITION)] };
    assert_eq!(instrumented.attrs.len(), 1);
    assert_attr_eq(&instrumented.attrs[0], &expected);
}

#[test]
fn requires_with_cfg_emits_cfg_attr_contracts_attribute() {
    let spec: Spec = parse_quote! {
        #[cfg(SETTING)]
        requires: CONDITION,
    };
    let func: ItemFn = parse_quote! {
        fn demo() {}
    };

    let instrumented = instrument_fn(&spec, func).expect("requires should succeed");

    let expected: Attribute = parse_quote! { #[cfg_attr(SETTING, contracts::requires(CONDITION))] };
    assert_eq!(instrumented.attrs.len(), 1);
    assert_attr_eq(&instrumented.attrs[0], &expected);
}

#[test]
fn maintains_emits_requires_and_ensures_attributes() {
    let spec: Spec = parse_quote! {
        maintains: CONDITION,
    };
    let func: ItemFn = parse_quote! {
        fn demo() {}
    };

    let instrumented = instrument_fn(&spec, func).expect("maintains should succeed");

    let expected_requires: Attribute = parse_quote! { #[contracts::requires(CONDITION)] };
    let expected_ensures: Attribute = parse_quote! { #[contracts::ensures(|_| CONDITION)] };

    assert_eq!(instrumented.attrs.len(), 2);
    assert_attr_eq(&instrumented.attrs[0], &expected_requires);
    assert_attr_eq(&instrumented.attrs[1], &expected_ensures);
}

#[test]
fn ensures_from_expression_uses_generated_closure() {
    let spec: Spec = parse_quote! {
        ensures: CONDITION,
    };
    let func: ItemFn = parse_quote! {
        fn demo() {}
    };

    let instrumented = instrument_fn(&spec, func).expect("ensures should succeed");

    let expected: Attribute = parse_quote! { #[contracts::ensures(|output| CONDITION)] };

    assert_eq!(instrumented.attrs.len(), 1);
    assert_attr_eq(&instrumented.attrs[0], &expected);
}

#[test]
fn ensures_with_custom_closure_is_preserved() {
    let spec: Spec = parse_quote! {
        ensures: |result| result.is_ok(),
    };
    let func: ItemFn = parse_quote! {
        fn demo() {}
    };

    let instrumented = instrument_fn(&spec, func).expect("ensures should succeed");

    let expected: Attribute = parse_quote! { #[contracts::ensures(|result| result.is_ok())] };

    assert_eq!(instrumented.attrs.len(), 1);
    assert_attr_eq(&instrumented.attrs[0], &expected);
}

#[test]
fn existing_attributes_are_preserved_after_contracts_attributes() {
    let spec: Spec = parse_quote! {
        requires: CONDITION,
    };
    let func: ItemFn = parse_quote! {
        #[inline]
        fn demo() {}
    };

    let instrumented = instrument_fn(&spec, func).expect("requires should succeed");

    assert_eq!(instrumented.attrs.len(), 2);

    let expected_requires: Attribute = parse_quote! { #[contracts::requires(CONDITION)] };
    assert_attr_eq(&instrumented.attrs[0], &expected_requires);
    let expected_inline: Attribute = parse_quote! { #[inline] };
    assert_attr_eq(&instrumented.attrs[1], &expected_inline);
}

#[test]
#[should_panic(expected = "not supported by the nightly contracts backend")]
fn reject_captures() {
    let spec: Spec = parse_quote! {
        captures: value as old_value,
    };
    let func: ItemFn = parse_quote! {
        fn demo() {}
    };

    match instrument_fn(&spec, func) {
        Ok(_) => panic!("expected captures to be rejected"),
        Err(err) => panic!("{}", err),
    }
}
