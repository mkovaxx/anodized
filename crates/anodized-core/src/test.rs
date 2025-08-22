use super::*;
use crate::test_util::assert_contract_eq;
use quote::quote;
use syn::{parse_quote, parse2};

fn parse_contract(tokens: proc_macro2::TokenStream) -> Result<Contract> {
    let args: ContractArgs = parse2(tokens)?;
    Contract::try_from(args)
}

#[test]
fn test_parse_simple_contract() -> Result<()> {
    let contract = parse_contract(quote! {
        requires: x > 0,
        ensures: output > x,
    })?;

    let expected = Contract {
        requires: vec![parse_quote! { x > 0 }],
        maintains: vec![],
        ensures: vec![parse_quote! { |output| output > x }],
    };

    assert_contract_eq(&contract, &expected);

    Ok(())
}

#[test]
fn test_parse_all_clauses() -> Result<()> {
    let contract = parse_contract(quote! {
        requires: x > 0,
        maintains: y.is_valid(),
        binds: z,
        ensures: z > x,
    })?;

    let expected = Contract {
        requires: vec![parse_quote! { x > 0 }],
        maintains: vec![parse_quote! { y.is_valid() }],
        ensures: vec![parse_quote! { |z| z > x }],
    };

    assert_contract_eq(&contract, &expected);

    Ok(())
}

#[test]
fn test_parse_out_of_order() -> Result<()> {
    let result = parse_contract(quote! {
        ensures: output > x,
        requires: x > 0,
    });
    assert!(result.is_err());

    Ok(())
}

#[test]
fn test_parse_multiple_binds() -> Result<()> {
    let result = parse_contract(quote! {
        binds: y,
        binds: z,
    });
    assert!(result.is_err());

    Ok(())
}

#[test]
fn test_parse_array_of_conditions() -> Result<()> {
    let contract = parse_contract(quote! {
        requires: [
            x > 0,
            y > 0,
        ],
        ensures: [
            output > x,
            |output| output > y,
        ],
    })?;

    let expected = Contract {
        requires: vec![parse_quote! { x > 0 }, parse_quote! { y > 0 }],
        maintains: vec![],
        ensures: vec![
            parse_quote! { |output| output > x },
            parse_quote! { |output| output > y },
        ],
    };

    assert_contract_eq(&contract, &expected);

    Ok(())
}

#[test]
fn test_parse_ensures_with_closure() -> Result<()> {
    let contract = parse_contract(quote! {
        ensures: |result| result.is_ok(),
    })?;

    let expected = Contract {
        requires: vec![],
        maintains: vec![],
        ensures: vec![parse_quote! { |result| result.is_ok() }],
    };

    assert_contract_eq(&contract, &expected);

    Ok(())
}

#[test]
fn test_parse_multiple_clauses_of_same_flavor() -> Result<()> {
    let contract = parse_contract(quote! {
        requires: x > 0,
        requires: y > 0,
        ensures: output > x,
        ensures: |output| output > y,
    })?;

    let expected = Contract {
        requires: vec![parse_quote! { x > 0 }, parse_quote! { y > 0 }],
        maintains: vec![],
        ensures: vec![
            parse_quote! { |output| output > x },
            parse_quote! { |output| output > y },
        ],
    };

    assert_contract_eq(&contract, &expected);

    Ok(())
}

#[test]
fn test_parse_mixed_single_and_array_clauses() -> Result<()> {
    let contract = parse_contract(quote! {
        requires: x > 0,
        requires: [
            y > 1,
            z > 2,
        ],
        ensures: [
            output > y,
            |output| output > z,
        ],
        ensures: output > x,
    })?;

    let expected = Contract {
        requires: vec![
            parse_quote! { x > 0 },
            parse_quote! { y > 1 },
            parse_quote! { z > 2 },
        ],
        maintains: vec![],
        ensures: vec![
            parse_quote! { |output| output > y },
            parse_quote! { |output| output > z },
            parse_quote! { |output| output > x },
        ],
    };

    assert_contract_eq(&contract, &expected);

    Ok(())
}

#[test]
fn test_parse_cfg_attributes() -> Result<()> {
    let contract = parse_contract(quote! {
        #[cfg(test)]
        requires: x > 0,
        #[cfg(not(debug_assertions))]
        ensures: output > x,
    })?;

    let expected = Contract {
        requires: vec![Condition {
            expr: parse_quote! { x > 0 },
            cfg: Some(parse_quote! { test }),
        }],
        maintains: vec![],
        ensures: vec![ConditionClosure {
            closure: parse_quote! { |output| output > x },
            cfg: Some(parse_quote! { not(debug_assertions) }),
        }],
    };

    assert_contract_eq(&contract, &expected);

    Ok(())
}

#[test]
fn test_parse_non_cfg_attribute() -> Result<()> {
    let error = parse_contract(quote! {
        #[allow(dead_code)]
        requires: x > 0,
    })
    .expect_err("parsing should have failed but it succeeded");

    assert_eq!(
        error.to_string(),
        "unsupported attribute; only `cfg` is allowed"
    );

    Ok(())
}
