use anodized::spec;

// This test verifies that clone aliases are not accessible in the function body
// or in requires/maintains conditions

#[spec(
    captures: [
        x as old_x,
        y as old_y,
    ],
    ensures: [
        old_x == 5,  // OK: aliases are available in ensures
        old_y == 10, // OK: aliases are available in ensures
    ],
)]
fn test_clones_not_in_body(x: i32, y: i32) -> i32 {
    // Clone aliases should not be accessible in function body
    let a = old_x; // Should be an error: `old_x` must not be in scope.
    let b = old_y; // Should be an error: `old_y` must not be in scope.
    x + y
}

#[spec(
    requires: [
        old_x > 0, // Should be an error: `old_x` must not be in scope.
    ],
    captures: [
        x as old_x,
    ],
)]
fn test_clones_not_in_requires(x: i32) {
    // Function body
}

#[spec(
    maintains: [
        old_x > 0, // Should be an error: `old_x` must not be in scope.
    ],
    captures: [
        x as old_x,
    ],
)]
fn test_clones_not_in_maintains(x: i32) {
    // Function body
}

fn main() {}
