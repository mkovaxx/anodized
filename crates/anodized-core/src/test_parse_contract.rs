use crate::test_util::assert_contract_eq;

use super::*;
use syn::parse_quote;

#[test]
fn test_parse_simple_contract() {
    let contract: Contract = parse_quote! {
        requires: is_valid(x),
        ensures: output > x,
    };

    let expected = Contract {
        requires: vec![parse_quote! { is_valid(x) }],
        maintains: vec![],
        ensures: vec![parse_quote! { |output| output > x }],
    };

    assert_contract_eq(&contract, &expected);
}

#[test]
fn test_parse_all_clauses() {
    let contract: Contract = parse_quote! {
        requires: x > 0 && x.is_power_of_two(),
        maintains: self.is_valid(),
        binds: z,
        ensures: z >= x,
    };

    let expected = Contract {
        requires: vec![parse_quote! { x > 0 && x.is_power_of_two() }],
        maintains: vec![parse_quote! { self.is_valid() }],
        ensures: vec![parse_quote! { |z| z >= x }],
    };

    assert_contract_eq(&contract, &expected);
}

#[test]
#[should_panic(expected = "parameters are out of order")]
fn test_parse_out_of_order() {
    let _: Contract = parse_quote! {
        ensures: output == x,
        requires: x > 0 && !is_zero(x),
    };
}

#[test]
#[should_panic(expected = "multiple `binds` parameters are not allowed")]
fn test_parse_multiple_binds() {
    let _: Contract = parse_quote! {
        binds: y,
        binds: z,
    };
}

#[test]
fn test_parse_array_of_conditions() {
    let contract: Contract = parse_quote! {
        requires: [
            x >= 0,
            y.len() < 10,
        ],
        ensures: [
            output != x,
            |output| output.is_some(),
        ],
    };

    let expected = Contract {
        requires: vec![
            parse_quote! { x >= 0 },
            parse_quote! { y.len() < 10 },
        ],
        maintains: vec![],
        ensures: vec![
            parse_quote! { |output| output != x },
            parse_quote! { |output| output.is_some() },
        ],
    };

    assert_contract_eq(&contract, &expected);
}

#[test]
fn test_parse_ensures_with_closure() {
    let contract: Contract = parse_quote! {
        ensures: |result| result.is_ok() || result.unwrap_err().kind() == ErrorKind::NotFound,
    };

    let expected = Contract {
        requires: vec![],
        maintains: vec![],
        ensures: vec![
            parse_quote! { |result| result.is_ok() || result.unwrap_err().kind() == ErrorKind::NotFound },
        ],
    };

    assert_contract_eq(&contract, &expected);
}

#[test]
fn test_parse_multiple_clauses_of_same_flavor() {
    let contract: Contract = parse_quote! {
        requires: x > 0 || x < -10,
        requires: y.is_ascii(),
        ensures: output < x,
        ensures: |output| output.len() >= y.len(),
    };

    let expected = Contract {
        requires: vec![
            parse_quote! { x > 0 || x < -10 },
            parse_quote! { y.is_ascii() },
        ],
        maintains: vec![],
        ensures: vec![
            parse_quote! { |output| output < x },
            parse_quote! { |output| output.len() >= y.len() },
        ],
    };

    assert_contract_eq(&contract, &expected);
}

#[test]
fn test_parse_mixed_single_and_array_clauses() {
    let contract: Contract = parse_quote! {
        requires: x == 0,
        requires: [
            y > 1,
            z.is_empty() || z.contains("foo"),
        ],
        ensures: [
            output != y,
            |output| output.starts_with(z),
        ],
        ensures: output.len() > x,
    };

    let expected = Contract {
        requires: vec![
            parse_quote! { x == 0 },
            parse_quote! { y > 1 },
            parse_quote! { z.is_empty() || z.contains("foo") },
        ],
        maintains: vec![],
        ensures: vec![
            parse_quote! { |output| output != y },
            parse_quote! { |output| output.starts_with(z) },
            parse_quote! { |output| output.len() > x },
        ],
    };

    assert_contract_eq(&contract, &expected);
}

