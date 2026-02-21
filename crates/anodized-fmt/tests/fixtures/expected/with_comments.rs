use anodized::spec;

#[spec(
    // This is a precondition
    requires: x > 0,
    // This ensures the output is positive
    ensures: *output > x,
)]
fn double_positive(x: i32) -> i32 {
    x * 2
}

#[spec(
    // First parameter must be positive
    requires: a > 0,
    // Result is positive
    ensures: *output > 0,
)]
fn add(a: i32, b: i32) -> i32 {
    a + b
}

// Test: Complex capture with multiple patterns
// Note: Comments inside nested structures (like arrays) are not preserved
// in their original positions. They will be moved to before the next top-level
// spec arg. This is a known limitation of the current implementation.
#[spec(
    // Comment requires
    requires: active,
    captures: [
        values as [first , second , third],
        state.clone() as State { active , count },
    ],
    // Capture 1st
    // Capture 2nd
    ensures: first + second + third == count,
)]
fn complex_capture_multiple(values: [i32; 3], state: &State) -> bool {
    todo!()
}
