use super::*;
use quote::quote;
use syn::{parse2, parse_quote};

fn parse_contract(tokens: proc_macro2::TokenStream) -> Result<Contract> {
    let args: ContractArgs = parse2(tokens)?;
    Contract::try_from(args)
}

#[test]
fn test_parse_simple_contract() {
    let contract = parse_contract(quote! {
        requires: x > 0,
        ensures: output > x,
    })
    .unwrap();

    let expected = Contract {
        requires: vec![parse_quote! { x > 0 }],
        maintains: vec![],
        ensures: vec![parse_quote! { |output| output > x }],
    };

    assert_eq!(contract, expected);
}

#[test]
fn test_parse_all_clauses() {
    let contract = parse_contract(quote! {
        requires: x > 0,
        maintains: y.is_valid(),
        binds: z,
        ensures: z > x,
    })
    .unwrap();

    let expected = Contract {
        requires: vec![parse_quote! { x > 0 }],
        maintains: vec![parse_quote! { y.is_valid() }],
        ensures: vec![parse_quote! { |z| z > x }],
    };

    assert_eq!(contract, expected);
}

#[test]
fn test_parse_out_of_order() {
    let result = parse_contract(quote! {
        ensures: output > x,
        requires: x > 0,
    });
    assert!(result.is_err());
}

#[test]
fn test_parse_multiple_binds() {
    let result = parse_contract(quote! {
        binds: y,
        binds: z,
    });
    assert!(result.is_err());
}

#[test]
fn test_parse_array_of_conditions() {
    let contract = parse_contract(quote! {
        requires: [
            x > 0,
            y > 0,
        ],
        ensures: [
            output > x,
            |output| output > y,
        ],
    })
    .unwrap();

    let expected = Contract {
        requires: vec![
            parse_quote! { x > 0 },
            parse_quote! { y > 0 },
        ],
        maintains: vec![],
        ensures: vec![
            parse_quote! { |output| output > x },
            parse_quote! { |output| output > y },
        ],
    };

    assert_eq!(contract, expected);
}

#[test]
fn test_parse_ensures_with_closure() {
    let contract = parse_contract(quote! {
        ensures: |result| result.is_ok(),
    })
    .unwrap();

    let expected = Contract {
        requires: vec![],
        maintains: vec![],
        ensures: vec![parse_quote! { |result| result.is_ok() }],
    };

    assert_eq!(contract, expected);
}