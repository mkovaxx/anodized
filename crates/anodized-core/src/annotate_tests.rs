use crate::test_util::{SpecItemFn, assert_spec_eq};

use super::*;
use proc_macro2::Span;
use quote::ToTokens;
use syn::{parse_quote, parse_str};

#[test]
#[should_panic(expected = "must have no fields")]
fn empty_spec_rejects_nonempty_fields() {
    let fields: crate::syntax::spec::SpecFields = parse_quote! { maintains: true };
    let _: EmptySpec = fields.try_into().unwrap();
}

#[test]
fn simple_spec() {
    let spec_item_fn: SpecItemFn = parse_quote! {
        #[spec(
            requires: is_valid(x),
            ensures: output > x,
        )]
        fn f() {}
    };

    let expected = FnSpec {
        qualifiers: FnQualifiers::empty(),
        input_specs: vec![],
        output_spec_on_exit: false,
        requires: vec![Condition {
            expr: parse_quote! { is_valid(x) },
            cfg: None,
        }],
        maintains: vec![],
        captures: vec![],
        ensures: vec![PostCondition {
            pat: None,
            expr: parse_quote! { output > x },
            cfg: None,
        }],
        span: Span::call_site(),
    };

    assert_spec_eq(&spec_item_fn.spec, &expected);
}

#[test]
fn fn_qualifiers_functional() {
    let spec_item_fn: SpecItemFn = parse_quote! {
        #[spec(functional)]
        fn f() {}
    };

    let expected = FnSpec {
        qualifiers: FnQualifiers::FUNCTIONAL,
        input_specs: vec![],
        output_spec_on_exit: false,
        requires: vec![],
        maintains: vec![],
        captures: vec![],
        ensures: vec![],
        span: Span::call_site(),
    };

    assert_spec_eq(&spec_item_fn.spec, &expected);
}

#[test]
fn fn_qualifiers_pure_total() {
    let spec_item_fn: SpecItemFn = parse_quote! {
        #[spec(pure, total)]
        fn f() {}
    };

    let expected = FnSpec {
        qualifiers: FnQualifiers::PURE | FnQualifiers::TOTAL,
        input_specs: vec![],
        output_spec_on_exit: false,
        requires: vec![],
        maintains: vec![],
        captures: vec![],
        ensures: vec![],
        span: Span::call_site(),
    };

    assert_spec_eq(&spec_item_fn.spec, &expected);
}

#[test]
fn fn_qualifiers_deterministic_effectfree_infallible_terminating() {
    let spec_item_fn: SpecItemFn = parse_quote! {
        #[spec(
            deterministic,
            effectfree,
            infallible,
            terminating,
        )]
        fn f() {}
    };

    let expected = FnSpec {
        qualifiers: FnQualifiers::DETERMINISTIC
            | FnQualifiers::EFFECTFREE
            | FnQualifiers::INFALLIBLE
            | FnQualifiers::TERMINATING,
        input_specs: vec![],
        output_spec_on_exit: false,
        requires: vec![],
        maintains: vec![],
        captures: vec![],
        ensures: vec![],
        span: Span::call_site(),
    };

    assert_spec_eq(&spec_item_fn.spec, &expected);
}

#[test]
#[should_panic = "expected `,`"]
fn fn_qualifiers_typo_missing_comma() {
    let _: SpecItemFn = parse_quote! {
        #[spec(pure total)]
        fn f() {}
    };
}

#[test]
#[should_panic = "qualifier does not take a value"]
fn fn_qualifiers_typo_colon_instead_of_comma() {
    let _: SpecItemFn = parse_quote! {
        #[spec(pure: total)]
        fn f() {}
    };
}

#[test]
#[should_panic = "expected an expression"]
fn fn_qualifiers_invalid_colon() {
    let _: SpecItemFn = parse_quote! {
        #[spec(pure:)]
        fn f() {}
    };
}

#[test]
#[should_panic = "qualifier does not take a value"]
fn fn_qualifiers_invalid_value_expr() {
    let _: SpecItemFn = parse_quote! {
        #[spec(functional: x == 42)]
        fn f() {}
    };
}

#[test]
#[should_panic = "this qualifier is redundant; remove it"]
fn fn_qualifiers_functional_pure() {
    let _: SpecItemFn = parse_quote! {
        #[spec(functional, pure)]
        fn f() {}
    };
}

#[test]
#[should_panic = "this qualifier is redundant; remove it"]
fn fn_qualifiers_functional_total() {
    let _: SpecItemFn = parse_quote! {
        #[spec(functional, total)]
        fn f() {}
    };
}

