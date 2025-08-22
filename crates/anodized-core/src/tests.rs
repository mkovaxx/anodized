use super::*;
use quote::quote;
use syn::parse2;

#[test]
fn test_parse_simple_contract() {
    let tokens = quote! {
        requires: x > 0,
        ensures: output > x,
    };
    let args: ContractArgs = parse2(tokens.into()).unwrap();
    let contract = Contract::try_from(args).unwrap();

    assert_eq!(contract.requires.len(), 1);
    assert_eq!(contract.maintains.len(), 0);
    assert_eq!(contract.ensures.len(), 1);
}

#[test]
fn test_parse_all_clauses() {
    let tokens = quote! {
        requires: x > 0,
        maintains: y.is_valid(),
        binds: z,
        ensures: z > x,
    };
    let args: ContractArgs = parse2(tokens.into()).unwrap();
    let contract = Contract::try_from(args).unwrap();

    assert_eq!(contract.requires.len(), 1);
    assert_eq!(contract.maintains.len(), 1);
    assert_eq!(contract.ensures.len(), 1);
}

#[test]
fn test_parse_out_of_order() {
    let tokens = quote! {
        ensures: output > x,
        requires: x > 0,
    };
    let args: ContractArgs = parse2(tokens.into()).unwrap();
    let result = Contract::try_from(args);
    assert!(result.is_err());
}

#[test]
fn test_parse_multiple_binds() {
    let tokens = quote! {
        binds: y,
        binds: z,
    };
    let args: ContractArgs = parse2(tokens.into()).unwrap();
    let result = Contract::try_from(args);
    assert!(result.is_err());
}

#[test]
fn test_parse_array_of_conditions() {
    let tokens = quote! {
        requires: [
            x > 0,
            y > 0,
        ],
        ensures: [
            output > x,
            output > y,
        ],
    };
    let args: ContractArgs = parse2(tokens.into()).unwrap();
    let contract = Contract::try_from(args).unwrap();

    assert_eq!(contract.requires.len(), 2);
    assert_eq!(contract.maintains.len(), 0);
    assert_eq!(contract.ensures.len(), 2);
}
