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

    // We can also verify it's a future by trying to poll it (won't run due to todo!())
    // This ensures the async transformation is correct
    fn is_future<T: core::future::Future>(_: &T) {}
    is_future(&future);
}