#[test]
#[should_panic = "this qualifier is redundant; remove it"]
fn fn_qualifiers_pure_deterministic() {
    let _: SpecItemFn = parse_quote! {
        #[spec(
            pure,
            deterministic,
        )]
        fn f() {}
    };
}

#[test]
#[should_panic = "this qualifier is redundant; remove it"]
fn fn_qualifiers_pure_effectfree() {
    let _: SpecItemFn = parse_quote! {
        #[spec(pure, effectfree)]
        fn f() {}
    };
}

#[test]
#[should_panic = "this qualifier is redundant; remove it"]
fn fn_qualifiers_total_infallible() {
    let _: SpecItemFn = parse_quote! {
        #[spec(total, infallible)]
        fn f() {}
    };
}

#[test]
#[should_panic = "this qualifier is redundant; remove it"]
fn fn_qualifiers_total_terminating() {
    let _: SpecItemFn = parse_quote! {
        #[spec(total, terminating)]
        fn f() {}
    };
}

#[test]
fn all_clauses() {
    let spec_item_fn: SpecItemFn = parse_quote! {
        #[spec(
            requires: x > 0 && x.is_power_of_two(),
            maintains: self.is_valid(),
            ensures: |z| z >= x,
        )]
        fn f() {}
    };

    let expected = FnSpec {
        qualifiers: FnQualifiers::empty(),
        input_specs: vec![],
        output_spec_on_exit: false,
        requires: vec![Condition {
            expr: parse_quote! { x > 0 && x.is_power_of_two() },
            cfg: None,
        }],
        maintains: vec![Condition {
            expr: parse_quote! { self.is_valid() },
            cfg: None,
        }],
        captures: vec![],
        ensures: vec![PostCondition {
            pat: Some(parse_quote! { z }),
            expr: parse_quote! { z >= x },
            cfg: None,
        }],
        span: Span::call_site(),
    };

    assert_spec_eq(&spec_item_fn.spec, &expected);
}

#[test]
#[should_panic(expected = "unknown spec field")]
fn unknown_keyword() {
    let _: SpecItemFn = parse_quote! {
        #[spec(
            ensures: output == x,
            goat: 42,
            requires: x > 0 && !is_zero(x),
        )]
        fn f() {}
    };
}

#[test]
#[should_panic(expected = "fields are out of order")]
fn out_of_order() {
    let _: SpecItemFn = parse_quote! {
        #[spec(
            ensures: output == x,
            requires: x > 0 && !is_zero(x),
        )]
        fn f() {}
    };
}

#[test]
#[should_panic(expected = "no longer supported")]
fn multiple_binds() {
    let _: SpecItemFn = parse_quote! {
        #[spec(
            inspects: y,
            inspects: z,
        )]
        fn f() {}
    };
}

#[test]
#[should_panic(
    expected = "at most one `captures` field is allowed; to capture multiple values, use a list"
)]
fn multiple_captures() {
    let _: SpecItemFn = parse_quote! {
        #[spec(
            captures: old_value = value,
            captures: old_count = count,
        )]
        fn f() {}
    };
}

#[test]
fn array_of_conditions() {
    let spec_item_fn: SpecItemFn = parse_quote! {
        #[spec(
            requires: [
                x >= 0,
                y.len() < 10,
            ],
            ensures: [
                output != x,
                |retval| retval.is_some(),
            ],
        )]
        fn f() {}
    };

    let expected = FnSpec {
        qualifiers: FnQualifiers::empty(),
        input_specs: vec![],
        output_spec_on_exit: false,
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

    assert_spec_eq(&spec_item_fn.spec, &expected);
}

#[test]
fn ensures_with_explicit_closure() {
    let spec_item_fn: SpecItemFn = parse_quote! {
        #[spec(
            ensures: |result| result.is_ok() || result.unwrap_err().kind() == ErrorKind::NotFound,
        )]
        fn f() {}
    };

    let expected = FnSpec {
        qualifiers: FnQualifiers::empty(),
        input_specs: vec![],
        output_spec_on_exit: false,
        requires: vec![],
        maintains: vec![],
        captures: vec![],
        ensures: vec![PostCondition {
            pat: Some(parse_quote! { result }),
            expr: parse_quote! { result.is_ok() || result.unwrap_err().kind() == ErrorKind::NotFound },
            cfg: None,
        }],
        span: Span::call_site(),
    };

    assert_spec_eq(&spec_item_fn.spec, &expected);
}

