//! A cargo project outside the workspace, used to compile snippets.

use std::path::{Path, PathBuf};
use std::process::Command;

use crate::{Result, repo_root};

/// The outcome of one build.
#[derive(Debug)]
pub struct Outcome {
    pub succeeded: bool,
    pub stderr: String,
}

/// A library crate that snippets are written into, one at a time.
///
/// It is a library, not a binary: the snippets are item-level and carry no `main`, so a binary
/// crate would fail with `E0601` before the spec was ever exercised. The manifest declares an
/// empty `[workspace]`, or the parent workspace claims it — `CARGO_TARGET_TMPDIR` lives under
/// the workspace target directory.
pub struct Project {
    dir: PathBuf,
    rustflags: Option<String>,
}

impl Project {
    /// Creates a scratch project with the default (unset) build configuration.
    ///
    /// `base` is the directory to build under; tests pass `CARGO_TARGET_TMPDIR`, which only
    /// exists for test targets.
    pub fn new(base: &Path, name: &str) -> Result<Self> {
        Self::with_rustflags(base, name, None)
    }

    /// Creates a scratch project built with the given `RUSTFLAGS`.
    ///
    /// Configurations get separate directories so flipping between them does not invalidate
    /// the other's build cache on every run.
    pub fn with_rustflags(base: &Path, name: &str, rustflags: Option<&str>) -> Result<Self> {
        let dir = base.join(name);
        std::fs::create_dir_all(dir.join("src"))?;
        // Both targets are declared, so both files must exist even when only one is in use.
        std::fs::write(dir.join("src/lib.rs"), "")?;
        std::fs::write(dir.join("src/main.rs"), "fn main() {}\n")?;

        let anodized = repo_root()?.join("crates/anodized");
        std::fs::write(
            dir.join("Cargo.toml"),
            format!(
                "[workspace]\nmembers = []\n\n\
                 [package]\nname = \"skill-scratch\"\nversion = \"0.0.0\"\nedition = \"2024\"\n\
                 publish = false\n\n\
                 [lib]\npath = \"src/lib.rs\"\n\n\
                 [[bin]]\nname = \"skill-scratch\"\npath = \"src/main.rs\"\n\n\
                 [dependencies]\nanodized = {{ path = {:?} }}\n",
                anodized
            ),
        )?;

        Ok(Self {
            dir,
            rustflags: rustflags.map(str::to_owned),
        })
    }

    /// Compiles one snippet, returning whether it built and what rustc said.
    pub fn build(&self, source: &str) -> Result<Outcome> {
        std::fs::write(self.dir.join("src/lib.rs"), source)?;
        std::fs::write(self.dir.join("src/main.rs"), "fn main() {}\n")?;
        let output = self.cargo("build").arg("--lib").output()?;
        Ok(Outcome {
            succeeded: output.status.success(),
            stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
        })
    }

    /// Builds and runs one snippet as a binary, returning whether it exited cleanly.
    ///
    /// Used for examples that exercise themselves, so a condition that is merely well-formed
    /// but false of the code fails here instead of in a reader's crate.
    pub fn run(&self, source: &str) -> Result<Outcome> {
        std::fs::write(self.dir.join("src/lib.rs"), "")?;
        std::fs::write(self.dir.join("src/main.rs"), source)?;
        let output = self
            .cargo("run")
            .arg("--bin")
            .arg("skill-scratch")
            .output()?;
        let mut stderr = String::from_utf8_lossy(&output.stderr).into_owned();
        stderr.push_str(&String::from_utf8_lossy(&output.stdout));
        Ok(Outcome {
            succeeded: output.status.success(),
            stderr,
        })
    }

    /// Runs the project's tests, returning whether they passed and what was reported.
    pub fn test(&self, source: &str) -> Result<Outcome> {
        std::fs::write(self.dir.join("src/lib.rs"), source)?;
        let output = self.cargo("test").output()?;
        let mut stderr = String::from_utf8_lossy(&output.stderr).into_owned();
        stderr.push_str(&String::from_utf8_lossy(&output.stdout));
        Ok(Outcome {
            succeeded: output.status.success(),
            stderr,
        })
    }

    fn cargo(&self, subcommand: &str) -> Command {
        let mut command = Command::new(env!("CARGO"));
        command.arg(subcommand).current_dir(&self.dir);
        // Warnings are errors: a snippet that warns in this project would warn in the
        // reader's crate too, and plenty of readers build with `-D warnings`.
        let mut flags = String::from("-D warnings");
        if let Some(extra) = &self.rustflags {
            flags.push(' ');
            flags.push_str(extra);
        }
        command.env("RUSTFLAGS", flags);
        command
    }
}
