use anodized::spec;

#[spec(
    requires: x > 0,
    ensures: output == x * 2,
)]
async fn double_async(x: i32) -> i32 {
    todo!()
}

#[test]
fn test_async_function_compiles() {
    let _ = double_async(5);
}