#[test]
fn multiple_clauses_of_same_flavor() {
    let spec_item_fn: SpecItemFn = parse_quote! {
        #[spec(
            requires: x > 0 || x < -10,
            requires: y.is_ascii(),
            ensures: retval < x,
            ensures: |output| output.len() >= y.len(),
        )]
        fn f() {}
    };

    let expected = FnSpec {
        qualifiers: FnQualifiers::empty(),
        input_specs: vec![],
        output_spec_on_exit: false,
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

    assert_spec_eq(&spec_item_fn.spec, &expected);
}

#[test]
fn mixed_single_and_array_clauses() {
    let spec_item_fn: SpecItemFn = parse_quote! {
        #[spec(
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
            ensures: |output| [
                output >= x,
                x != z,
            ],
        )]
        fn f() {}
    };

    let expected = FnSpec {
        qualifiers: FnQualifiers::empty(),
        input_specs: vec![],
        output_spec_on_exit: false,
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
            PostCondition {
                pat: Some(parse_quote! { output }),
                expr: parse_quote! { output >= x },
                cfg: None,
            },
            PostCondition {
                pat: Some(parse_quote! { output }),
                expr: parse_quote! { x != z },
                cfg: None,
            },
        ],
        span: Span::call_site(),
    };

    assert_spec_eq(&spec_item_fn.spec, &expected);
}

#[test]
fn cfg_attributes() {
    let spec_item_fn: SpecItemFn = parse_quote! {
        #[spec(
            #[cfg(test)]
            requires: x > 0 && is_mode(),
            #[cfg(not(debug_assertions))]
            ensures: output < x,
        )]
        fn f() {}
    };

    let expected = FnSpec {
        qualifiers: FnQualifiers::empty(),
        input_specs: vec![],
        output_spec_on_exit: false,
        requires: vec![Condition {
            expr: parse_quote! { x > 0 && is_mode() },
            cfg: Some(parse_quote! { test }),
        }],
        maintains: vec![],
        captures: vec![],
        ensures: vec![PostCondition {
            pat: None,
            expr: parse_quote! { output < x },
            cfg: Some(parse_quote! { not(debug_assertions) }),
        }],
        span: Span::call_site(),
    };

    assert_spec_eq(&spec_item_fn.spec, &expected);
}

#[test]
#[should_panic(expected = "unsupported attribute; only `cfg` is allowed")]
fn non_cfg_attribute() {
    let _: SpecItemFn = parse_quote! {
        #[spec(
            #[allow(dead_code)]
            requires: x > 0,
        )]
        fn f() {}
    };
}

#[test]
#[should_panic(expected = "multiple `cfg` attributes are not supported")]
fn multiple_cfg_attributes() {
    let _: SpecItemFn = parse_quote! {
        #[spec(
            #[cfg(test)]
            #[cfg(debug_assertions)]
            requires: x > 0,
        )]
        fn f() {}
    };
}

#[test]
#[should_panic(expected = "no longer supported")]
fn cfg_on_binds() {
    let _: SpecItemFn = parse_quote! {
        #[spec(
            #[cfg(test)]
            inspects: y,
        )]
        fn f() {}
    };
}

#[test]
fn macro_in_condition() {
    let spec_item_fn: SpecItemFn = parse_quote! {
        #[spec(
            requires: matches!(self.state, State::Idle),
            maintains: matches!(self.state, State::Idle | State::Running | State::Finished),
            ensures: matches!(self.state, State::Running),
        )]
        fn f() {}
    };

    let expected = FnSpec {
        qualifiers: FnQualifiers::empty(),
        input_specs: vec![],
        output_spec_on_exit: false,
        requires: vec![Condition {
            expr: parse_quote! { matches!(self.state, State::Idle) },
            cfg: None,
        }],
        maintains: vec![Condition {
            expr: parse_quote! { matches!(self.state, State::Idle | State::Running | State::Finished) },
            cfg: None,
        }],
        captures: vec![],
        ensures: vec![PostCondition {
            pat: None,
            expr: parse_quote! { matches!(self.state, State::Running) },
            cfg: None,
        }],
        span: Span::call_site(),
    };

    assert_spec_eq(&spec_item_fn.spec, &expected);
}

