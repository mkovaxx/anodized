use crate::Contract;
use quote::ToTokens;

pub fn assert_contract_eq(left: &Contract, right: &Contract) {
    let left_requires = left
        .requires
        .iter()
        .map(|e| e.to_token_stream().to_string())
        .collect::<Vec<_>>();
    let right_requires = right
        .requires
        .iter()
        .map(|e| e.to_token_stream().to_string())
        .collect::<Vec<_>>();
    let left_maintains = left
        .maintains
        .iter()
        .map(|e| e.to_token_stream().to_string())
        .collect::<Vec<_>>();
    let right_maintains = right
        .maintains
        .iter()
        .map(|e| e.to_token_stream().to_string())
        .collect::<Vec<_>>();
    let left_ensures = left
        .ensures
        .iter()
        .map(|e| e.to_token_stream().to_string())
        .collect::<Vec<_>>();
    let right_ensures = right
        .ensures
        .iter()
        .map(|e| e.to_token_stream().to_string())
        .collect::<Vec<_>>();

    assert_eq!(
        left_requires, right_requires,
        "requires clauses do not match"
    );
    assert_eq!(
        left_maintains, right_maintains,
        "maintains clauses do not match"
    );
    assert_eq!(left_ensures, right_ensures, "ensures clauses do not match");
}
