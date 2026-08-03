use anodized::spec;

#[spec(
    // Should fail: postcondition closures must bind exactly one output value.
    ensures: |x, y| x > 0,
)]
fn returns_positive() -> i32 {
    42
}

#[spec(
    // Should fail: postcondition closures must bind exactly one output value.
    ensures: || true,
)]
fn returns_something() -> i32 {
    42
}

#[spec(
    // Should fail: postcondition closures must return `bool`.
    ensures: |output| 42,
)]
fn some_func() -> i32 {
    todo!()
}

#[spec(
    // Should fail: cannot re-bind the postcondition's input.
    ensures: |output| [
        output >= 42,
        |answer| answer <= 42,
    ],
)]
fn some_func_2() -> i32 {
    todo!()
}

fn main() {}
