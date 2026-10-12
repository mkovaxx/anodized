use std::path::PathBuf;

use clap::{Args, Parser, Subcommand};

#[derive(Parser)]
#[command(version, about)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    /// Format a Cargo project.
    Fmt(FmtOptions),
}

#[derive(Args)]
pub struct FmtOptions {
    /// Specify packages to format.
    #[arg(short, long = "package", value_name = "PACKAGE")]
    pub packages: Vec<String>,

    /// Specify the path to Cargo.toml.
    #[arg(long, value_name = "PATH")]
    pub manifest_path: Option<PathBuf>,

    /// Format all packages in the workspace.
    #[arg(long)]
    pub all: bool,

    /// Use verbose output.
    #[arg(short, long)]
    pub verbose: bool,

    /// Check formatting without modifying files.
    #[arg(long)]
    pub check: bool,
}
