// Tests attempting to mutate return value in a postcondition should give a sensible error
// Current behavior is bad: The macro panics with a confusing error.
use anodized::spec;

#[spec(ensures: |mut a| {
    *a = 999;
    true
})]
fn sort_pair(val: &mut i32) -> &mut i32 {
    val
}

fn main() {}
