use anodized::spec;

// This test verifies that capture aliases are not accessible in the function body
// or in requires/maintains conditions

#[spec(
    captures: [
        x as old_x,
        y as old_y,
    ],
    ensures: [
        // OK: aliases are available in ensures.
        old_x == 5,
        // OK: aliases are available in ensures.
        old_y == 10,
    ],
)]
fn captures_not_in_body(x: i32, y: i32) -> i32 {
    // Should be an error: `old_x` must not be in scope.
    let a = old_x;
    // Should be an error: `old_y` must not be in scope.
    let b = old_y;
    x + y
}

#[spec(
    requires: [
        // Should be an error: `old_x` must not be in scope.
        old_x > 0,
    ],
    captures: [
        x as old_x,
    ],
)]
fn captures_not_in_requires(x: i32) {
    // Function body
}

#[spec(
    maintains: [
        // Should be an error: `old_x` must not be in scope.
        old_x > 0,
    ],
    captures: [
        x as old_x,
    ],
)]
fn captures_not_in_maintains(x: i32) {
    // Function body
}

fn main() {}
