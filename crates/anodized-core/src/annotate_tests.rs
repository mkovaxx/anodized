use crate::test_util::assert_spec_eq;

use super::*;
use proc_macro2::Span;
use syn::{parse_quote, parse_str};

#[test]
fn simple_spec() {
    let spec: Spec = parse_quote! {
        requires: is_valid(x),
        ensures: output > x,
    };

    let expected = Spec {
        qualifiers: FnQualifiers::empty(),
        requires: vec![Condition {
            expr: parse_quote! { is_valid(x) },
            cfg: None,
        }],
        maintains: vec![],
        captures: vec![],
        inspects: None,
        ensures: vec![PostCondition {
            pat: None,
            expr: parse_quote! { output > x },
            cfg: None,
        }],
        span: Span::call_site(),
    };

    assert_spec_eq(&spec, &expected);
}

#[test]
fn fn_qualifiers_functional() {
    let spec: Spec = parse_quote! {
        functional
    };

    let expected = Spec {
        qualifiers: FnQualifiers::FUNCTIONAL,
        requires: vec![],
        maintains: vec![],
        captures: vec![],
        inspects: None,
        ensures: vec![],
        span: Span::call_site(),
    };

    assert_spec_eq(&spec, &expected);
}

#[test]
fn fn_qualifiers_pure_total() {
    let spec: Spec = parse_quote! {
        pure,
        total,
    };

    let expected = Spec {
        qualifiers: FnQualifiers::PURE | FnQualifiers::TOTAL,
        requires: vec![],
        maintains: vec![],
        captures: vec![],
        inspects: None,
        ensures: vec![],
        span: Span::call_site(),
    };

    assert_spec_eq(&spec, &expected);
}

#[test]
fn fn_qualifiers_deterministic_effectfree_infallible_terminating() {
    let spec: Spec = parse_quote! {
        deterministic,
        effectfree,
        infallible,
        terminating,
    };

    let expected = Spec {
        qualifiers: FnQualifiers::DETERMINISTIC
            | FnQualifiers::EFFECTFREE
            | FnQualifiers::INFALLIBLE
            | FnQualifiers::TERMINATING,
        requires: vec![],
        maintains: vec![],
        captures: vec![],
        inspects: None,
        ensures: vec![],
        span: Span::call_site(),
    };

    assert_spec_eq(&spec, &expected);
}

#[test]
#[should_panic = "expected `,`"]
fn fn_qualifiers_typo_missing_comma() {
    let _: Spec = parse_quote! {
        pure total,
    };
}

#[test]
#[should_panic = "qualifier `pure` does not take a value"]
fn fn_qualifiers_typo_colon_instead_of_comma() {
    let _: Spec = parse_quote! {
        pure: total,
    };
}

#[test]
#[should_panic = "expected an expression or a pattern"]
fn fn_qualifiers_invalid_colon() {
    let _: Spec = parse_quote! {
        pure:,
    };
}

#[test]
#[should_panic = "qualifier `functional` does not take a value"]
fn fn_qualifiers_invalid_value_expr() {
    let _: Spec = parse_quote! {
        functional: x == 42,
    };
}

#[test]
#[should_panic = "this qualifier is redundant; remove it"]
fn fn_qualifiers_functional_pure() {
    let _: Spec = parse_quote! {
        functional,
        pure,
    };
}

#[test]
#[should_panic = "this qualifier is redundant; remove it"]
fn fn_qualifiers_functional_total() {
    let _: Spec = parse_quote! {
        functional,
        total,
    };
}

#[test]
#[should_panic = "this qualifier is redundant; remove it"]
fn fn_qualifiers_pure_deterministic() {
    let _: Spec = parse_quote! {
        pure,
        deterministic,
    };
}

#[test]
#[should_panic = "this qualifier is redundant; remove it"]
fn fn_qualifiers_pure_effectfree() {
    let _: Spec = parse_quote! {
        pure,
        effectfree,
    };
}

#[test]
#[should_panic = "this qualifier is redundant; remove it"]
fn fn_qualifiers_total_infallible() {
    let _: Spec = parse_quote! {
        total,
        infallible,
    };
}

#[test]
#[should_panic = "this qualifier is redundant; remove it"]
fn fn_qualifiers_total_terminating() {
    let _: Spec = parse_quote! {
        total,
        terminating,
    };
}

