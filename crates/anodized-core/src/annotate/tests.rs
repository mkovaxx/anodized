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
        requires: vec![PreCondition {
            closure: parse_quote! { || is_valid(x) },
            cfg: None,
        }],
        maintains: vec![],
        captures: vec![],
        ensures: vec![PostCondition {
            closure: parse_quote! { |output| output > x },
            cfg: None,
        }],
        span: Span::call_site(),
    };

    assert_spec_eq(&spec, &expected);
}

#[test]
fn all_clauses() {
    let spec: Spec = parse_quote! {
        requires: x > 0 && x.is_power_of_two(),
        maintains: self.is_valid(),
        binds: z,
        ensures: z >= x,
    };

    let expected = Spec {
        requires: vec![PreCondition {
            closure: parse_quote! { || x > 0 && x.is_power_of_two() },
            cfg: None,
        }],
        maintains: vec![PreCondition {
            closure: parse_quote! { || self.is_valid() },
            cfg: None,
        }],
        captures: vec![],
        ensures: vec![PostCondition {
            closure: parse_quote! { |z| z >= x },
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
#[should_panic(expected = "multiple `binds` parameters are not allowed")]
fn multiple_binds() {
    let _: Spec = parse_quote! {
        binds: y,
        binds: z,
    };
}

#[test]
#[should_panic(
    expected = "at most one `captures` parameter is allowed; to capture multiple values, use a list"
)]
fn multiple_captures() {
    let _: Spec = parse_quote! {
        captures: value,
        captures: count as old_count,
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
            |output| output.is_some(),
        ],
    };

    let expected = Spec {
        requires: vec![
            PreCondition {
                closure: parse_quote! { || x >= 0 },
                cfg: None,
            },
            PreCondition {
                closure: parse_quote! { || y.len() < 10 },
                cfg: None,
            },
        ],
        maintains: vec![],
        captures: vec![],
        ensures: vec![
            PostCondition {
                closure: parse_quote! { |output| output != x },
                cfg: None,
            },
            PostCondition {
                closure: parse_quote! { |output| output.is_some() },
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
        requires: vec![],
        maintains: vec![],
        captures: vec![],
        ensures: vec![PostCondition {
            closure: parse_quote! { |result| result.is_ok() || result.unwrap_err().kind() == ErrorKind::NotFound },
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
        ensures: output < x,
        ensures: |output| output.len() >= y.len(),
    };

    let expected = Spec {
        requires: vec![
            PreCondition {
                closure: parse_quote! { || x > 0 || x < -10 },
                cfg: None,
            },
            PreCondition {
                closure: parse_quote! { || y.is_ascii() },
                cfg: None,
            },
        ],
        maintains: vec![],
        captures: vec![],
        ensures: vec![
            PostCondition {
                closure: parse_quote! { |output| output < x },
                cfg: None,
            },
            PostCondition {
                closure: parse_quote! { |output| output.len() >= y.len() },
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
            |output| output.starts_with(z),
        ],
        ensures: output.len() > x,
    };

    let expected = Spec {
        requires: vec![
            PreCondition {
                closure: parse_quote! { || x == 0 },
                cfg: None,
            },
            PreCondition {
                closure: parse_quote! { || y > 1 },
                cfg: None,
            },
            PreCondition {
                closure: parse_quote! { || z.is_empty() || z.contains("foo") },
                cfg: None,
            },
        ],
        maintains: vec![],
        captures: vec![],
        ensures: vec![
            PostCondition {
                closure: parse_quote! { |output| output != y },
                cfg: None,
            },
            PostCondition {
                closure: parse_quote! { |output| output.starts_with(z) },
                cfg: None,
            },
            PostCondition {
                closure: parse_quote! { |output| output.len() > x },
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
        requires: vec![PreCondition {
            closure: parse_quote! { || x > 0 && is_mode() },
            cfg: Some(parse_quote! { test }),
        }],
        maintains: vec![],
        captures: vec![],
        ensures: vec![PostCondition {
            closure: parse_quote! { |output| output < x },
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
#[should_panic(expected = "`cfg` attribute is not supported on `binds`")]
fn cfg_on_binds() {
    let _: Spec = parse_quote! {
        #[cfg(test)]
        binds: y,
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
        requires: vec![PreCondition {
            closure: parse_quote! { || matches!(self.state, State::Idle) },
            cfg: None,
        }],
        maintains: vec![PreCondition {
            closure: parse_quote! { || matches!(self.state, State::Idle | State::Running | State::Finished) },
            cfg: None,
        }],
        captures: vec![],
        ensures: vec![PostCondition {
            closure: parse_quote! { |output| matches!(self.state, State::Running) },
            cfg: None,
        }],
        span: Span::call_site(),
    };

    assert_spec_eq(&spec, &expected);
}

#[test]
fn binds_pattern() {
    let spec: Spec = parse_quote! {
        binds: (a, b),
        ensures: [
            a <= b,
            (a, b) == pair || (b, a) == pair,
        ],
    };

    let expected = Spec {
        requires: vec![],
        maintains: vec![],
        captures: vec![],
        ensures: vec![
            PostCondition {
                closure: parse_quote! { |(a, b)| a <= b },
                cfg: None,
            },
            PostCondition {
                closure: parse_quote! { |(a, b)| (a, b) == pair || (b, a) == pair },
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
        requires: vec![
            PreCondition {
                closure: parse_quote! { || self.initialized },
                cfg: None,
            },
            PreCondition {
                closure: parse_quote! { || !self.locked },
                cfg: None,
            },
            PreCondition {
                closure: parse_quote! { || index < self.items.len() },
                cfg: None,
            },
        ],
        maintains: vec![PreCondition {
            closure: parse_quote! { || self.items.len() <= self.items.capacity() },
            cfg: None,
        }],
        captures: vec![],
        ensures: vec![],
        span: Span::call_site(),
    };

    assert_spec_eq(&spec, &expected);
}

#[test]
fn rename_return_value() {
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
        captures: vec![],
        ensures: vec![
            PostCondition {
                closure: parse_quote! { |result| result > output },
                cfg: None,
            },
            PostCondition {
                closure: parse_quote! { |val| val % 2 == 0 },
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
        captures: count,
        ensures: output == old_count + 1,
    };

    let expected = Spec {
        requires: vec![],
        maintains: vec![],
        captures: vec![Capture {
            expr: parse_quote! { count },
            pat: parse_quote! { old_count },
        }],
        ensures: vec![PostCondition {
            closure: parse_quote! { |output| output == old_count + 1 },
            cfg: None,
        }],
        span: Span::call_site(),
    };

    assert_spec_eq(&spec, &expected);
}

#[test]
fn captures_identifier_with_alias() {
    let spec: Spec = parse_quote! {
        captures: value as prev_value,
        ensures: output > prev_value,
    };

    let expected = Spec {
        requires: vec![],
        maintains: vec![],
        captures: vec![Capture {
            expr: parse_quote! { value },
            pat: parse_quote! { prev_value },
        }],
        ensures: vec![PostCondition {
            closure: parse_quote! { |output| output > prev_value },
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
        captures: vec![
            Capture {
                expr: parse_quote! { count },
                pat: parse_quote! { old_count },
            },
            Capture {
                expr: parse_quote! { index },
                pat: parse_quote! { old_index },
            },
            Capture {
                expr: parse_quote! { value },
                pat: parse_quote! { old_value },
            },
        ],
        ensures: vec![
            PostCondition {
                closure: parse_quote! { |output| count == old_count + 1 },
                cfg: None,
            },
            PostCondition {
                closure: parse_quote! { |output| index == old_index + 1 },
                cfg: None,
            },
            PostCondition {
                closure: parse_quote! { |output| value > old_value },
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
        captures: value as old_val,
        binds: result,
        ensures: result > old_val,
    };

    let expected = Spec {
        requires: vec![PreCondition {
            closure: parse_quote! { || x > 0 },
            cfg: None,
        }],
        maintains: vec![PreCondition {
            closure: parse_quote! { || self.is_valid() },
            cfg: None,
        }],
        captures: vec![Capture {
            expr: parse_quote! { value },
            pat: parse_quote! { old_val },
        }],
        ensures: vec![PostCondition {
            closure: parse_quote! { |result| result > old_val },
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
        captures: value,
        maintains: self.is_valid(),
    };
}

#[test]
fn captures_array_expression() {
    let spec: Spec = parse_quote! {
        captures: [a, b, c] as slice,
        ensures: slice.len() == 3,
    };

    let expected = Spec {
        requires: vec![],
        maintains: vec![],
        captures: vec![Capture {
            expr: parse_quote! { [a, b, c] },
            pat: parse_quote! { slice },
        }],
        ensures: vec![PostCondition {
            closure: parse_quote! { |output| slice.len() == 3 },
            cfg: None,
        }],
        span: Span::call_site(),
    };

    assert_spec_eq(&spec, &expected);
}

#[test]
#[should_panic(
    expected = "complex expression must be bound/descructured: <expression> `as` <pattern>"
)]
fn captures_complex_expr_without_alias() {
    let _: Spec = parse_quote! {
        captures: self.items.len(),
        ensures: output > 0,
    };
}

#[test]
#[should_panic(
    expected = "complex expression must be bound/descructured: <expression> `as` <pattern>"
)]
fn captures_method_call_without_alias() {
    let _: Spec = parse_quote! {
        captures: foo.bar(),
        ensures: output > 0,
    };
}

#[test]
#[should_panic(
    expected = "complex expression must be bound/descructured: <expression> `as` <pattern>"
)]
fn captures_binary_expr_without_alias() {
    let _: Spec = parse_quote! {
        captures: a + b,
        ensures: output > 0,
    };
}

#[test]
#[should_panic(
    expected = "complex expression must be bound/descructured: <expression> `as` <pattern>"
)]
fn captures_array_with_complex_expr_no_alias() {
    let _: Spec = parse_quote! {
        captures: [
            count,
            // This should fail - complex expression needs explicit alias
            self.items.len(),
        ],
        ensures: output > 0,
    };
}

#[test]
#[should_panic(
    expected = "complex expression must be bound/descructured: <expression> `as` <pattern>"
)]
fn captures_indexing_expr_requires_alias() {
    // This should fail - `foo[0]` is a complex expression that requires an alias
    // Previously this was incorrectly parsed as just `foo`, silently capturing the wrong value
    let _: Spec = parse_quote! {
        captures: foo[0],
        ensures: output > 0,
    };
}

#[test]
#[should_panic(expected = "`cfg` attribute is not supported on `captures`")]
fn cfg_on_captures() {
    let _: Spec = parse_quote! {
        #[cfg(test)]
        captures: value as old_value,
        ensures: output > old_value,
    };
}

#[test]
fn captures_edge_case_cast_expr() {
    let spec: Spec = parse_quote! {
        captures: r as u8 as old_red,
    };

    let expected = Spec {
        requires: vec![],
        maintains: vec![],
        captures: vec![Capture {
            expr: parse_quote! { r as u8 },
            pat: parse_quote! { old_red },
        }],
        ensures: vec![],
        span: Span::call_site(),
    };

    assert_spec_eq(&spec, &expected);
}

#[test]
fn captures_edge_case_array_of_cast_exprs() {
    let spec: Spec = parse_quote! {
        captures: [
            r as u8,
            g as u8,
            b as u8,
        ] as r8g8b8,
    };

    let expected = Spec {
        requires: vec![],
        maintains: vec![],
        captures: vec![Capture {
            expr: parse_quote! {
                [
                    r as u8,
                    g as u8,
                    b as u8,
                ]
            },
            pat: parse_quote! { r8g8b8 },
        }],
        ensures: vec![],
        span: Span::call_site(),
    };

    assert_spec_eq(&spec, &expected);
}

#[test]
fn captures_edge_case_list_of_cast_exprs() {
    let spec: Spec = parse_quote! {
        captures: [
            r as u8 as old_red,
            g as u8 as old_green,
            b as u8 as old_blue,
        ],
    };

    let expected = Spec {
        requires: vec![],
        maintains: vec![],
        captures: vec![
            Capture {
                expr: parse_quote! { r as u8 },
                pat: parse_quote! { old_red },
            },
            Capture {
                expr: parse_quote! { g as u8 },
                pat: parse_quote! { old_green },
            },
            Capture {
                expr: parse_quote! { b as u8 },
                pat: parse_quote! { old_blue },
            },
        ],
        ensures: vec![],
        span: Span::call_site(),
    };

    assert_spec_eq(&spec, &expected);
}

#[test]
fn captures_pattern_matches_slices() {
    let spec: Spec = parse_quote! {
        captures: rgb as [r, g, b],
    };

    let expected = Spec {
        requires: vec![],
        maintains: vec![],
        captures: vec![Capture {
            expr: parse_quote! { rgb },
            pat: parse_quote! { [r, g, b] },
        }],
        ensures: vec![],
        span: Span::call_site(),
    };

    assert_spec_eq(&spec, &expected);
}

#[test]
fn captures_pattern_matches_tuples() {
    let spec: Spec = parse_quote! {
        captures: point as (x, y, z),
    };

    let expected = Spec {
        requires: vec![],
        maintains: vec![],
        captures: vec![Capture {
            expr: parse_quote! { point },
            pat: parse_quote! { (x, y, z) },
        }],
        ensures: vec![],
        span: Span::call_site(),
    };

    assert_spec_eq(&spec, &expected);
}

#[test]
fn captures_pattern_matches_structs() {
    let spec: Spec = parse_quote! {
        captures: person.clone() as Person { name, age },
    };

    let expected = Spec {
        requires: vec![],
        maintains: vec![],
        captures: vec![Capture {
            expr: parse_quote! { person.clone() },
            pat: parse_quote! { Person { name, age } },
        }],
        ensures: vec![],
        span: Span::call_site(),
    };

    assert_spec_eq(&spec, &expected);
}

#[test]
fn captures_pattern_matches_nested() {
    let spec: Spec = parse_quote! {
        captures: data.as_ref() as Some((a, b)),
    };

    let expected = Spec {
        requires: vec![],
        maintains: vec![],
        captures: vec![Capture {
            expr: parse_quote! { data.as_ref() },
            pat: parse_quote! { Some((a, b)) },
        }],
        ensures: vec![],
        span: Span::call_site(),
    };

    assert_spec_eq(&spec, &expected);
}

#[test]
fn captures_pattern_with_binding_modifier() {
    let spec: Spec = parse_quote! {
        captures: data as Some(inner_tuple @ (a, b)),
    };

    let expected = Spec {
        requires: vec![],
        maintains: vec![],
        captures: vec![Capture {
            expr: parse_quote! { data },
            pat: parse_quote! { Some(inner_tuple @ (a, b)) },
        }],
        ensures: vec![],
        span: Span::call_site(),
    };

    assert_spec_eq(&spec, &expected);
}

#[test]
fn captures_missing_expr_parses_as_spec_args() {
    let input = "captures: as Person { name, age },";
    let spec_args: syntax::SpecArgs =
        parse_str(input).expect("should parse incomplete capture expr for formatting");
    assert_eq!(spec_args.args.len(), 1);
}

#[test]
#[should_panic(expected = "expected capture: <expression> `as` <pattern>")]
fn captures_missing_expr_errors_as_spec() {
    let _: Spec = parse_str("captures: as Person { name, age },").unwrap();
}

#[test]
fn captures_expr_as_with_missing_pat_errors() {
    let capture_expr = syntax::CaptureExpr {
        expr: Some(parse_quote! { value }),
        as_: Some(Default::default()),
        pat: None,
    };
    let err = interpret_capture_expr_as_capture(capture_expr).unwrap_err();
    assert!(
        err.to_string().contains("expected pattern after `as`"),
        "{}",
        err
    );
}

#[test]
fn captures_pat_without_as_errors() {
    let capture_expr = syntax::CaptureExpr {
        expr: Some(parse_quote! { value }),
        as_: None,
        pat: Some(parse_quote! { old_value }),
    };
    let err = interpret_capture_expr_as_capture(capture_expr).unwrap_err();
    assert!(
        err.to_string().contains("expected `as` <pattern>"),
        "{}",
        err
    );
}