#[test]
fn binds_pattern() {
    let spec_item_fn: SpecItemFn = parse_quote! {
        #[spec(
            ensures: |(a, b)| [
                a <= b,
                (a, b) == pair || (b, a) == pair,
            ],
        )]
        fn f() {}
    };

    let expected = FnSpec {
        qualifiers: FnQualifiers::empty(),
        input_specs: vec![],
        output_spec_on_exit: false,
        requires: vec![],
        maintains: vec![],
        captures: vec![],
        ensures: vec![
            PostCondition {
                pat: Some(parse_quote! { (a, b) }),
                expr: parse_quote! { a <= b },
                cfg: None,
            },
            PostCondition {
                pat: Some(parse_quote! { (a, b) }),
                expr: parse_quote! { (a, b) == pair || (b, a) == pair },
                cfg: None,
            },
        ],
        span: Span::call_site(),
    };

    assert_spec_eq(&spec_item_fn.spec, &expected);
}

#[test]
fn multiple_conditions() {
    let spec_item_fn: SpecItemFn = parse_quote! {
        #[spec(
            requires: [
                self.initialized,
                !self.locked,
            ],
            requires: index < self.items.len(),
            maintains: self.items.len() <= self.items.capacity(),
        )]
        fn f() {}
    };

    let expected = FnSpec {
        qualifiers: FnQualifiers::empty(),
        input_specs: vec![],
        output_spec_on_exit: false,
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
        ensures: vec![],
        span: Span::call_site(),
    };

    assert_spec_eq(&spec_item_fn.spec, &expected);
}

#[test]
fn rename_return_value() {
    let spec_item_fn: SpecItemFn = parse_quote! {
        #[spec(
            ensures: |result| [
                result > output,
                result % 2 == 0,
            ],
        )]
        fn f() {}
    };

    let expected = FnSpec {
        qualifiers: FnQualifiers::empty(),
        input_specs: vec![],
        output_spec_on_exit: false,
        requires: vec![],
        maintains: vec![],
        captures: vec![],
        ensures: vec![
            PostCondition {
                pat: Some(parse_quote! { result }),
                expr: parse_quote! { result > output },
                cfg: None,
            },
            PostCondition {
                pat: Some(parse_quote! { result }),
                expr: parse_quote! { result % 2 == 0 },
                cfg: None,
            },
        ],
        span: Span::call_site(),
    };

    assert_spec_eq(&spec_item_fn.spec, &expected);
}

#[test]
fn captures_simple_identifier() {
    let spec_item_fn: SpecItemFn = parse_quote! {
        #[spec(
            captures: old_count = count,
            ensures: output == old_count + 1,
        )]
        fn f() {}
    };

    let expected = FnSpec {
        qualifiers: FnQualifiers::empty(),
        input_specs: vec![],
        output_spec_on_exit: false,
        requires: vec![],
        maintains: vec![],
        captures: vec![Capture {
            pat: parse_quote! { old_count },
            expr: parse_quote! { count },
        }],
        ensures: vec![PostCondition {
            pat: None,
            expr: parse_quote! { output == old_count + 1 },
            cfg: None,
        }],
        span: Span::call_site(),
    };

    assert_spec_eq(&spec_item_fn.spec, &expected);
}

#[test]
fn captures_identifier_with_alias() {
    let spec_item_fn: SpecItemFn = parse_quote! {
        #[spec(
            captures: prev_value = value,
            ensures: output > prev_value,
        )]
        fn f() {}
    };

    let expected = FnSpec {
        qualifiers: FnQualifiers::empty(),
        input_specs: vec![],
        output_spec_on_exit: false,
        requires: vec![],
        maintains: vec![],
        captures: vec![Capture {
            pat: parse_quote! { prev_value },
            expr: parse_quote! { value },
        }],
        ensures: vec![PostCondition {
            pat: None,
            expr: parse_quote! { output > prev_value },
            cfg: None,
        }],
        span: Span::call_site(),
    };

    assert_spec_eq(&spec_item_fn.spec, &expected);
}

#[test]
fn captures_array() {
    let spec_item_fn: SpecItemFn = parse_quote! {
        #[spec(
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
        )]
        fn f() {}
    };

    let expected = FnSpec {
        qualifiers: FnQualifiers::empty(),
        input_specs: vec![],
        output_spec_on_exit: false,
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

    assert_spec_eq(&spec_item_fn.spec, &expected);
}