#[test]
fn all_clauses() {
    let spec: Spec = parse_quote! {
        requires: x > 0 && x.is_power_of_two(),
        maintains: self.is_valid(),
        inspects: z,
        ensures: z >= x,
    };

    let expected = Spec {
        qualifiers: FnQualifiers::empty(),
        requires: vec![Condition {
            expr: parse_quote! { x > 0 && x.is_power_of_two() },
            cfg: None,
        }],
        maintains: vec![Condition {
            expr: parse_quote! { self.is_valid() },
            cfg: None,
        }],
        captures: vec![],
        inspects: Some(parse_quote! { z }),
        ensures: vec![PostCondition {
            pat: None,
            expr: parse_quote! { z >= x },
            cfg: None,
        }],
        span: Span::call_site(),
    };

    assert_spec_eq(&spec, &expected);
}

#[test]
#[should_panic(expected = "unknown spec keyword `goat`")]
fn unknown_keyword() {
    let _: Spec = parse_quote! {
        ensures: output == x,
        goat: 42,
        requires: x > 0 && !is_zero(x),
    };
}

#[test]
#[should_panic(expected = "parameters are out of order")]
fn out_of_order() {
    let _: Spec = parse_quote! {
        ensures: output == x,
        requires: x > 0 && !is_zero(x),
    };
}

#[test]
#[should_panic(expected = "multiple `inspects` parameters are not allowed")]
fn multiple_binds() {
    let _: Spec = parse_quote! {
        inspects: y,
        inspects: z,
    };
}

#[test]
#[should_panic(
    expected = "at most one `captures` parameter is allowed; to capture multiple values, use a list"
)]
fn multiple_captures() {
    let _: Spec = parse_quote! {
        captures: old_value = value,
        captures: old_count = count,
    };
}

#[test]
fn array_of_conditions() {
    let spec: Spec = parse_quote! {
        requires: [
            x >= 0,
            y.len() < 10,
        ],
        ensures: [
            output != x,
            |retval| retval.is_some(),
        ],
    };

    let expected = Spec {
        qualifiers: FnQualifiers::empty(),
        requires: vec![
            Condition {
                expr: parse_quote! { x >= 0 },
                cfg: None,
            },
            Condition {
                expr: parse_quote! { y.len() < 10 },
                cfg: None,
            },
        ],
        maintains: vec![],
        captures: vec![],
        inspects: None,
        ensures: vec![
            PostCondition {
                pat: None,
                expr: parse_quote! { output != x },
                cfg: None,
            },
            PostCondition {
                pat: Some(parse_quote! { retval }),
                expr: parse_quote! { retval.is_some() },
                cfg: None,
            },
        ],
        span: Span::call_site(),
    };

    assert_spec_eq(&spec, &expected);
}

#[test]
fn ensures_with_explicit_closure() {
    let spec: Spec = parse_quote! {
        ensures: |result| result.is_ok() || result.unwrap_err().kind() == ErrorKind::NotFound,
    };

    let expected = Spec {
        qualifiers: FnQualifiers::empty(),
        requires: vec![],
        maintains: vec![],
        captures: vec![],
        inspects: None,
        ensures: vec![PostCondition {
            pat: Some(parse_quote! { result }),
            expr: parse_quote! { result.is_ok() || result.unwrap_err().kind() == ErrorKind::NotFound },
            cfg: None,
        }],
        span: Span::call_site(),
    };

    assert_spec_eq(&spec, &expected);
}

#[test]
fn multiple_clauses_of_same_flavor() {
    let spec: Spec = parse_quote! {
        requires: x > 0 || x < -10,
        requires: y.is_ascii(),
        ensures: retval < x,
        ensures: |output| output.len() >= y.len(),
    };

    let expected = Spec {
        qualifiers: FnQualifiers::empty(),
        requires: vec![
            Condition {
                expr: parse_quote! { x > 0 || x < -10 },
                cfg: None,
            },
            Condition {
                expr: parse_quote! { y.is_ascii() },
                cfg: None,
            },
        ],
        maintains: vec![],
        captures: vec![],
        inspects: None,
        ensures: vec![
            PostCondition {
                pat: None,
                expr: parse_quote! { retval < x },
                cfg: None,
            },
            PostCondition {
                pat: Some(parse_quote! { output }),
                expr: parse_quote! { output.len() >= y.len() },
                cfg: None,
            },
        ],
        span: Span::call_site(),
    };

    assert_spec_eq(&spec, &expected);
}

