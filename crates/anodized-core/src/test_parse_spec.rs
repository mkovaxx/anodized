use crate::test_util::assert_spec_eq;

use super::*;
use syn::parse_quote;

#[test]
fn test_parse_simple_spec() {
    let spec: Spec = parse_quote! {
        requires: is_valid(x),
        ensures: output > x,
    };

    let expected = Spec {
        requires: vec![parse_quote! { is_valid(x) }],
        maintains: vec![],
        clones: vec![],
        ensures: vec![parse_quote! { |output| output > x }],
    };

    assert_spec_eq(&spec, &expected);
}

#[test]
fn test_parse_all_clauses() {
    let spec: Spec = parse_quote! {
        requires: x > 0 && x.is_power_of_two(),
        maintains: self.is_valid(),
        binds: z,
        ensures: z >= x,
    };

    let expected = Spec {
        requires: vec![parse_quote! { x > 0 && x.is_power_of_two() }],
        maintains: vec![parse_quote! { self.is_valid() }],
        clones: vec![],
        ensures: vec![parse_quote! { |z| z >= x }],
    };

    assert_spec_eq(&spec, &expected);
}

#[test]
#[should_panic(expected = "parameters are out of order")]
fn test_parse_out_of_order() {
    let _: Spec = parse_quote! {
        ensures: output == x,
        requires: x > 0 && !is_zero(x),
    };
}

#[test]
#[should_panic(expected = "multiple `binds` parameters are not allowed")]
fn test_parse_multiple_binds() {
    let _: Spec = parse_quote! {
        binds: y,
        binds: z,
    };
}

#[test]
#[should_panic(
    expected = "at most one `clones` parameter is allowed; to clone multiple values, use a list"
)]
fn test_parse_multiple_clones() {
    let _: Spec = parse_quote! {
        clones: value,
        clones: count as old_count,
    };
}

#[test]
fn test_parse_array_of_conditions() {
    let spec: Spec = parse_quote! {
        requires: [
            x >= 0,
            y.len() < 10,
        ],
        ensures: [
            output != x,
            |output| output.is_some(),
        ],
    };

    let expected = Spec {
        requires: vec![parse_quote! { x >= 0 }, parse_quote! { y.len() < 10 }],
        maintains: vec![],
        clones: vec![],
        ensures: vec![
            parse_quote! { |output| output != x },
            parse_quote! { |output| output.is_some() },
        ],
    };

    assert_spec_eq(&spec, &expected);
}

#[test]
fn test_parse_ensures_with_closure() {
    let spec: Spec = parse_quote! {
        ensures: |result| result.is_ok() || result.unwrap_err().kind() == ErrorKind::NotFound,
    };

    let expected = Spec {
        requires: vec![],
        maintains: vec![],
        clones: vec![],
        ensures: vec![
            parse_quote! { |result| result.is_ok() || result.unwrap_err().kind() == ErrorKind::NotFound },
        ],
    };

    assert_spec_eq(&spec, &expected);
}

#[test]
fn test_parse_multiple_clauses_of_same_flavor() {
    let spec: Spec = parse_quote! {
        requires: x > 0 || x < -10,
        requires: y.is_ascii(),
        ensures: output < x,
        ensures: |output| output.len() >= y.len(),
    };

    let expected = Spec {
        requires: vec![
            parse_quote! { x > 0 || x < -10 },
            parse_quote! { y.is_ascii() },
        ],
        maintains: vec![],
        clones: vec![],
        ensures: vec![
            parse_quote! { |output| output < x },
            parse_quote! { |output| output.len() >= y.len() },
        ],
    };

    assert_spec_eq(&spec, &expected);
}

