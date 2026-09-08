//! Rewrites, or verifies, the generated tables in the Anodized agent skill.

use std::process::ExitCode;

use skill_tools::generate;

fn main() -> ExitCode {
    let mode = std::env::args().nth(1);
    match mode.as_deref() {
        Some("--write") => run(write),
        Some("--check") | None => run(check),
        Some(other) => {
            eprintln!("unknown argument `{other}`; expected `--write` or `--check`");
            ExitCode::FAILURE
        }
    }
}

fn run(action: fn() -> skill_tools::Result<bool>) -> ExitCode {
    match action() {
        Ok(true) => ExitCode::SUCCESS,
        Ok(false) => ExitCode::FAILURE,
        Err(error) => {
            eprintln!("error: {error}");
            ExitCode::FAILURE
        }
    }
}

fn write() -> skill_tools::Result<bool> {
    let changed = generate::write()?;
    if changed.is_empty() {
        println!("generated regions are already up to date");
    } else {
        for file in changed {
            println!("updated {file}");
        }
    }
    Ok(true)
}

fn check() -> skill_tools::Result<bool> {
    let stale = generate::check()?;
    if stale.is_empty() {
        println!("generated regions are up to date");
        return Ok(true);
    }
    for region in stale {
        eprintln!("stale generated region: {region}");
    }
    eprintln!("run `cargo run -p skill-tools --bin skill-gen -- --write`");
    Ok(false)
}