#[test]
fn mixed_single_and_array_clauses() {
    let spec: Spec = parse_quote! {
        requires: x == 0,
        requires: [
            y > 1,
            z.is_empty() || z.contains("foo"),
        ],
        ensures: [
            output != y,
            |val| output.starts_with(z),
        ],
        ensures: retval.len() > x,
    };

    let expected = Spec {
        qualifiers: FnQualifiers::empty(),
        requires: vec![
            Condition {
                expr: parse_quote! { x == 0 },
                cfg: None,
            },
            Condition {
                expr: parse_quote! { y > 1 },
                cfg: None,
            },
            Condition {
                expr: parse_quote! { z.is_empty() || z.contains("foo") },
                cfg: None,
            },
        ],
        maintains: vec![],
        captures: vec![],
        inspects: None,
        ensures: vec![
            PostCondition {
                pat: None,
                expr: parse_quote! { output != y },
                cfg: None,
            },
            PostCondition {
                pat: Some(parse_quote! { val }),
                expr: parse_quote! { output.starts_with(z) },
                cfg: None,
            },
            PostCondition {
                pat: None,
                expr: parse_quote! { retval.len() > x },
                cfg: None,
            },
        ],
        span: Span::call_site(),
    };

    assert_spec_eq(&spec, &expected);
}

#[test]
fn cfg_attributes() {
    let spec: Spec = parse_quote! {
        #[cfg(test)]
        requires: x > 0 && is_mode(),
        #[cfg(not(debug_assertions))]
        ensures: output < x,
    };

    let expected = Spec {
        qualifiers: FnQualifiers::empty(),
        requires: vec![Condition {
            expr: parse_quote! { x > 0 && is_mode() },
            cfg: Some(parse_quote! { test }),
        }],
        maintains: vec![],
        captures: vec![],
        inspects: None,
        ensures: vec![PostCondition {
            pat: None,
            expr: parse_quote! { output < x },
            cfg: Some(parse_quote! { not(debug_assertions) }),
        }],
        span: Span::call_site(),
    };

    assert_spec_eq(&spec, &expected);
}

#[test]
#[should_panic(expected = "unsupported attribute; only `cfg` is allowed")]
fn non_cfg_attribute() {
    let _: Spec = parse_quote! {
        #[allow(dead_code)]
        requires: x > 0,
    };
}

#[test]
#[should_panic(expected = "multiple `cfg` attributes are not supported")]
fn multiple_cfg_attributes() {
    let _: Spec = parse_quote! {
        #[cfg(test)]
        #[cfg(debug_assertions)]
        requires: x > 0,
    };
}

#[test]
#[should_panic(expected = "`cfg` attribute is not supported on `inspects`")]
fn cfg_on_binds() {
    let _: Spec = parse_quote! {
        #[cfg(test)]
        inspects: y,
    };
}

#[test]
fn macro_in_condition() {
    let spec: Spec = parse_quote! {
        requires: matches!(self.state, State::Idle),
        maintains: matches!(self.state, State::Idle | State::Running | State::Finished),
        ensures: matches!(self.state, State::Running),
    };

    let expected = Spec {
        qualifiers: FnQualifiers::empty(),
        requires: vec![Condition {
            expr: parse_quote! { matches!(self.state, State::Idle) },
            cfg: None,
        }],
        maintains: vec![Condition {
            expr: parse_quote! { matches!(self.state, State::Idle | State::Running | State::Finished) },
            cfg: None,
        }],
        captures: vec![],
        inspects: None,
        ensures: vec![PostCondition {
            pat: None,
            expr: parse_quote! { matches!(self.state, State::Running) },
            cfg: None,
        }],
        span: Span::call_site(),
    };

    assert_spec_eq(&spec, &expected);
}

#[test]
fn binds_pattern() {
    let spec: Spec = parse_quote! {
        inspects: (a, b),
        ensures: [
            a <= b,
            (a, b) == pair || (b, a) == pair,
        ],
    };

    let expected = Spec {
        qualifiers: FnQualifiers::empty(),
        requires: vec![],
        maintains: vec![],
        captures: vec![],
        inspects: Some(parse_quote! { (a, b) }),
        ensures: vec![
            PostCondition {
                pat: None,
                expr: parse_quote! { a <= b },
                cfg: None,
            },
            PostCondition {
                pat: None,
                expr: parse_quote! { (a, b) == pair || (b, a) == pair },
                cfg: None,
            },
        ],
        span: Span::call_site(),
    };

    assert_spec_eq(&spec, &expected);
}