#[test]
fn captures_with_all_clauses() {
    let spec_item_fn: SpecItemFn = parse_quote! {
        #[spec(
            requires: x > 0,
            maintains: self.is_valid(),
            captures: old_val = value,
            ensures: |result| result > old_val,
        )]
        fn f() {}
    };

    let expected = FnSpec {
        qualifiers: FnQualifiers::empty(),
        input_specs: vec![],
        output_spec_on_exit: false,
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
        ensures: vec![PostCondition {
            pat: Some(parse_quote! { result }),
            expr: parse_quote! { result > old_val },
            cfg: None,
        }],
        span: Span::call_site(),
    };

    assert_spec_eq(&spec_item_fn.spec, &expected);
}

#[test]
#[should_panic(expected = "fields are out of order")]
fn captures_out_of_order() {
    let _: SpecItemFn = parse_quote! {
        #[spec(
            captures: old_value = value,
            maintains: self.is_valid(),
        )]
        fn f() {}
    };
}

#[test]
fn captures_array_expression() {
    let spec_item_fn: SpecItemFn = parse_quote! {
        #[spec(
            captures: slice = [a, b, c],
            ensures: slice.len() == 3,
        )]
        fn f() {}
    };

    let expected = FnSpec {
        qualifiers: FnQualifiers::empty(),
        input_specs: vec![],
        output_spec_on_exit: false,
        requires: vec![],
        maintains: vec![],
        captures: vec![Capture {
            pat: parse_quote! { slice },
            expr: parse_quote! { [a, b, c] },
        }],
        ensures: vec![PostCondition {
            pat: None,
            expr: parse_quote! { slice.len() == 3 },
            cfg: None,
        }],
        span: Span::call_site(),
    };

    assert_spec_eq(&spec_item_fn.spec, &expected);
}

#[test]
fn captures_complex_expressions() {
    let spec_item_fn: SpecItemFn = parse_quote! {
        #[spec(
            captures: [
                item_count = self.items.len(),
                bar = foo.bar(),
                sum = a + b,
                first = foo[0],
            ],
            ensures: output > 0,
        )]
        fn f() {}
    };

    assert_eq!(spec_item_fn.spec.captures.len(), 4);
}

#[test]
#[should_panic(expected = "`cfg` attribute is not supported here")]
fn cfg_on_captures() {
    let _: SpecItemFn = parse_quote! {
        #[spec(
            #[cfg(test)]
            captures: old_value = value,
            ensures: output > old_value,
        )]
        fn f() {}
    };
}

#[test]
fn captures_edge_case_cast_expr() {
    let spec_item_fn: SpecItemFn = parse_quote! {
        #[spec(captures: old_red = r as u8)]
        fn f() {}
    };

    let expected = FnSpec {
        qualifiers: FnQualifiers::empty(),
        input_specs: vec![],
        output_spec_on_exit: false,
        requires: vec![],
        maintains: vec![],
        captures: vec![Capture {
            pat: parse_quote! { old_red },
            expr: parse_quote! { r as u8 },
        }],
        ensures: vec![],
        span: Span::call_site(),
    };

    assert_spec_eq(&spec_item_fn.spec, &expected);
}

#[test]
fn captures_edge_case_array_of_cast_exprs() {
    let spec_item_fn: SpecItemFn = parse_quote! {
        #[spec(captures: r8g8b8 = [
            r as u8,
            g as u8,
            b as u8,
        ])]
        fn f() {}
    };

    let expected = FnSpec {
        qualifiers: FnQualifiers::empty(),
        input_specs: vec![],
        output_spec_on_exit: false,
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
        ensures: vec![],
        span: Span::call_site(),
    };

    assert_spec_eq(&spec_item_fn.spec, &expected);
}

#[test]
fn captures_edge_case_list_of_cast_exprs() {
    let spec_item_fn: SpecItemFn = parse_quote! {
        #[spec(
            captures: [
                old_red = r as u8,
                old_green = g as u8,
                old_blue = b as u8,
            ],
        )]
        fn f() {}
    };

    let expected = FnSpec {
        qualifiers: FnQualifiers::empty(),
        input_specs: vec![],
        output_spec_on_exit: false,
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
        ensures: vec![],
        span: Span::call_site(),
    };

    assert_spec_eq(&spec_item_fn.spec, &expected);
}

#[test]
fn captures_pattern_matches_slices() {
    let spec_item_fn: SpecItemFn = parse_quote! {
        #[spec(captures: [r, g, b] = rgb)]
        fn f() {}
    };

    let expected = FnSpec {
        qualifiers: FnQualifiers::empty(),
        input_specs: vec![],
        output_spec_on_exit: false,
        requires: vec![],
        maintains: vec![],
        captures: vec![Capture {
            pat: parse_quote! { [r, g, b] },
            expr: parse_quote! { rgb },
        }],
        ensures: vec![],
        span: Span::call_site(),
    };

    assert_spec_eq(&spec_item_fn.spec, &expected);
}

