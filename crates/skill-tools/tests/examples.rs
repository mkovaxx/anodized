//! Compiles every Rust block in the skill.
//!
//! A wrong example in an agent skill is worse than no example, because it is copied verbatim.
//! `REFERENCE.md` drifted exactly this way, teaching a form that stopped compiling in 0.6.

mod drivers;

use std::path::Path;

use skill_tools::scratch;

use skill_tools::fences::{Fence, Kind};
use skill_tools::{Result, SKILL_FILES, VERIFIED_DOCS, read, read_skill_file, repo_root};

fn fences() -> Result<Vec<Fence>> {
    let mut all = Vec::new();
    for name in SKILL_FILES {
        all.extend(skill_tools::fences::extract(name, &read_skill_file(name)?)?);
    }
    let root = repo_root()?;
    for path in VERIFIED_DOCS {
        all.extend(skill_tools::fences::extract(
            path,
            &read(&root.join(path))?,
        )?);
    }
    Ok(all)
}

#[test]
fn every_fence_uses_a_known_tag() -> Result<()> {
    fences()?;
    Ok(())
}

#[test]
fn every_fragment_is_labelled() -> Result<()> {
    for fence in fences()?.iter().filter(|f| f.kind == Kind::Fragment) {
        assert!(
            fence.is_labelled_fragment(),
            "{fence}: an `ignore` block must open with `// fragment`; wrong-form examples \
             belong in a `compile_fail` block instead"
        );
    }
    Ok(())
}

#[test]
fn every_rust_fence_compiles() -> Result<()> {
    let project = scratch::Project::new(Path::new(env!("CARGO_TARGET_TMPDIR")), "examples")?;
    for fence in &fences()? {
        match fence.kind {
            Kind::Pass | Kind::CompileOnly => {
                let outcome = project.build(&fence.as_program())?;
                assert!(
                    outcome.succeeded,
                    "{fence}: expected this to compile, but it did not:\n{}",
                    outcome.stderr
                );
            }
            Kind::CompileFail => {
                let expected = fence.expected_error()?;
                let outcome = project.build(&fence.as_program())?;
                assert!(
                    !outcome.succeeded,
                    "{fence}: expected {expected}, but it compiled"
                );
                assert!(
                    outcome.stderr.contains(&expected),
                    "{fence}: expected {expected}, got:\n{}",
                    outcome.stderr
                );
            }
            Kind::Fragment | Kind::Other => {}
        }
    }
    Ok(())
}

/// Every runnable example is either driven or deliberately listed as undriven.
///
/// Without this, coverage would depend on whether an author remembered to write a driver, and
/// a new example could join the document exercised by nothing at all.
#[test]
fn every_example_is_driven_or_declared_undriven() -> Result<()> {
    for fence in fences()?.iter().filter(|f| f.kind == Kind::Pass) {
        if !drivers::DRIVEN_FILES.contains(&fence.file.as_str()) || fence.is_self_exercising() {
            continue;
        }
        let driven = drivers::DRIVERS
            .iter()
            .any(|d| d.file == fence.file && fence.body.contains(d.defines));
        let declared = drivers::UNDRIVEN
            .iter()
            .any(|(file, defines)| *file == fence.file && fence.body.contains(defines));
        assert!(
            driven || declared,
            "{fence}: the example under `{}` is never run. Add a driver in \
             tests/drivers.rs, or list it in UNDRIVEN with a reason.",
            fence.heading()
        );
    }
    Ok(())
}

/// Every `UNDRIVEN` entry names exactly one example, and never one that is also driven.
///
/// Without this the list is an unguarded escape hatch: a single loose substring such as
/// `use anodized::spec` would declare every present and future example deliberately undriven,
/// and an entry left behind by a deleted example would linger unnoticed.
#[test]
fn every_undriven_entry_names_exactly_one_example() -> Result<()> {
    let all = fences()?;
    for (file, defines) in drivers::UNDRIVEN {
        let matched: Vec<_> = all
            .iter()
            .filter(|f| f.kind == Kind::Pass && f.file == *file && f.body.contains(defines))
            .collect();
        assert_eq!(
            matched.len(),
            1,
            "UNDRIVEN entry `{defines}` in {file} matches {} examples; it must name exactly one",
            matched.len()
        );
        let also_driven = drivers::DRIVERS
            .iter()
            .any(|d| d.file == *file && matched[0].body.contains(d.defines));
        assert!(
            !also_driven,
            "`{defines}` in {file} is both driven and declared undriven; keep one"
        );
    }
    Ok(())
}

