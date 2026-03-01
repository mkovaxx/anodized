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
 // This ensures the output is positive
         ensures: *output > x,
    // This is a precondition
    requires: x > 0,
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
    // Balance must be positive before withdrawal
    captures: *balance as initial,
    // Balance must be positive before withdrawal
    requires: *balance > 0,
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
requires:   value != 0,
        // Must return positive result
        ensures  : *output > 0,
    )]
    fn inverse(&self, value: f64) -> f64 {
        1.0 / value
    }

    #[spec(
                // Captures the base value
            captures : self.base as initial_base,
    // Base must be positive
        requires: self.base > 0,
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
        // Must capture the data
         captures:  data.clone() as snapshot,
             // Data must not be empty
        requires: !data.is_empty(),
    // Result reflects the snapshot
        ensures: *output == snapshot.len(),
    )]
    fn process(&self, data: &[u8]) -> usize;
}
