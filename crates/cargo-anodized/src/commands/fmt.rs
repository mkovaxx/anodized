use anodized_fmt::{Config, check_file, format_file};
use std::{
    collections::BTreeSet as Set,
    fs,
    io::Write,
    path::PathBuf,
    process::{Command, ExitStatus},
};

use crate::cli::FmtOptions;

type Result<T> = std::result::Result<T, Error>;

pub fn fmt(options: FmtOptions) -> Result<()> {
    let FmtOptions {
        packages,
        manifest_path,
        all,
        verbose,
        check,
    } = options;

    let mut command = Command::new("cargo");
    command.arg("fmt");

    for package in &packages {
        command.args(["--package", package]);
    }
    if let Some(manifest_path) = &manifest_path {
        command.args(["--manifest-path"]);
        command.arg(manifest_path);
    }
    if all {
        command.arg("--all");
    }
    if check {
        command.arg("--check");
    }
    command.args(["--", "--verbose"]);

    if verbose {
        let command_line = std::iter::once(command.get_program())
            .chain(command.get_args())
            .map(|arg| arg.to_string_lossy())
            .collect::<Vec<_>>()
            .join(" ");
        println!("Running `{command_line}`");
    }
    let output = command.output()?;

    let paths = formatted_paths(&output.stdout)?;
    if verbose {
        std::io::stdout().write_all(&output.stdout)?;
    }

    if !output.status.success() {
        return Err(Error::CargoFmtFailed(output.status));
    }

    if verbose {
        println!("Running `anodized-fmt`");
    }

    let config = Config::load()?;
    if check {
        let mut all_formatted = true;
        for path in paths {
            if verbose {
                println!("check {}", path.display());
            }
            let source = fs::read_to_string(&path)?;
            let is_formatted = check_file(&source, &config)?;
            if verbose && !is_formatted {
                println!("needs formatting");
            }
            all_formatted &= is_formatted;
        }

        if !all_formatted {
            return Err(Error::CheckFailed);
        }
    } else {
        for path in paths {
            if verbose {
                println!("format {}", path.display());
            }
            let source = fs::read_to_string(&path)?;
            let formatted = format_file(&source, &config)?;

            if formatted != source {
                if verbose {
                    println!("reformatted");
                }
                fs::write(path, formatted)?;
            }
        }
    }

    Ok(())
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error("`cargo fmt` output was not valid UTF-8: {0}")]
    Utf8(#[from] std::str::Utf8Error),

    #[error("`cargo fmt` failed with status {0}")]
    CargoFmtFailed(ExitStatus),

    #[error("some files need formatting with `anodized-fmt`")]
    CheckFailed,

    #[error(transparent)]
    Config(#[from] anodized_fmt::ConfigError),

    #[error(transparent)]
    Format(#[from] anodized_fmt::FormatError),
}

fn formatted_paths(output: &[u8]) -> Result<Set<PathBuf>> {
    let output = std::str::from_utf8(output)?;
    output
        .lines()
        .filter_map(|line| line.strip_prefix("Formatting "))
        .map(|path_str| PathBuf::from(path_str).canonicalize().map_err(Error::from))
        .collect()
}
