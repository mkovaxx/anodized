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
fn instrumented_function(log: &mut Vec<&'static str>) {
    log.push("body");
}

#[cfg(not(feature = "backend-no-checks"))]
#[test]
fn test_execution_order() {
    let mut log = Vec::new();
    instrumented_function(&mut log);

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
