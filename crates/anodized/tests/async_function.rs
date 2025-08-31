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
    let future = double_async(5);

    fn is_future<T: core::future::Future>(_: &T) {}
    is_future(&future);
}
