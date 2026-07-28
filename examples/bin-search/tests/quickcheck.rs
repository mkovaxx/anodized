use anodized::result::{PostError, PreError, try_call};

use quickcheck::{Arbitrary, Gen, QuickCheck, TestResult};

/// Generate random inputs and assert that the precondition implies the postcondition.
#[test]
fn test_spec() {
    QuickCheck::new()
        .tests(10_000)
        .quickcheck(test_spec_property as fn(Inputs<i32>) -> TestResult);
}

fn test_spec_property(inputs: Inputs<i32>) -> TestResult {
    // Use Anodized's `try_call!` macro to defer acting on spec violations.
    let result = try_call! {
        bin_search::bin_search(&inputs.seq, &inputs.value)
    };
    match result {
        // Successful call.
        Ok(_) => TestResult::passed(),
        // When preconditions are violated, reject the input.
        Err(PreError(_)) => TestResult::discard(),
        // When postconditions are violated, fail to signal a counter-example.
        Err(PostError(output, errors)) => {
            eprintln!("inputs:");
            dbg!(inputs.seq);
            dbg!(inputs.value);
            dbg!(output);
            TestResult::error(format!("postcondition failed:{errors}"))
        }
    }
}

#[derive(Debug, Clone)]
struct Inputs<T: Ord> {
    seq: AscendingVec<T>,
    value: T,
}

impl<T: Ord + Arbitrary> Arbitrary for Inputs<T> {
    fn arbitrary(g: &mut Gen) -> Self {
        Self {
            seq: AscendingVec::arbitrary(g),
            value: T::arbitrary(g),
        }
    }
}

/// Helper newtype to more efficiently generate valid inputs.
#[derive(Debug, Clone)]
struct AscendingVec<T>(Vec<T>);

impl<T: Ord + Arbitrary> Arbitrary for AscendingVec<T> {
    fn arbitrary(g: &mut Gen) -> Self {
        let mut values = Vec::<T>::arbitrary(g);
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
