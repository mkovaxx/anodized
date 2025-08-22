use crate::Contract;
use quote::ToTokens;

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
            left_tokens, right_tokens,
            "{} clause #{} does not match",
            clause_name,
            i + 1
        );
    }
}

pub fn assert_contract_eq(left: &Contract, right: &Contract) {
    assert_token_streams_eq(&left.requires, &right.requires, "requires");
    assert_token_streams_eq(&left.maintains, &right.maintains, "maintains");
    assert_token_streams_eq(&left.ensures, &right.ensures, "ensures");
}
