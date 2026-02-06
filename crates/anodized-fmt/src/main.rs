use anodized_fmt::{Config, Result, check_file, format_file};
use clap::Parser;
use std::fs;
use std::path::PathBuf;
use walkdir::WalkDir;

#[derive(Parser)]
#[command(
    name = "anodized-fmt",
    version,
    about = "Format #[spec] annotations in Rust code",
    long_about = "A formatter for Anodized #[spec] attributes. Reformats specifications while leaving all other code unchanged."
)]
struct Cli {
    /// Files or directories to format (default: current directory)
    #[arg(value_name = "PATH")]
    paths: Vec<PathBuf>,

    /// Check if files are formatted without modifying them
    #[arg(short, long)]
    check: bool,

    /// Path to configuration file
    #[arg(long, value_name = "FILE")]
    config: Option<PathBuf>,

    /// Print default configuration
    #[arg(long, value_name = "OPTION")]
    print_config: Option<PrintConfig>,

    /// Verbose output
    #[arg(short, long)]
    verbose: bool,

    /// Suppress non-error output
    #[arg(short, long)]
    quiet: bool,
}

#[derive(Clone, Copy, clap::ValueEnum)]
enum PrintConfig {
    Default,
    Current,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    // Handle print-config
    if let Some(print_config_option) = cli.print_config {
        match print_config_option {
            PrintConfig::Default => {
                println!("{}", Config::default_toml());
                return Ok(());
            }
            PrintConfig::Current => {
                let config = if let Some(config_path) = &cli.config {
                    Config::from_file(config_path)?
                } else {
                    Config::load()?
                };
                println!("{}", toml::to_string_pretty(&config)?);
                return Ok(());
            }
        }
    }

    // Load configuration
    let config = if let Some(config_path) = &cli.config {
        if cli.verbose {
            eprintln!("Loading config from: {}", config_path.display());
        }
        Config::from_file(config_path)?
    } else {
        match Config::load() {
            Ok(config) => {
                if cli.verbose {
                    eprintln!("Using loaded configuration");
                }
                config
            }
            Err(_) => {
                if cli.verbose {
                    eprintln!("Using default configuration");
                }
                Config::default()
            }
        }
    };

    // Determine paths to process
    let paths = if cli.paths.is_empty() {
        vec![PathBuf::from(".")]
    } else {
        cli.paths.clone()
    };

    // Collect all Rust files
    let mut rust_files = Vec::new();
    for path in paths {
        if path.is_file() {
            if path.extension().map_or(false, |ext| ext == "rs") {
                rust_files.push(path);
            }
        } else if path.is_dir() {
            for entry in WalkDir::new(&path)
                .into_iter()
                .filter_map(|e| e.ok())
                .filter(|e| e.path().extension().map_or(false, |ext| ext == "rs"))
            {
                rust_files.push(entry.path().to_path_buf());
            }
        }
    }

    if rust_files.is_empty() {
        if !cli.quiet {
            eprintln!("No Rust files found to format");
        }
        return Ok(());
    }

    // Process files
    let mut files_formatted = 0;
    let mut files_checked = 0;
    let mut errors = Vec::new();

    for file_path in &rust_files {
        if cli.verbose {
            eprintln!("Processing: {}", file_path.display());
        }

        match process_file(file_path, &config, cli.check) {
            Ok(changed) => {
                if cli.check {
                    files_checked += 1;
                    if changed {
                        if !cli.quiet {
                            println!("{}: not formatted", file_path.display());
                        }
                    } else if cli.verbose {
                        println!("{}: formatted correctly", file_path.display());
                    }
                } else {
                    if changed {
                        files_formatted += 1;
                        if !cli.quiet {
                            println!("{}: formatted", file_path.display());
                        }
                    } else if cli.verbose {
                        println!("{}: no changes needed", file_path.display());
                    }
                }
            }
            Err(e) => {
                errors.push((file_path.clone(), e));
            }
        }
    }

    // Report results
    if !cli.quiet {
        if cli.check {
            let unformatted = files_checked - files_formatted;
            if unformatted > 0 {
                eprintln!("\n{} file(s) not formatted correctly", unformatted);
                std::process::exit(1);
            } else {
                eprintln!("\nAll {} file(s) formatted correctly", files_checked);
            }
        } else {
            eprintln!("\nFormatted {} file(s)", files_formatted);
        }
    }

    // Report errors
    if !errors.is_empty() {
        eprintln!("\nErrors encountered:");
        for (path, error) in errors {
            eprintln!("  {}: {}", path.display(), error);
        }
        std::process::exit(1);
    }

    Ok(())
}

/// Process a single file
/// Returns Ok(true) if the file was changed, Ok(false) if no changes were needed
fn process_file(path: &PathBuf, config: &Config, check_only: bool) -> Result<bool> {
    let source = fs::read_to_string(path)?;

    if check_only {
        let is_formatted = check_file(&source, config)?;
        Ok(!is_formatted)
    } else {
        let formatted = format_file(&source, config)?;
        let changed = formatted != source;

        if changed {
            fs::write(path, formatted)?;
        }

        Ok(changed)
    }
}
