use super::*;

use crate::test_util::assert_fn_eq;
use syn::{ItemFn, parse_quote};

#[test]
fn requires_clause_emits_contracts_attribute() {
    let spec: Spec = parse_quote! {
        requires: CONDITION,
    };
    let func: ItemFn = parse_quote! { fn demo() {} };

    let observed = instrument_fn(&spec, func).expect("requires should succeed");

    let expected: ItemFn = parse_quote! {
        #[contracts::requires(CONDITION)]
        fn demo() {}
    };

    assert_fn_eq(&observed, &expected);
}

#[test]
fn requires_with_cfg_emits_cfg_attr_contracts_attribute() {
    let spec: Spec = parse_quote! {
        #[cfg(SETTING)]
        requires: CONDITION,
    };
    let func: ItemFn = parse_quote! { fn demo() {} };

    let observed = instrument_fn(&spec, func).expect("requires should succeed");

    let expected: ItemFn = parse_quote! {
        #[cfg_attr(SETTING, contracts::requires(CONDITION))]
        fn demo() {}
    };

    assert_fn_eq(&observed, &expected);
}

#[test]
fn maintains_emits_requires_and_ensures_attributes() {
    let spec: Spec = parse_quote! {
        maintains: CONDITION,
    };
    let func: ItemFn = parse_quote! { fn demo() {} };

    let observed = instrument_fn(&spec, func).expect("maintains should succeed");

    let expected: ItemFn = parse_quote! {
        #[contracts::requires(CONDITION)]
        #[contracts::ensures(|_| CONDITION)]
        fn demo() {}
    };

    assert_fn_eq(&observed, &expected);
}

#[test]
fn ensures_from_expression_uses_generated_closure() {
    let spec: Spec = parse_quote! {
        ensures: CONDITION,
    };
    let func: ItemFn = parse_quote! { fn demo() {} };

    let observed = instrument_fn(&spec, func).expect("ensures should succeed");

    let expected: ItemFn = parse_quote! {
        #[contracts::ensures(|output| CONDITION)]
        fn demo() {}
    };

    assert_fn_eq(&observed, &expected);
}

#[test]
fn ensures_with_custom_closure_is_preserved() {
    let spec: Spec = parse_quote! {
        ensures: |result| result.is_ok(),
    };
    let func: ItemFn = parse_quote! { fn demo() {} };

    let observed = instrument_fn(&spec, func).expect("ensures should succeed");

    let expected: ItemFn = parse_quote! {
        #[contracts::ensures(|result| result.is_ok())]
        fn demo() {}
    };

    assert_fn_eq(&observed, &expected);
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

    let observed = instrument_fn(&spec, func).expect("requires should succeed");

    let expected: ItemFn = parse_quote! {
        #[contracts::requires(CONDITION)]
        #[inline]
        fn demo() {}
    };

    assert_fn_eq(&observed, &expected);
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