#[test]
fn test_parse_cfg_attributes() {
    let contract: Contract = parse_quote! {
        #[cfg(test)]
        requires: x > 0 && is_test_mode(),
        #[cfg(not(debug_assertions))]
        ensures: output < x,
    };

    let expected = Contract {
        requires: vec![Condition {
            expr: parse_quote! { x > 0 && is_test_mode() },
            cfg: Some(parse_quote! { test }),
        }],
        maintains: vec![],
        ensures: vec![ConditionClosure {
            closure: parse_quote! { |output| output < x },
            cfg: Some(parse_quote! { not(debug_assertions) }),
        }],
    };

    assert_contract_eq(&contract, &expected);
}

#[test]
#[should_panic(expected = "unsupported attribute; only `cfg` is allowed")]
fn test_parse_non_cfg_attribute() {
    let _: Contract = parse_quote! {
        #[allow(dead_code)]
        requires: x > 0,
    };
}

#[test]
#[should_panic(expected = "multiple `cfg` attributes are not supported")]
fn test_parse_multiple_cfg_attributes() {
    let _: Contract = parse_quote! {
        #[cfg(test)]
        #[cfg(debug_assertions)]
        requires: x > 0,
    };
}

#[test]
#[should_panic(expected = "`cfg` attribute is not supported on `binds`")]
fn test_parse_cfg_on_binds() {
    let _: Contract = parse_quote! {
        #[cfg(test)]
        binds: y,
    };
}

#[test]
fn test_parse_macro_in_condition() {
    let contract: Contract = parse_quote! {
        requires: matches!(self.state, State::Idle),
        maintains: matches!(self.state, State::Idle | State::Running | State::Finished),
        ensures: matches!(self.state, State::Running),
    };

    let expected = Contract {
        requires: vec![parse_quote! { matches!(self.state, State::Idle) }],
        maintains: vec![
            parse_quote! { matches!(self.state, State::Idle | State::Running | State::Finished) },
        ],
        ensures: vec![parse_quote! { |output| matches!(self.state, State::Running) }],
    };

    assert_contract_eq(&contract, &expected);
}

#[test]
fn test_parse_binds_pattern() {
    let contract: Contract = parse_quote! {
        binds: (a, b),
        ensures: [
            a <= b,
            (a, b) == pair || (b, a) == pair,
            |(a, b)| (a, b) == pair || (b, a) == pair,
        ],
    };

    let expected = Contract {
        requires: vec![],
        maintains: vec![],
        ensures: vec![
            parse_quote! { |(a, b)| a <= b },
            parse_quote! { |(a, b)| (a, b) == pair || (b, a) == pair },
            parse_quote! { |(a, b)| (a, b) == pair || (b, a) == pair },
        ],
    };

    assert_contract_eq(&contract, &expected);
}

#[test]
fn test_parse_multiple_conditions() {
    let contract: Contract = parse_quote! {
        requires: [
            self.initialized,
            !self.locked,
        ],
        requires: index < self.items.len(),
        maintains: self.items.len() <= self.items.capacity(),
    };

    let expected = Contract {
        requires: vec![
            parse_quote! { self.initialized },
            parse_quote! { !self.locked },
            parse_quote! { index < self.items.len() },
        ],
        maintains: vec![parse_quote! { self.items.len() <= self.items.capacity() }],
        ensures: vec![],
    };

    assert_contract_eq(&contract, &expected);
}

#[test]
fn test_parse_rename_return_value() {
    let contract: Contract = parse_quote! {
        binds: result,
        ensures: [
            result > output,
            |val| val % 2 == 0,
        ],
    };

    let expected = Contract {
        requires: vec![],
        maintains: vec![],
        ensures: vec![
            parse_quote! { |result| result > output },
            parse_quote! { |val| val % 2 == 0 },
        ],
    };

    assert_contract_eq(&contract, &expected);
}
