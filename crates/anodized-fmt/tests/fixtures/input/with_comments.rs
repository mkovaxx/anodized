use anodized::spec;

#[spec(
 // This ensures the output is positive
         ensures: *output > x,
    // This is a precondition
    requires: x > 0,
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

// Test: capture pattern matches tuples
#[spec(
    // Capture the point coordinates
    captures: point as (x , y , z),
    // All coordinates must be less than 100
    ensures: x < 100 && y < 100 && z < 100,
)]
fn validate_point(point: (i32, i32, i32)) -> bool {
    todo!()
}

// Test: Capture with all spec clauses
#[spec(
             // Bind the result
        binds: result,
    // Capture initial balance
    captures: *balance as initial,
    // Balance must be positive before withdrawal
    requires: *balance > 0,
    // Ensure correct calculation
    ensures: result == initial - amount,
)]
fn withdraw_with_capture(balance: &mut u64, amount: u64) -> u64 {
    todo!()
}
