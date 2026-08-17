//! In the future, an `anodized-fuzz` tool should generate this harness.
#![no_main]

use anodized::result::{try_call, PostError, PreError};
use libfuzzer_sys::arbitrary::{self, Arbitrary, Error, Unstructured};
use libfuzzer_sys::{fuzz_target, Corpus};

fuzz_target!(|inputs: Inputs<i32>| -> Corpus {
    // Use Anodized's `try_call!` macro to defer acting on spec violations.
    let result = try_call! {
        bin_search::bin_search(&inputs.seq, &inputs.value)
    };
    match result {
        // Successful call.
        Ok(_) => Corpus::Keep,
        // When preconditions are violated, reject the input.
        Err(PreError) => Corpus::Reject,
        // When postconditions are violated, panic to signal a counter-example.
        Err(PostError(output)) => {
            eprintln!("inputs:");
            dbg!(inputs.seq);
            dbg!(inputs.value);
            dbg!(output);
            panic!("postcondition failed");
        }
    }
});

#[derive(Debug, Arbitrary)]
struct Inputs<T: Ord> {
    seq: AscendingVec<T>,
    value: T,
}

/// Helper newtype to more efficiently generate valid inputs.
#[derive(Debug, Clone)]
struct AscendingVec<T>(Vec<T>);

impl<'a, T: Ord + Arbitrary<'a>> Arbitrary<'a> for AscendingVec<T> {
    fn arbitrary(unst: &mut Unstructured<'a>) -> Result<Self, Error> {
        let mut raw_seq = <Vec<T> as Arbitrary>::arbitrary(unst)?;
        raw_seq.sort();
        Ok(Self(raw_seq))
    }
}

impl<T> std::ops::Deref for AscendingVec<T> {
    type Target = Vec<T>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
