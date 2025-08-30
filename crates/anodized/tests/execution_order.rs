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
        log.push("clone1") as _clone1,
        log.push("clone2") as _clone2,
    ],
    ensures: [
        log.push("ensures1") == (),
        log.push("ensures2") == (),
    ],
)]
fn instrumented_function(log: &mut Vec<&'static str>) {
    log.push("body");
}

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
            "clone1",
            "clone2",
            "body",
            "maintains1",
            "maintains2",
            "ensures1",
            "ensures2",
        ]
    );
}