#[test]
fn multiple_conditions() {
    let spec: Spec = parse_quote! {
        requires: [
            self.initialized,
            !self.locked,
        ],
        requires: index < self.items.len(),
        maintains: self.items.len() <= self.items.capacity(),
    };

    let expected = Spec {
        qualifiers: FnQualifiers::empty(),
        requires: vec![
            Condition {
                expr: parse_quote! { self.initialized },
                cfg: None,
            },
            Condition {
                expr: parse_quote! { !self.locked },
                cfg: None,
            },
            Condition {
                expr: parse_quote! { index < self.items.len() },
                cfg: None,
            },
        ],
        maintains: vec![Condition {
            expr: parse_quote! { self.items.len() <= self.items.capacity() },
            cfg: None,
        }],
        captures: vec![],
        inspects: None,
        ensures: vec![],
        span: Span::call_site(),
    };

    assert_spec_eq(&spec, &expected);
}

#[test]
fn rename_return_value() {
    let spec: Spec = parse_quote! {
        inspects: result,
        ensures: [
            result > output,
            |val| val % 2 == 0,
        ],
    };

    let expected = Spec {
        qualifiers: FnQualifiers::empty(),
        requires: vec![],
        maintains: vec![],
        captures: vec![],
        inspects: Some(parse_quote! { result }),
        ensures: vec![
            PostCondition {
                pat: None,
                expr: parse_quote! { result > output },
                cfg: None,
            },
            PostCondition {
                pat: Some(parse_quote! { val }),
                expr: parse_quote! { val % 2 == 0 },
                cfg: None,
            },
        ],
        span: Span::call_site(),
    };

    assert_spec_eq(&spec, &expected);
}

#[test]
fn captures_simple_identifier() {
    let spec: Spec = parse_quote! {
        captures: old_count = count,
        ensures: output == old_count + 1,
    };

    let expected = Spec {
        qualifiers: FnQualifiers::empty(),
        requires: vec![],
        maintains: vec![],
        captures: vec![Capture {
            pat: parse_quote! { old_count },
            expr: parse_quote! { count },
        }],
        inspects: None,
        ensures: vec![PostCondition {
            pat: None,
            expr: parse_quote! { output == old_count + 1 },
            cfg: None,
        }],
        span: Span::call_site(),
    };

    assert_spec_eq(&spec, &expected);
}

#[test]
fn captures_identifier_with_alias() {
    let spec: Spec = parse_quote! {
        captures: prev_value = value,
        ensures: output > prev_value,
    };

    let expected = Spec {
        qualifiers: FnQualifiers::empty(),
        requires: vec![],
        maintains: vec![],
        captures: vec![Capture {
            pat: parse_quote! { prev_value },
            expr: parse_quote! { value },
        }],
        inspects: None,
        ensures: vec![PostCondition {
            pat: None,
            expr: parse_quote! { output > prev_value },
            cfg: None,
        }],
        span: Span::call_site(),
    };

    assert_spec_eq(&spec, &expected);
}

#[test]
fn captures_array() {
    let spec: Spec = parse_quote! {
        captures: [
            old_count = count,
            old_index = index,
            old_value = value,
        ],
        ensures: [
            count == old_count + 1,
            index == old_index + 1,
            value > old_value,
        ],
    };

    let expected = Spec {
        qualifiers: FnQualifiers::empty(),
        requires: vec![],
        maintains: vec![],
        captures: vec![
            Capture {
                pat: parse_quote! { old_count },
                expr: parse_quote! { count },
            },
            Capture {
                pat: parse_quote! { old_index },
                expr: parse_quote! { index },
            },
            Capture {
                pat: parse_quote! { old_value },
                expr: parse_quote! { value },
            },
        ],
        inspects: None,
        ensures: vec![
            PostCondition {
                pat: None,
                expr: parse_quote! { count == old_count + 1 },
                cfg: None,
            },
            PostCondition {
                pat: None,
                expr: parse_quote! { index == old_index + 1 },
                cfg: None,
            },
            PostCondition {
                pat: None,
                expr: parse_quote! { value > old_value },
                cfg: None,
            },
        ],
        span: Span::call_site(),
    };

    assert_spec_eq(&spec, &expected);
}

