use anodized::spec;

#[spec(
    requires: {
        log.push("requires");
        true
    },
    maintains: {
        log.push("maintains-pre");
        true
    },
    clones: log.push("clone") as _,
    ensures: {
        log.push("ensures");
        true
    },
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
            "requires",          // Preconditions first
            "maintains-pre",     // Pre-invariants second
            "clone",            // Clones third (before body)
            "body",             // Function body fourth
            "maintains-pre",    // Post-invariants fifth (reuses same condition)
            "ensures",          // Postconditions last
        ]
    );
}

#[spec(
    requires: [
        {
            log.push("req1");
            true
        },
        {
            log.push("req2");
            true
        },
    ],
    clones: [
        log.push("clone1") as _,
        log.push("clone2") as _,
    ],
    ensures: [
        {
            log.push("ens1");
            true
        },
        {
            log.push("ens2");
            true
        },
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
        vec![
            "req1",    // First precondition
            "req2",    // Second precondition
            "clone1",  // First clone
            "clone2",  // Second clone
            "body",    // Function body
            "ens1",    // First postcondition
            "ens2",    // Second postcondition
        ]
    );
}

// Test that clones are evaluated in a single expression (left-to-right)
#[spec(
    clones: [
        log.push("clone1") as _,
        log.push("clone2") as _,
    ],
    ensures: true,
)]
fn clone_evaluation_order(log: &mut Vec<&'static str>) {
    log.push("body");
}

#[test]
fn test_clone_evaluation_order() {
    let mut log = Vec::new();
    clone_evaluation_order(&mut log);
    
    assert_eq!(
        log,
        vec![
            "clone1",  // First clone evaluates
            "clone2",  // Second clone evaluates (sees clone1's effect)
            "body",    // Then body executes
        ]
    );
}