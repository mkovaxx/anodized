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

impl ToTokens for Condition {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        self.expr.to_tokens(tokens);
    }
}

impl ToTokens for ConditionClosure {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        self.closure.to_tokens(tokens);
    }
}

pub fn assert_contract_eq(left: &Contract, right: &Contract) {
    assert_token_streams_eq(&left.requires, &right.requires, "requires");
    assert_token_streams_eq(&left.maintains, &right.maintains, "maintains");
    assert_token_streams_eq(&left.ensures, &right.ensures, "ensures");
}

fn assert_token_streams_eq<T: ToTokens>(left: &[T], right: &[T], clause_name: &str) {
    assert_eq!(
        left.len(),
        right.len(),
        "number of {} clauses do not match",
        clause_name
    );

    for (i, (left_item, right_item)) in left.iter().zip(right.iter()).enumerate() {
        let left_tokens: Vec<String> = left_item
            .to_token_stream()
            .into_iter()
            .map(|t| t.to_string())
            .collect();
        let right_tokens: Vec<String> = right_item
            .to_token_stream()
            .into_iter()
            .map(|t| t.to_string())
            .collect();
        assert_eq!(
            left_tokens,
            right_tokens,
            "{} clause #{} does not match",
            clause_name,
            i + 1
        );
    }
}