#[test]
fn captures_with_all_clauses() {
    let spec: Spec = parse_quote! {
        requires: x > 0,
        maintains: self.is_valid(),
        captures: old_val = value,
        inspects: result,
        ensures: result > old_val,
    };

    let expected = Spec {
        qualifiers: FnQualifiers::empty(),
        requires: vec![Condition {
            expr: parse_quote! { x > 0 },
            cfg: None,
        }],
        maintains: vec![Condition {
            expr: parse_quote! { self.is_valid() },
            cfg: None,
        }],
        captures: vec![Capture {
            pat: parse_quote! { old_val },
            expr: parse_quote! { value },
        }],
        inspects: Some(parse_quote! { result }),
        ensures: vec![PostCondition {
            pat: None,
            expr: parse_quote! { result > old_val },
            cfg: None,
        }],
        span: Span::call_site(),
    };

    assert_spec_eq(&spec, &expected);
}

#[test]
#[should_panic(expected = "parameters are out of order")]
fn captures_out_of_order() {
    let _: Spec = parse_quote! {
        captures: old_value = value,
        maintains: self.is_valid(),
    };
}

#[test]
fn captures_array_expression() {
    let spec: Spec = parse_quote! {
        captures: slice = [a, b, c],
        ensures: slice.len() == 3,
    };

    let expected = Spec {
        qualifiers: FnQualifiers::empty(),
        requires: vec![],
        maintains: vec![],
        captures: vec![Capture {
            pat: parse_quote! { slice },
            expr: parse_quote! { [a, b, c] },
        }],
        inspects: None,
        ensures: vec![PostCondition {
            pat: None,
            expr: parse_quote! { slice.len() == 3 },
            cfg: None,
        }],
        span: Span::call_site(),
    };

    assert_spec_eq(&spec, &expected);
}

#[test]
fn captures_complex_expressions() {
    let spec: Spec = parse_quote! {
        captures: [
            item_count = self.items.len(),
            bar = foo.bar(),
            sum = a + b,
            first = foo[0],
        ],
        ensures: output > 0,
    };

    assert_eq!(spec.captures.len(), 4);
}

#[test]
#[should_panic(expected = "`cfg` attribute is not supported on `captures`")]
fn cfg_on_captures() {
    let _: Spec = parse_quote! {
        #[cfg(test)]
        captures: old_value = value,
        ensures: output > old_value,
    };
}

#[test]
fn captures_edge_case_cast_expr() {
    let spec: Spec = parse_quote! {
        captures: old_red = r as u8,
    };

    let expected = Spec {
        qualifiers: FnQualifiers::empty(),
        requires: vec![],
        maintains: vec![],
        captures: vec![Capture {
            pat: parse_quote! { old_red },
            expr: parse_quote! { r as u8 },
        }],
        inspects: None,
        ensures: vec![],
        span: Span::call_site(),
    };

    assert_spec_eq(&spec, &expected);
}

#[test]
fn captures_edge_case_array_of_cast_exprs() {
    let spec: Spec = parse_quote! {
        captures: r8g8b8 = [
            r as u8,
            g as u8,
            b as u8,
        ],
    };

    let expected = Spec {
        qualifiers: FnQualifiers::empty(),
        requires: vec![],
        maintains: vec![],
        captures: vec![Capture {
            pat: parse_quote! { r8g8b8 },
            expr: parse_quote! {
                [
                    r as u8,
                    g as u8,
                    b as u8,
                ]
            },
        }],
        inspects: None,
        ensures: vec![],
        span: Span::call_site(),
    };

    assert_spec_eq(&spec, &expected);
}

#[test]
fn captures_edge_case_list_of_cast_exprs() {
    let spec: Spec = parse_quote! {
        captures: [
            old_red = r as u8,
            old_green = g as u8,
            old_blue = b as u8,
        ],
    };

    let expected = Spec {
        qualifiers: FnQualifiers::empty(),
        requires: vec![],
        maintains: vec![],
        captures: vec![
            Capture {
                pat: parse_quote! { old_red },
                expr: parse_quote! { r as u8 },
            },
            Capture {
                pat: parse_quote! { old_green },
                expr: parse_quote! { g as u8 },
            },
            Capture {
                pat: parse_quote! { old_blue },
                expr: parse_quote! { b as u8 },
            },
        ],
        inspects: None,
        ensures: vec![],
        span: Span::call_site(),
    };

    assert_spec_eq(&spec, &expected);
}

