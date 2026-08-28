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
        ensures: |ref output| [
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
        // Stronger than trait postcondition: clarifies what to output when `input` is empty.
        ensures: |ref output| [
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
        ensures: |ref output| [
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
        // INVALID - Weaker than trait postcondition: `input` may be ignored completely.
        ensures: |ref output| [
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
#[should_panic(expected = "precondition failed")]
fn runtime_rejects_stronger_impl_precondition() {
    // NOTE: The trait's runtime checks are active even when the concrete type is statically known.
    let seq = [5, -42, 3];
    assert_eq!(StrongerImplPre::find_min(&seq), -42);
}

#[cfg(all(anodized_print, anodized_panic))]
#[test]
#[should_panic(expected = "postcondition failed")]
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
    fn shape(&self) -> MatrixShape;

    #[spec(
        requires: rhs.shape().rows == self.shape().cols,
        ensures: |output| [
            output.shape().rows == self.shape().rows,
            output.shape().cols == rhs.shape().cols,
        ],
    )]
    fn mul<Input: Matrix<T>, Output: Matrix<T>>(&self, rhs: &Input) -> Output;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MatrixShape {
    pub rows: usize,
    pub cols: usize,
}

pub struct DiagonalMatrix<T>(Vec<T>);

#[spec]
impl<T> Matrix<T> for DiagonalMatrix<T> {
    #[spec(ensures: |MatrixShape { rows, cols }| rows == cols)]
    fn shape(&self) -> MatrixShape {
        let n = self.0.len();
        MatrixShape { rows: n, cols: n }
    }

    #[spec(
        requires: rhs.shape().rows == self.shape().cols,
        ensures: |output| output.shape() == rhs.shape(),
    )]
    fn mul<Input: Matrix<T>, Output: Matrix<T>>(&self, rhs: &Input) -> Output {
        let _ = rhs;
        todo!()
    }
}