#[test]
fn captures_pattern_matches_tuples() {
    let spec_item_fn: SpecItemFn = parse_quote! {
        #[spec(captures: (x, y, z) = point)]
        fn f() {}
    };

    let expected = FnSpec {
        qualifiers: FnQualifiers::empty(),
        input_specs: vec![],
        output_spec_on_exit: false,
        requires: vec![],
        maintains: vec![],
        captures: vec![Capture {
            pat: parse_quote! { (x, y, z) },
            expr: parse_quote! { point },
        }],
        ensures: vec![],
        span: Span::call_site(),
    };

    assert_spec_eq(&spec_item_fn.spec, &expected);
}

#[test]
fn captures_pattern_matches_structs() {
    let spec_item_fn: SpecItemFn = parse_quote! {
        #[spec(captures: Person { name, age } = person.clone())]
        fn f() {}
    };

    let expected = FnSpec {
        qualifiers: FnQualifiers::empty(),
        input_specs: vec![],
        output_spec_on_exit: false,
        requires: vec![],
        maintains: vec![],
        captures: vec![Capture {
            pat: parse_quote! { Person { name, age } },
            expr: parse_quote! { person.clone() },
        }],
        ensures: vec![],
        span: Span::call_site(),
    };

    assert_spec_eq(&spec_item_fn.spec, &expected);
}

#[test]
fn captures_pattern_matches_nested() {
    let spec_item_fn: SpecItemFn = parse_quote! {
        #[spec(captures: Some((a, b)) = data.as_ref())]
        fn f() {}
    };

    let expected = FnSpec {
        qualifiers: FnQualifiers::empty(),
        input_specs: vec![],
        output_spec_on_exit: false,
        requires: vec![],
        maintains: vec![],
        captures: vec![Capture {
            pat: parse_quote! { Some((a, b)) },
            expr: parse_quote! { data.as_ref() },
        }],
        ensures: vec![],
        span: Span::call_site(),
    };

    assert_spec_eq(&spec_item_fn.spec, &expected);
}

#[test]
#[should_panic(expected = "expected `,`")]
fn captures_pattern_with_binding_modifier() {
    let _: SpecItemFn = parse_quote! {
        #[spec(captures: Some(inner_tuple @ (a, b)) = data)]
        fn f() {}
    };
}

#[test]
#[should_panic(expected = "expected an expression")]
fn captures_missing_assignment_value_is_rejected_by_spec_fields() {
    let input = "captures: Person { name, age } =,";
    let _: crate::syntax::spec::SpecFields = parse_str(input).unwrap();
}

#[test]
#[should_panic(expected = "expected an expression")]
fn captures_missing_assignment_value_errors_as_spec() {
    let _: SpecItemFn = parse_quote! {
        #[spec(captures: Person { name, age } =,)]
        fn f() {}
    };
}

#[test]
#[should_panic(expected = "expected an assignment or block")]
fn captures_require_an_assignment() {
    let _: SpecItemFn = parse_quote! {
        #[spec(captures: value)]
        fn f() {}
    };
}

#[test]
#[should_panic(expected = "expected `,`")]
fn captures_with_extra_semicolon() {
    let _: SpecItemFn = parse_quote! {
        #[spec(captures: old_value = value;,)]
        fn f() {}
    };
}

#[test]
fn field_value_supports_shorthand_expression() {
    let spec_fields: crate::syntax::spec::SpecFields = parse_quote! {
        key_1,
        key_2: value,
        key_3: value as expr,
    };

    let fields: Vec<_> = spec_fields.fields.into_iter().collect();
    let [field_1, field_2, field_3] = fields.as_slice() else {
        panic!("expected 3 fields");
    };

    assert_eq!(field_1.member.to_token_stream().to_string(), "key_1");
    assert!(field_1.colon_token.is_none());
    assert_eq!(field_1.expr.to_token_stream().to_string(), "key_1");

    assert_eq!(field_2.member.to_token_stream().to_string(), "key_2");
    assert!(field_2.colon_token.is_some());
    assert_eq!(field_2.expr.to_token_stream().to_string(), "value");

    assert_eq!(field_3.member.to_token_stream().to_string(), "key_3");
    assert!(field_3.colon_token.is_some());
    assert_eq!(field_3.expr.to_token_stream().to_string(), "value as expr");
}
