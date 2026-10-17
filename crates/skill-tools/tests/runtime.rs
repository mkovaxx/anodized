//! Asserts the runtime behavior `diagnostics.md` documents actually happens.
//!
//! The fence harness builds with no `anodized_*` configuration, where checks are embedded but
//! inert. Nothing there can catch a claim about what a violation *does*, so these cases build
//! with checks live and run them.

use std::path::Path;

use skill_tools::scratch;

const CHECKS_ON: &str = "--cfg anodized_print --cfg anodized_panic";

// Each case gets its own scratch directory. They run in parallel and would otherwise
// overwrite one another's source file mid-build.

fn case(body: &str) -> String {
    format!("use anodized::spec;\n\n{body}\n")
}

#[test]
fn a_violated_precondition_panics() -> skill_tools::Result<()> {
    let project = scratch::Project::with_rustflags(
        Path::new(env!("CARGO_TARGET_TMPDIR")),
        "runtime-precondition",
        Some(CHECKS_ON),
    )?;
    let outcome = project.test(&case(
        r#"
#[spec(requires: divisor != 0)]
pub fn divide(dividend: u32, divisor: u32) -> u32 {
    dividend / divisor
}

#[cfg(test)]
mod tests {
    #[test]
    #[should_panic(expected = "precondition failed")]
    fn rejects_zero() {
        let _ = super::divide(1, 0);
    }
}
"#,
    ))?;
    assert!(outcome.succeeded, "{}", outcome.stderr);
    Ok(())
}

#[test]
fn a_violated_postcondition_panics() -> skill_tools::Result<()> {
    let project = scratch::Project::with_rustflags(
        Path::new(env!("CARGO_TARGET_TMPDIR")),
        "runtime-postcondition",
        Some(CHECKS_ON),
    )?;
    let outcome = project.test(&case(
        r#"
#[spec(ensures: |output| output > 0)]
pub fn zero() -> u32 {
    0
}

#[cfg(test)]
mod tests {
    #[test]
    #[should_panic(expected = "postcondition failed")]
    fn breaks_its_promise() {
        let _ = super::zero();
    }
}
"#,
    ))?;
    assert!(outcome.succeeded, "{}", outcome.stderr);
    Ok(())
}

#[test]
fn a_satisfied_spec_is_silent() -> skill_tools::Result<()> {
    let project = scratch::Project::with_rustflags(
        Path::new(env!("CARGO_TARGET_TMPDIR")),
        "runtime-satisfied",
        Some(CHECKS_ON),
    )?;
    let outcome = project.test(&case(
        r#"
#[spec(
    requires: divisor != 0,
    ensures: |output| output <= dividend,
)]
pub fn divide(dividend: u32, divisor: u32) -> u32 {
    dividend / divisor
}

#[cfg(test)]
mod tests {
    #[test]
    fn accepts_valid_input() {
        assert_eq!(super::divide(6, 2), 3);
    }
}
"#,
    ))?;
    assert!(outcome.succeeded, "{}", outcome.stderr);
    Ok(())
}
