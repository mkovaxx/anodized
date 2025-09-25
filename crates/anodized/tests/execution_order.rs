use anodized::spec;

#[spec(
    requires: [
        log.push("requires1") == (),
        log.push("requires2") == (),
    ],
    maintains: [
        log.push("maintains1") == (),
        log.push("maintains2") == (),
    ],
    captures: [
        log.push("captures1") as _alias1,
        log.push("captures2") as _alias2,
    ],
    ensures: [
        log.push("ensures1") == (),
        log.push("ensures2") == (),
    ],
)]
fn func(log: &mut Vec<&'static str>) {
    log.push("body");
    return;
}

#[cfg(not(feature = "backend-no-checks"))]
#[test]
fn test_execution_order() {
    let mut log = Vec::new();
    func(&mut log);

    // Verify the exact execution order
    assert_eq!(
        log,
        vec![
            "requires1",
            "requires2",
            "maintains1",
            "maintains2",
            "captures1",
            "captures2",
            "body",
            "maintains1",
            "maintains2",
            "ensures1",
            "ensures2",
        ]
    );
}

#[spec(
    requires: [
        log.push("requires1") == (),
        log.push("requires2") == (),
    ],
    maintains: [
        log.push("maintains1") == (),
        log.push("maintains2") == (),
    ],
    captures: [
        log.push("captures1") as _alias1,
        log.push("captures2") as _alias2,
    ],
    ensures: [
        log.push("ensures1") == (),
        log.push("ensures2") == (),
    ],
)]
async fn async_func(log: &mut Vec<&'static str>) {
    log.push("body");
    return;
}

#[cfg(not(feature = "backend-no-checks"))]
#[test]
fn test_async_execution_order() {
    let mut log = Vec::new();
    let future = async_func(&mut log);

    fn is_future<T: core::future::Future>(_: &T) {}
    is_future(&future);

    // TODO: Verify the exact execution order

    // assert_eq!(
    //     log,
    //     vec![
    //         "requires1",
    //         "requires2",
    //         "maintains1",
    //         "maintains2",
    //         "captures1",
    //         "captures2",
    //         "body",
    //         "maintains1",
    //         "maintains2",
    //         "ensures1",
    //         "ensures2",
    //     ]
    // );
}
