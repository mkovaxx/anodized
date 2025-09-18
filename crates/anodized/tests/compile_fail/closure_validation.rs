use anodized::spec;

#[spec(
    // Should fail: closure has 2 arguments
    ensures: |x, y| x > 0,
)]
fn returns_positive() -> i32 {
    42
}

#[spec(
    // Should fail: closure has 0 arguments
    ensures: || true,
)]
fn returns_something() -> i32 {
    42
}

fn main() {}
