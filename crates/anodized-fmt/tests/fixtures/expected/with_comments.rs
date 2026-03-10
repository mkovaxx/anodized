use anodized::spec;

// Basic comment preservation
#[spec(
    // This is a comment for the requires clause
    requires: x > 0,
    // This is a comment for the ensures clause
    ensures: *output > 0,
)]
fn foo(x: i32) -> i32 {
    x + 1
}

// Comment reordering test
#[spec(
    // This is a precondition
    requires: x > 0,
    // This ensures the output is positive
    ensures: *output > x,
)]
fn double_positive(x: i32) -> i32 {
    x * 2
}

// Multiple comments for same arg
#[spec(
    // First parameter must be positive
    // Another comment about the precondition
    requires: a > 0,
    // Result is positive
    ensures: *output > 0,
)]
fn add(a: i32, b: i32) -> i32 {
    a + b
}

// Test: capture pattern matches tuples
#[spec(
    // Capture the point coordinates
    captures: point as (x, y, z),
    // All coordinates must be less than 100
    ensures: x < 100 && y < 100 && z < 100,
)]
fn validate_point(point: (i32, i32, i32)) -> bool {
    todo!()
}

// Test: Capture with all spec clauses
#[spec(
    // Balance must be positive before withdrawal
    requires: *balance > 0,
    // Capture initial balance
    // Balance must be positive before withdrawal
    captures: *balance as initial,
    // Bind the result
    binds: result,
    // Ensure correct calculation
    // Result should be initial balance minus amount
    ensures: result == initial - amount,
)]
fn withdraw_with_capture(balance: &mut u64, amount: u64) -> u64 {
    todo!()
}

// Test: Impl method with comments
struct Calculator;

impl Calculator {
    // Method in impl block
    #[spec(
        // Value must not be zero
        requires: value != 0,
        // Must return positive result
        ensures: *output > 0,
    )]
    fn inverse(&self, value: f64) -> f64 {
        1.0 / value
    }

    #[spec(
        // Base must be positive
        requires: self.base > 0,
        // Captures the base value
        captures: self.base as initial_base,
        // Result combines base and value
        ensures: *output == initial_base + value,
    )]
    fn add_to_base(&self, value: i32) -> i32 {
        todo!()
    }
}

// Test: Trait method with comments
trait Validator {
    // Trait method declaration
    #[spec(
        // Input must be in valid range
        requires: input >= 0 && input <= 100,
        // Output indicates validity
        ensures: *output == true,
    )]
    fn validate(&self, input: i32) -> bool;

    #[spec(
        // Data must not be empty
        requires: !data.is_empty(),
        // Must capture the data
        captures: data.clone() as snapshot,
        // Result reflects the snapshot
        ensures: *output == snapshot.len(),
    )]
    fn process(&self, data: &[u8]) -> usize;
}

// Multiple clauses in requires without internal comments
#[spec(
    // Preconditions for all three parameters
    requires: x > 0 && y > 0 && z > 0,
    ensures: *output > 0,
)]
fn requires_multiple_clauses(x: i32, y: i32, z: i32) -> i32 {
    todo!()
}

// Multiple clauses in ensures without internal comments
#[spec(
    requires: x > 0,
    // Post-conditions about the result
    ensures: *output > x && *output < 100,
)]
fn ensures_multiple_clauses(x: i32) -> i32 {
    todo!()
}

// Multiple clauses in maintains without internal comments
#[spec(
    // Invariants for the method
    maintains: self.count >= 0 && self.active,
    ensures: *output == self.count,
)]
fn maintains_multiple_clauses(&self) -> i32 {
    todo!()
}

// Array syntax for multiple ensures clauses (valid syntax, unusual pattern)
#[spec(
    requires: x > 0,
    // Multiple post-conditions using array syntax
    ensures: [
        *output > x,
        *output < 100,
    ],
)]
fn ensures_array_syntax(x: i32) -> i32 {
    todo!()
}

// Array syntax for multiple requires clauses
#[spec(
    // Multiple preconditions using array syntax
    requires: [
        x > 0,
        y > 0,
        z > 0,
    ],
    ensures: *output > 0,
)]
fn requires_array_syntax(x: i32, y: i32, z: i32) -> i32 {
    todo!()
}

// Mixed: some clauses with array syntax, some without
#[spec(
    requires: x > 0,
    // Capture initial values
    captures: [
        x as old_x,
        y as old_y,
    ],
    // Multiple post-conditions
    ensures: [
        *output > old_x,
        *output > old_y,
    ],
)]
fn mixed_array_syntax(x: i32, y: i32) -> i32 {
    todo!()
}
