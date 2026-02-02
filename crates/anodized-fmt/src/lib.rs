//! Anodized-fmt: A formatter for #[spec] annotations
//!
//! This library provides functionality to format #[spec] attributes in Rust code
//! while leaving all other code unchanged.

pub mod config;
pub mod expr_fmt;
pub mod finder;
pub mod formatter;

pub use config::{Config, ConfigError};
pub use finder::{FindError, SpecLocation, find_spec_attributes};
pub use formatter::format_spec;

use anodized_core::Spec;
use syn::parse_str;

#[derive(Debug, thiserror::Error)]
pub enum FormatError {
    #[error("Failed to parse spec attribute: {0}")]
    SpecParseError(#[from] syn::Error),

    #[error("Unmatched brackets in spec attribute at position {0}")]
    UnmatchedBracket(usize),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Configuration error: {0}")]
    ConfigError(#[from] ConfigError),

    #[error("File contains invalid UTF-8")]
    InvalidUtf8,

    #[error("Failed to format expression: {0}")]
    ExpressionFormatError(String),

    #[error("Find error: {0}")]
    FindError(#[from] FindError),

    #[error("TOML serialization error: {0}")]
    TomlSerError(#[from] toml::ser::Error),
}

pub type Result<T> = std::result::Result<T, FormatError>;

/// Format a single file's #[spec] attributes
///
/// This function:
/// 1. Finds all #[spec] attributes in the source
/// 2. Parses each one using anodized-core
/// 3. Reformats them according to the configuration
/// 4. Returns the modified source code
pub fn format_file(source: &str, config: &Config) -> Result<String> {
    // Find all spec attributes
    let locations = find_spec_attributes(source)?;

    if locations.is_empty() {
        // No specs to format, return unchanged
        return Ok(source.to_string());
    }

    // Build the output by replacing each spec attribute
    let mut output = String::new();
    let mut last_end = 0;

    for location in &locations {
        // Add everything before this spec
        output.push_str(&source[last_end..location.start]);

        // Parse and format this spec
        let spec: Spec = parse_str(&location.content)?;
        let formatted = format_spec(&spec, config);
        output.push_str(&formatted);

        last_end = location.end;
    }

    // Add remaining content after last spec
    output.push_str(&source[last_end..]);

    Ok(output)
}

/// Check if a file's #[spec] attributes are formatted correctly
///
/// Returns true if the file is already formatted according to the config
pub fn check_file(source: &str, config: &Config) -> Result<bool> {
    let formatted = format_file(source, config)?;
    Ok(formatted == source)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_file_with_simple_spec() {
        let source = r#"
                #[spec(requires: x > 0)]
                fn foo(x: i32) -> i32 {
                    x + 1
                }
                "#;
        let config = Config::default();
        let result = format_file(source, &config);

        assert!(result.is_ok());
        let formatted = result.unwrap();
        assert!(formatted.contains("#[spec("));
        assert!(formatted.contains("requires:"));
    }

    #[test]
    fn test_format_file_without_specs() {
        let source = r#"
                fn foo(x: i32) -> i32 {
                    x + 1
                }
                "#;
        let config = Config::default();
        let result = format_file(source, &config);

        assert!(result.is_ok());
        let formatted = result.unwrap();
        assert_eq!(formatted, source);
    }

    #[test]
    fn test_check_file_formatted() {
        let source = r#"
                #[spec(
                    requires: x > 0,
                )]
                fn foo(x: i32) -> i32 {
                    x + 1
                }
                "#;
        let config = Config::default();

        // First format it
        let formatted = format_file(source, &config).unwrap();

        // Check should return true for the formatted version
        let is_formatted = check_file(&formatted, &config).unwrap();
        assert!(is_formatted);
    }
}
