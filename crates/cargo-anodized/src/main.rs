use clap::Parser;

use crate::cli::{Cli, Command};

mod cli;
mod commands;

fn main() {
    if let Err(error) = run() {
        eprintln!("Error: {error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), Error> {
    let cli = Cli::parse();

    match cli.command {
        Command::Fmt(options) => commands::fmt::fmt(options)?,
        #[cfg(feature = "z3")]
        Command::Z3Smt2In(options) => commands::z3::emulate_smt2_in(options)?,
    }

    Ok(())
}

#[derive(Debug, thiserror::Error)]
enum Error {
    #[error(transparent)]
    Fmt(#[from] commands::fmt::Error),

    #[cfg(feature = "z3")]
    #[error(transparent)]
    Z3Smt2In(#[from] commands::z3::Error),
}
