//! Anodized-fmt: A formatter for #[spec] annotations
//!
//! This library provides functionality to format #[spec] attributes in Rust code
//! while leaving all other code unchanged.

pub mod config;

// Comment-preserving formatting modules
mod collect;
mod collect_comments;
mod formatter;
pub mod source_file;

pub use config::{Config, ConfigError};
pub use source_file::{check_file, format_file};

#[derive(Debug, thiserror::Error)]
pub enum FormatError {
    #[error("Failed to parse spec attribute: {0}")]
    SpecParseError(#[from] syn::Error),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Configuration error: {0}")]
    ConfigError(#[from] ConfigError),

    #[error("TOML serialization error: {0}")]
    TomlSerError(#[from] toml::ser::Error),
}

pub type Result<T> = std::result::Result<T, FormatError>;