/// A driven document may not park an example beyond the reach of the drivers.
///
/// `Kind::CompileOnly` is compiled exactly like `Kind::Pass`, so retagging a fence
/// `rust, no_run` would quietly drop it from the driven-or-declared requirement.
#[test]
fn driven_files_contain_no_compile_only_examples() -> Result<()> {
    for fence in fences()?.iter().filter(|f| f.kind == Kind::CompileOnly) {
        assert!(
            !drivers::DRIVEN_FILES.contains(&fence.file.as_str()),
            "{fence}: `no_run` in a driven document exempts the example from being run; \
             tag it `rust` and give it a driver"
        );
    }
    Ok(())
}

/// Runs every driven example over the inputs its driver sweeps, with checks enabled.
#[test]
fn driven_examples_satisfy_their_own_specs() -> Result<()> {
    let project = scratch::Project::with_rustflags(
        Path::new(env!("CARGO_TARGET_TMPDIR")),
        "examples-driven",
        Some("--cfg anodized_print --cfg anodized_panic"),
    )?;
    let all = fences()?;
    for driver in drivers::DRIVERS {
        let matched: Vec<_> = all
            .iter()
            .filter(|f| {
                f.kind == Kind::Pass && f.file == driver.file && f.body.contains(driver.defines)
            })
            .collect();
        let fence = match matched.as_slice() {
            [one] => *one,
            [] => {
                return Err(format!(
                    "driver `{}` in {} matches no example; its `defines` is stale",
                    driver.defines, driver.file
                )
                .into());
            }
            many => {
                return Err(format!(
                    "driver `{}` in {} matches {} examples; make `defines` unique",
                    driver.defines,
                    driver.file,
                    many.len()
                )
                .into());
            }
        };
        assert!(
            !fence.is_self_exercising(),
            "{fence}: `{}` has both a driver and its own `fn main`; keep one",
            driver.defines
        );
        let program = format!(
            "{}\n{}\n{}\n",
            fence.as_program(),
            drivers::PRELUDE,
            driver.main
        );
        let outcome = project.run(&program)?;
        assert!(
            outcome.succeeded,
            "{fence}: `{}` trips its own spec:\n{}",
            driver.defines, outcome.stderr
        );
    }
    Ok(())
}

/// Runs the examples that exercise themselves, with checks enabled.
///
/// The compile pass proves a spec is well-formed; it cannot prove the spec is true of the code
/// beside it. An example here once promised `output >= items.len()` from a sum that is zero
/// for an all-zero slice: it compiled, and panicked on the first honest input. Any block that
/// carries a `fn main` is run under `anodized_print` and `anodized_panic`, so a condition that
/// is false of its own example fails the suite.
#[test]
fn self_exercising_examples_satisfy_their_own_specs() -> Result<()> {
    let project = scratch::Project::with_rustflags(
        Path::new(env!("CARGO_TARGET_TMPDIR")),
        "examples-checked",
        Some("--cfg anodized_print --cfg anodized_panic"),
    )?;
    let mut ran = 0;
    for fence in fences()?.iter().filter(|f| f.kind == Kind::Pass) {
        if !fence.is_self_exercising() {
            continue;
        }
        let outcome = project.run(&fence.as_program())?;
        assert!(
            outcome.succeeded,
            "{fence}: this example trips its own spec when run:\n{}",
            outcome.stderr
        );
        ran += 1;
    }
    assert!(
        ran > 0,
        "no example exercises itself; the runtime check is inert"
    );
    Ok(())
}

#[test]
fn examples_cover_every_item_kind() -> Result<()> {
    let examples = read_skill_file("examples.md")?;
    for heading in [
        "## Free function",
        "## Inherent `impl`",
        "## Trait, with a narrowing impl",
        "## `while` loop",
        "## `for` loop",
        "## `struct` refinement",
        "## `enum` refinement",
    ] {
        assert!(examples.contains(heading), "examples.md omits {heading}");
    }
    Ok(())
}