#[test]
fn test_parse_mixed_single_and_array_clauses() {
    let spec: Spec = parse_quote! {
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

    let expected = Spec {
        requires: vec![
            parse_quote! { x == 0 },
            parse_quote! { y > 1 },
            parse_quote! { z.is_empty() || z.contains("foo") },
        ],
        maintains: vec![],
        clones: vec![],
        ensures: vec![
            parse_quote! { |output| output != y },
            parse_quote! { |output| output.starts_with(z) },
            parse_quote! { |output| output.len() > x },
        ],
    };

    assert_spec_eq(&spec, &expected);
}

#[test]
fn test_parse_cfg_attributes() {
    let spec: Spec = parse_quote! {
        #[cfg(test)]
        requires: x > 0 && is_test_mode(),
        #[cfg(not(debug_assertions))]
        ensures: output < x,
    };

    let expected = Spec {
        requires: vec![Condition {
            expr: parse_quote! { x > 0 && is_test_mode() },
            cfg: Some(parse_quote! { test }),
        }],
        maintains: vec![],
        clones: vec![],
        ensures: vec![ConditionClosure {
            closure: parse_quote! { |output| output < x },
            cfg: Some(parse_quote! { not(debug_assertions) }),
        }],
    };

    assert_spec_eq(&spec, &expected);
}

#[test]
#[should_panic(expected = "unsupported attribute; only `cfg` is allowed")]
fn test_parse_non_cfg_attribute() {
    let _: Spec = parse_quote! {
        #[allow(dead_code)]
        requires: x > 0,
    };
}

#[test]
#[should_panic(expected = "multiple `cfg` attributes are not supported")]
fn test_parse_multiple_cfg_attributes() {
    let _: Spec = parse_quote! {
        #[cfg(test)]
        #[cfg(debug_assertions)]
        requires: x > 0,
    };
}

#[test]
#[should_panic(expected = "`cfg` attribute is not supported on `binds`")]
fn test_parse_cfg_on_binds() {
    let _: Spec = parse_quote! {
        #[cfg(test)]
        binds: y,
    };
}

#[test]
fn test_parse_macro_in_condition() {
    let spec: Spec = parse_quote! {
        requires: matches!(self.state, State::Idle),
        maintains: matches!(self.state, State::Idle | State::Running | State::Finished),
        ensures: matches!(self.state, State::Running),
    };

    let expected = Spec {
        requires: vec![parse_quote! { matches!(self.state, State::Idle) }],
        maintains: vec![
            parse_quote! { matches!(self.state, State::Idle | State::Running | State::Finished) },
        ],
        clones: vec![],
        ensures: vec![parse_quote! { |output| matches!(self.state, State::Running) }],
    };

    assert_spec_eq(&spec, &expected);
}

#[test]
fn test_parse_binds_pattern() {
    let spec: Spec = parse_quote! {
        binds: (a, b),
        ensures: [
            a <= b,
            (a, b) == pair || (b, a) == pair,
            |(a, b)| (a, b) == pair || (b, a) == pair,
        ],
    };

    let expected = Spec {
        requires: vec![],
        maintains: vec![],
        clones: vec![],
        ensures: vec![
            parse_quote! { |(a, b)| a <= b },
            parse_quote! { |(a, b)| (a, b) == pair || (b, a) == pair },
            parse_quote! { |(a, b)| (a, b) == pair || (b, a) == pair },
        ],
    };

    assert_spec_eq(&spec, &expected);
}

#[test]
fn test_parse_multiple_conditions() {
    let spec: Spec = parse_quote! {
        requires: [
            self.initialized,
            !self.locked,
        ],
        requires: index < self.items.len(),
        maintains: self.items.len() <= self.items.capacity(),
    };

    let expected = Spec {
        requires: vec![
            parse_quote! { self.initialized },
            parse_quote! { !self.locked },
            parse_quote! { index < self.items.len() },
        ],
        maintains: vec![parse_quote! { self.items.len() <= self.items.capacity() }],
        clones: vec![],
        ensures: vec![],
    };

    assert_spec_eq(&spec, &expected);
}

#[test]
fn test_parse_rename_return_value() {
    let spec: Spec = parse_quote! {
        binds: result,
        ensures: [
            result > output,
            |val| val % 2 == 0,
        ],
    };

    let expected = Spec {
        requires: vec![],
        maintains: vec![],
        clones: vec![],
        ensures: vec![
            parse_quote! { |result| result > output },
            parse_quote! { |val| val % 2 == 0 },
        ],
    };

    assert_spec_eq(&spec, &expected);
}

#[test]
fn test_parse_clones_simple_identifier() {
    let spec: Spec = parse_quote! {
        clones: count,
        ensures: output == old_count + 1,
    };

    let expected = Spec {
        requires: vec![],
        maintains: vec![],
        clones: vec![CloneBinding {
            expr: parse_quote! { count },
            alias: parse_quote! { old_count },
        }],
        ensures: vec![parse_quote! { |output| output == old_count + 1 }],
    };

    assert_spec_eq(&spec, &expected);
}

#[test]
fn test_parse_clones_identifier_with_alias() {
    let spec: Spec = parse_quote! {
        clones: value as prev_value,
        ensures: output > prev_value,
    };

    let expected = Spec {
        requires: vec![],
        maintains: vec![],
        clones: vec![CloneBinding {
            expr: parse_quote! { value },
            alias: parse_quote! { prev_value },
        }],
        ensures: vec![parse_quote! { |output| output > prev_value }],
    };

    assert_spec_eq(&spec, &expected);
}

#[test]
fn test_parse_clones_array() {
    let spec: Spec = parse_quote! {
        clones: [
            count,
            index as old_index,
            value as old_value,
        ],
        ensures: [
            count == old_count + 1,
            index == old_index + 1,
            value > old_value,
        ],
    };

    let expected = Spec {
        requires: vec![],
        maintains: vec![],
        clones: vec![
            CloneBinding {
                expr: parse_quote! { count },
                alias: parse_quote! { old_count },
            },
            CloneBinding {
                expr: parse_quote! { index },
                alias: parse_quote! { old_index },
            },
            CloneBinding {
                expr: parse_quote! { value },
                alias: parse_quote! { old_value },
            },
        ],
        ensures: vec![
            parse_quote! { |output| count == old_count + 1 },
            parse_quote! { |output| index == old_index + 1 },
            parse_quote! { |output| value > old_value },
        ],
    };

    assert_spec_eq(&spec, &expected);
}

#[test]
fn test_parse_clones_with_all_clauses() {
    let spec: Spec = parse_quote! {
        requires: x > 0,
        maintains: self.is_valid(),
        clones: value as old_val,
        binds: result,
        ensures: result > old_val,
    };

    let expected = Spec {
        requires: vec![parse_quote! { x > 0 }],
        maintains: vec![parse_quote! { self.is_valid() }],
        clones: vec![CloneBinding {
            expr: parse_quote! { value },
            alias: parse_quote! { old_val },
        }],
        ensures: vec![parse_quote! { |result| result > old_val }],
    };

    assert_spec_eq(&spec, &expected);
}

#[test]
#[should_panic(expected = "parameters are out of order")]
fn test_parse_clones_out_of_order() {
    let _: Spec = parse_quote! {
        clones: value,
        maintains: self.is_valid(),
    };
}

#[test]
fn test_parse_clones_array_expression() {
    let spec: Spec = parse_quote! {
        clones: [a, b, c] as slice,
        ensures: slice.len() == 3,
    };

    let expected = Spec {
        requires: vec![],
        maintains: vec![],
        clones: vec![CloneBinding {
            expr: parse_quote! { [a, b, c] },
            alias: parse_quote! { slice },
        }],
        ensures: vec![parse_quote! { |output| slice.len() == 3 }],
    };

    assert_spec_eq(&spec, &expected);
}

#[test]
#[should_panic(expected = "complex expressions require an explicit alias using `as`")]
fn test_parse_clones_complex_expr_without_alias() {
    let _: Spec = parse_quote! {
        clones: self.items.len(),
        ensures: output > 0,
    };
}

#[test]
#[should_panic(expected = "complex expressions require an explicit alias using `as`")]
fn test_parse_clones_method_call_without_alias() {
    let _: Spec = parse_quote! {
        clones: foo.bar(),
        ensures: output > 0,
    };
}

#[test]
#[should_panic(expected = "complex expressions require an explicit alias using `as`")]
fn test_parse_clones_binary_expr_without_alias() {
    let _: Spec = parse_quote! {
        clones: a + b,
        ensures: output > 0,
    };
}

#[test]
#[should_panic(expected = "complex expressions require an explicit alias using `as`")]
fn test_parse_clones_array_with_complex_expr_no_alias() {
    let _: Spec = parse_quote! {
        clones: [
            count,
            // This should fail - complex expression needs explicit alias
            self.items.len(),
        ],
        ensures: output > 0,
    };
}

#[test]
#[should_panic(expected = "`cfg` attribute is not supported on `clones`")]
fn test_parse_cfg_on_clones() {
    let _: Spec = parse_quote! {
        #[cfg(test)]
        clones: value as old_value,
        ensures: output > old_value,
    };
}

#[test]
fn test_parse_clones_edge_case_cast_exprs() {
    let spec: Spec = parse_quote! {
        clones: [
            r as u8,
            g as u8,
            b as u8,
        ] as r8g8b8,
        ensures: r8g8b8[0] == 0xFF,
    };

    let expected = Spec {
        requires: vec![],
        maintains: vec![],
        clones: vec![CloneBinding {
            expr: parse_quote! {
                [
                    r as u8,
                    g as u8,
                    b as u8,
                ]
            },
            alias: parse_quote! { r8g8b8 },
        }],
        ensures: vec![parse_quote! { |output| r8g8b8[0] == 0xFF }],
    };

    assert_spec_eq(&spec, &expected);
}
