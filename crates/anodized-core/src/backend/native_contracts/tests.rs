use super::*;

use crate::test_util::assert_tokens_eq;
use syn::{ItemFn, parse_quote};

#[test]
fn requires_clause_emits_contracts_attribute() {
    let spec: Spec = parse_quote! {
        requires: CONDITION,
    };
    let func: ItemFn = parse_quote! { fn demo() {} };

    let expected: ItemFn = parse_quote! {
        #[core::contracts::requires(CONDITION)]
        #func
    };

    let observed = instrument_fn(&spec, func).unwrap();

    assert_tokens_eq(&observed, &expected);
}

#[test]
fn requires_with_cfg_emits_cfg_attr_contracts_attribute() {
    let spec: Spec = parse_quote! {
        #[cfg(SETTING)]
        requires: CONDITION,
    };
    let func: ItemFn = parse_quote! { fn demo() {} };

    let expected: ItemFn = parse_quote! {
        #[cfg_attr(SETTING, core::contracts::requires(CONDITION))]
        #func
    };

    let observed = instrument_fn(&spec, func).unwrap();

    assert_tokens_eq(&observed, &expected);
}

#[test]
fn maintains_emits_requires_and_ensures_attributes() {
    let spec: Spec = parse_quote! {
        maintains: CONDITION,
    };
    let func: ItemFn = parse_quote! { fn demo() {} };

    let expected: ItemFn = parse_quote! {
        #[core::contracts::requires(CONDITION)]
        #[core::contracts::ensures(|_| CONDITION)]
        #func
    };

    let observed = instrument_fn(&spec, func).unwrap();

    assert_tokens_eq(&observed, &expected);
}

#[test]
fn ensures_from_expression_uses_generated_closure() {
    let spec: Spec = parse_quote! {
        binds: PATTERN_1,
        ensures: CONDITION_1,
        ensures: |PATTERN_2| CONDITION_2,
    };
    let func: ItemFn = parse_quote! { fn demo() {} };

    let expected: ItemFn = parse_quote! {
        #[core::contracts::ensures(|PATTERN_1| CONDITION_1)]
        #[core::contracts::ensures(|PATTERN_2| CONDITION_2)]
        #func
    };

    let observed = instrument_fn(&spec, func).unwrap();

    assert_tokens_eq(&observed, &expected);
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

    let expected: ItemFn = parse_quote! {
        #[core::contracts::requires(CONDITION)]
        #func
    };

    let observed = instrument_fn(&spec, func).unwrap();

    assert_tokens_eq(&observed, &expected);
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
