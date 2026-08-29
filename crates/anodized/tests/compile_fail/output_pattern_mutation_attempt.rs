use anodized::spec;

#[spec(ensures: |mut a| {
    *a = 999;
    true
})]
fn sort_pair(val: &mut i32) -> &mut i32 {
    val
}

fn main() {}
