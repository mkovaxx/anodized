use anodized::spec;

#[spec(
    requires: log.push("requires") == (),
    maintains: log.push("maintains") == (),
    clones: log.push("clone") as _clone,
    ensures: log.push("ensures") == (),
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
        vec!["requires", "maintains", "clone", "body", "maintains", "ensures"]
    );
}

#[spec(
    requires: [
        log.push("req1") == (),
        log.push("req2") == (),
    ],
    clones: [
        log.push("clone1") as _clone1,
        log.push("clone2") as _clone2,
    ],
    ensures: [
        log.push("ens1") == (),
        log.push("ens2") == (),
    ],
)]
fn multiple_conditions(log: &mut Vec<&'static str>) {
    log.push("body");
}

#[test]
fn test_multiple_conditions_order() {
    let mut log = Vec::new();
    multiple_conditions(&mut log);

    assert_eq!(
        log,
        vec!["req1", "req2", "clone1", "clone2", "body", "ens1", "ens2"]
    );
}

// Test that clones are evaluated in a single expression (left-to-right)
#[spec(
    clones: [
        log.push("clone1") as _clone1,
        log.push("clone2") as _clone2,
    ],
    ensures: () == (),
)]
fn clone_evaluation_order(log: &mut Vec<&'static str>) {
    log.push("body");
}

#[test]
fn test_clone_evaluation_order() {
    let mut log = Vec::new();
    clone_evaluation_order(&mut log);

    assert_eq!(log, vec!["clone1", "clone2", "body"]);
}
