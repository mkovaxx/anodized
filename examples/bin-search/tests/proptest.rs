use anodized::result::{PostError, PreError, try_call};

use proptest::prelude::*;
use proptest_derive::Arbitrary;

proptest! {
    #![proptest_config(ProptestConfig::with_cases(10_000))]

    /// Generate random inputs and assert that the precondition implies the postcondition.
    #[test]
    fn test_spec(inputs: Inputs<i32>) {
        // Use Anodized's `try_call!` macro to defer acting on spec violations.
        let result = try_call! {
            bin_search::bin_search(&inputs.seq, &inputs.value)
        };
        match result {
            // Successful call.
            Ok(_) => {},
            // When preconditions are violated, reject the input.
            Err(PreError) => prop_assume!(false),
            // When postconditions are violated, panic to signal a counter-example.
            Err(PostError(output)) => {
                eprintln!("inputs:");
                dbg!(inputs.seq);
                dbg!(inputs.value);
                dbg!(output);
                panic!("postcondition failed");
            }
        }
    }
}

#[derive(Debug, Arbitrary)]
struct Inputs<T: Ord> {
    seq: AscendingVec<T>,
    value: T,
}

/// Helper newtype to more efficiently generate valid inputs.
#[derive(Debug, Clone)]
struct AscendingVec<T>(Vec<T>);

impl<T: Ord + Arbitrary> Arbitrary for AscendingVec<T> {
    type Parameters = <Vec<T> as Arbitrary>::Parameters;
    type Strategy = proptest::strategy::Map<<Vec<T> as Arbitrary>::Strategy, fn(Vec<T>) -> Self>;

    fn arbitrary_with(args: Self::Parameters) -> Self::Strategy {
        Vec::<T>::arbitrary_with(args).prop_map(Self::from_unsorted)
    }
}

impl<T: Ord> AscendingVec<T> {
    fn from_unsorted(mut values: Vec<T>) -> Self {
        values.sort();
        Self(values)
    }
}

impl<T> std::ops::Deref for AscendingVec<T> {
    type Target = Vec<T>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