#[test]
fn captures_pattern_matches_slices() {
    let spec: Spec = parse_quote! {
        captures: [r, g, b] = rgb,
    };

    let expected = Spec {
        qualifiers: FnQualifiers::empty(),
        requires: vec![],
        maintains: vec![],
        captures: vec![Capture {
            pat: parse_quote! { [r, g, b] },
            expr: parse_quote! { rgb },
        }],
        inspects: None,
        ensures: vec![],
        span: Span::call_site(),
    };

    assert_spec_eq(&spec, &expected);
}

#[test]
fn captures_pattern_matches_tuples() {
    let spec: Spec = parse_quote! {
        captures: (x, y, z) = point,
    };

    let expected = Spec {
        qualifiers: FnQualifiers::empty(),
        requires: vec![],
        maintains: vec![],
        captures: vec![Capture {
            pat: parse_quote! { (x, y, z) },
            expr: parse_quote! { point },
        }],
        inspects: None,
        ensures: vec![],
        span: Span::call_site(),
    };

    assert_spec_eq(&spec, &expected);
}

#[test]
fn captures_pattern_matches_structs() {
    let spec: Spec = parse_quote! {
        captures: Person { name, age } = person.clone(),
    };

    let expected = Spec {
        qualifiers: FnQualifiers::empty(),
        requires: vec![],
        maintains: vec![],
        captures: vec![Capture {
            pat: parse_quote! { Person { name, age } },
            expr: parse_quote! { person.clone() },
        }],
        inspects: None,
        ensures: vec![],
        span: Span::call_site(),
    };

    assert_spec_eq(&spec, &expected);
}

#[test]
fn captures_pattern_matches_nested() {
    let spec: Spec = parse_quote! {
        captures: Some((a, b)) = data.as_ref(),
    };

    let expected = Spec {
        qualifiers: FnQualifiers::empty(),
        requires: vec![],
        maintains: vec![],
        captures: vec![Capture {
            pat: parse_quote! { Some((a, b)) },
            expr: parse_quote! { data.as_ref() },
        }],
        inspects: None,
        ensures: vec![],
        span: Span::call_site(),
    };

    assert_spec_eq(&spec, &expected);
}

#[test]
#[should_panic(expected = "expected `,`")]
fn captures_pattern_with_binding_modifier() {
    let _: Spec = parse_quote! {
        captures: Some(inner_tuple @ (a, b)) = data,
    };
}

#[test]
#[should_panic(expected = "expected `,`")]
fn captures_missing_assignment_value_is_rejected_by_spec_args() {
    let input = "captures: Person { name, age } =,";
    let _: syntax::SpecArgs = parse_str(input).unwrap();
}

#[test]
#[should_panic(expected = "expected `,`")]
fn captures_missing_assignment_value_errors_as_spec() {
    let _: Spec = parse_str("captures: Person { name, age } =,").unwrap();
}

#[test]
#[should_panic(expected = "expected an assignment or block")]
fn captures_require_an_assignment() {
    let _: Spec = parse_str("captures: value,").unwrap();
}

#[test]
#[should_panic(expected = "expected `,`")]
fn captures_with_extra_semicolon() {
    let _: Spec = parse_str("captures: old_value = value;,").unwrap();
}

#[test]
fn spec_arg_can_omit_value() {
    let spec_args: syntax::SpecArgs = parse_quote! {
        key_1,
        key_2: value,
        key_3: value as expr,
    };

    let args: Vec<_> = spec_args.args.into_iter().collect();
    let [arg_1, arg_2, arg_3] = args.as_slice() else {
        panic!("expected 3 args");
    };

    assert!(arg_1.keyword.to_string() == "key_1");
    assert!(arg_1.colon.is_none());
    assert_eq!(arg_1.value, SpecArgValue::None);

    assert!(arg_2.keyword.to_string() == "key_2");
    assert!(arg_2.colon.is_some());
    assert_eq!(arg_2.value, SpecArgValue::Expr(parse_quote!(value)));

    assert!(arg_3.keyword.to_string() == "key_3");
    assert!(arg_3.colon.is_some());
    assert_eq!(arg_3.value, SpecArgValue::Expr(parse_quote!(value as expr)));
}
