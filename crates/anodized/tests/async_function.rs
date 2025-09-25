use anodized::spec;

#[spec(
    requires: x.is_finite(),
    ensures: output + output == x,
)]
async fn half_async(x: f32) -> f32 {
    todo!()
}

#[test]
fn async_function_compiles() {
    let future = half_async(5.0);

    fn is_future<T: core::future::Future>(_: &T) {}
    is_future(&future);
}
