use crate::Contract;
use quote::ToTokens;

#[derive(Debug)]
pub struct TestContract(pub Contract);

impl PartialEq for TestContract {
    fn eq(&self, other: &Self) -> bool {
        let self_requires = self
            .0
            .requires
            .iter()
            .map(|e| e.to_token_stream().to_string())
            .collect::<Vec<_>>();
        let other_requires = other
            .0
            .requires
            .iter()
            .map(|e| e.to_token_stream().to_string())
            .collect::<Vec<_>>();
        let self_maintains = self
            .0
            .maintains
            .iter()
            .map(|e| e.to_token_stream().to_string())
            .collect::<Vec<_>>();
        let other_maintains = other
            .0
            .maintains
            .iter()
            .map(|e| e.to_token_stream().to_string())
            .collect::<Vec<_>>();
        let self_ensures = self
            .0
            .ensures
            .iter()
            .map(|e| e.to_token_stream().to_string())
            .collect::<Vec<_>>();
        let other_ensures = other
            .0
            .ensures
            .iter()
            .map(|e| e.to_token_stream().to_string())
            .collect::<Vec<_>>();

        self_requires == other_requires
            && self_maintains == other_maintains
            && self_ensures == other_ensures
    }
}
