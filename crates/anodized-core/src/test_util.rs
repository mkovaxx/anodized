use crate::{Condition, ConditionClosure, Contract};
use quote::ToTokens;
use syn::parse::{Parse, ParseStream, Result};

impl Parse for Condition {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(Condition {
            expr: input.parse()?,
            cfg: None,
        })
    }
}

impl Parse for ConditionClosure {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(ConditionClosure {
            closure: input.parse()?,
            cfg: None,
        })
    }
}

pub fn assert_contract_eq(left: &Contract, right: &Contract) {
    assert_conditions_eq(&left.requires, &right.requires, "requires");
    assert_conditions_eq(&left.maintains, &right.maintains, "maintains");
    assert_condition_closures_eq(&left.ensures, &right.ensures, "ensures");
}

fn assert_conditions_eq(left: &[Condition], right: &[Condition], clause_name: &str) {
    assert_eq!(
        left.len(),
        right.len(),
        "number of {} clauses do not match",
        clause_name
    );

    for (i, (left_item, right_item)) in left.iter().zip(right.iter()).enumerate() {
        assert_eq!(
            left_item.cfg.to_token_stream().to_string(),
            right_item.cfg.to_token_stream().to_string(),
            "{} clause #{} cfg does not match",
            clause_name,
            i + 1
        );

        assert_eq!(
            left_item.expr.to_token_stream().to_string(),
            right_item.expr.to_token_stream().to_string(),
            "{} clause #{} does not match",
            clause_name,
            i + 1
        );
    }
}

fn assert_condition_closures_eq(
    left: &[ConditionClosure],
    right: &[ConditionClosure],
    clause_name: &str,
) {
    assert_eq!(
        left.len(),
        right.len(),
        "number of {} clauses do not match",
        clause_name
    );

    for (i, (left_item, right_item)) in left.iter().zip(right.iter()).enumerate() {
        assert_eq!(
            left_item.cfg.to_token_stream().to_string(),
            right_item.cfg.to_token_stream().to_string(),
            "{} clause #{} cfg does not match",
            clause_name,
            i + 1
        );

        assert_eq!(
            left_item.closure.to_token_stream().to_string(),
            right_item.closure.to_token_stream().to_string(),
            "{} clause #{} does not match",
            clause_name,
            i + 1
        );
    }
}
