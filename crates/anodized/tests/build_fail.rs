#![cfg_attr(anodized_charon, feature(register_tool))]
#![cfg_attr(anodized_charon, register_tool(charon))]

#[test]
fn qualifier_narrowing_fails_to_compile() {
    let output = std::process::Command::new("cargo")
        .arg("build")
        .current_dir("tests/build_fail/trait_narrowing")
        .output()
        .expect("failed to run cargo");

    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(!output.status.success(), "expected compilation to fail");
    assert!(
        stderr.contains("the qualifiers on the impl `WeakerImplQualifiers < T >::find_min` cannot be weaker than the qualifiers on the trait `MinFinder < T >::find_min`"),
        "expected specific error message, got:\n{stderr}"
    );
}
