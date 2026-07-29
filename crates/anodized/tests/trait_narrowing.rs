use anodized::spec;

//////////////////////////
// Test runtime checks. //
//////////////////////////

#[spec]
trait MinFinder<T: PartialOrd> {
    #[spec(
        total,
        requires: [
            !input.is_empty(),
        ],
        inspects: ref output,
        ensures: [
            input.iter().all(|item| output <= item),
            input.iter().any(|item| output == item) || input.is_empty(),
        ],
    )]
    fn find_min(input: &[T]) -> T;
}

pub struct ValidNarrowing;

#[spec]
impl MinFinder<f32> for ValidNarrowing {
    #[spec(
        // Stronger than trait qualifiers: is also `pure` (`deterministic` and `effectfree`).
        functional,
        // Weaker than trait precondition: allows `input` to be empty.
        requires: [],
        inspects: ref output,
        // Stronger than trait postcondition: clarifies what to output when `input` is empty.
        ensures: [
            input.iter().all(|item| output <= item),
            input.iter().any(|item| output == item)
                || (input.is_empty() && *output == f32::INFINITY),
        ],
    )]
    #[warn(unused_comparisons)]
    fn find_min(input: &[f32]) -> f32 {
        let mut min = f32::INFINITY;
        for item in input.iter().copied() {
            if item < min {
                min = item;
            }
        }
        min
    }
}

pub struct StrongerImplPre;

#[spec]
impl MinFinder<i32> for StrongerImplPre {
    #[spec(
        total,
        // INVALID - Stronger than trait precondition: requires sorted `input`.
        requires: [
            !input.is_empty(),
            input.is_sorted(),
        ],
        inspects: ref output,
        ensures: [
            input.iter().all(|item| output <= item),
            input.iter().any(|item| output == item) || input.is_empty(),
        ],
    )]
    fn find_min(input: &[i32]) -> i32 {
        input[0]
    }
}

pub struct WeakerImplPost;

#[spec]
impl MinFinder<u32> for WeakerImplPost {
    #[spec(
        total,
        requires: [
            !input.is_empty(),
        ],
        inspects: ref output,
        // INVALID - Weaker than trait postcondition: `input` may be ignored completely.
        ensures: [
            input.iter().all(|item| output <= item),
        ],
    )]
    fn find_min(input: &[u32]) -> u32 {
        let _ = input;
        0
    }
}

#[test]
fn runtime_allows_valid_narrowing() {
    // NOTE: The trait's runtime checks are active even when the concrete type is statically known.
    let seq = [5.0, -42.0, std::f32::consts::PI];
    assert_eq!(ValidNarrowing::find_min(&seq), -42.0);
}

#[cfg(all(anodized_print, anodized_panic))]
#[test]
#[should_panic(expected = "precondition failed:\
\n    input.is_sorted()")]
fn runtime_rejects_stronger_impl_precondition() {
    // NOTE: The trait's runtime checks are active even when the concrete type is statically known.
    let seq = [5, -42, 3];
    assert_eq!(StrongerImplPre::find_min(&seq), -42);
}

#[cfg(all(anodized_print, anodized_panic))]
#[test]
#[should_panic(expected = "postcondition failed:\
\n    input.iter().any(| item | output == item) || input.is_empty()")]
fn runtime_rejects_weaker_impl_postcondition() {
    // NOTE: The trait's runtime checks are active even when the concrete type is statically known.
    let seq = [5, 42, 3];
    assert_eq!(WeakerImplPost::find_min(&seq), 3);
}

//////////////////////////////////////////////////////
// Smoke-test instrumentation on more complex code. //
//////////////////////////////////////////////////////

#[spec]
pub trait Matrix<T> {
    fn count_rows(&self) -> usize;
    fn count_cols(&self) -> usize;

    #[spec(
        requires: [
            input.count_rows() == self.count_cols(),
        ],
        inspects: ref output,
        ensures: [
            output.count_rows() == self.count_rows(),
            output.count_cols() == input.count_cols(),
        ],
    )]
    fn mul<Input: Matrix<T>, Output: Matrix<T>>(&self, input: &Input) -> Output;
}

pub struct DiagonalMatrix<T>(Vec<T>);

#[spec]
impl<T> Matrix<T> for DiagonalMatrix<T> {
    #[spec(inspects: output, ensures: output == self.count_cols())]
    fn count_rows(&self) -> usize {
        self.0.len()
    }

    #[spec(inspects: output, ensures: output == self.count_rows())]
    fn count_cols(&self) -> usize {
        self.0.len()
    }

    #[spec(
        requires: [
            input.count_rows() == self.count_cols(),
        ],
        inspects: ref output,
        ensures: [
            output.count_rows() == input.count_rows(),
            output.count_cols() == input.count_cols(),
        ],
    )]
    fn mul<Input: Matrix<T>, Output: Matrix<T>>(&self, input: &Input) -> Output {
        let _ = input;
        todo!()
    }
}
