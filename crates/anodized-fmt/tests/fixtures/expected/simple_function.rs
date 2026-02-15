use anodized::spec;

#[spec(
    requires: x > 0,
    ensures: *output > 0,
)]
fn simple_function(x: i32) -> i32 {
    x + 1
}
