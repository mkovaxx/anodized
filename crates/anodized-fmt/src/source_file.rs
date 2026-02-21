use std::ops::Range;

use crop::Rope;
use proc_macro2::LineColumn;
use quote::ToTokens;
use syn::{parse_file, spanned::Spanned};
use thiserror::Error;

use crate::{
    collect::{collect_spec_attrs_in_file, SpecAttr},
    collect_comments::extract_whitespace_and_comments,
    config::Config,
    formatter_new::format_spec_attribute,
};

#[derive(Error, Debug)]
pub enum FormatError {
    #[error("Could not parse file: {0}")]
    ParseError(#[from] syn::Error),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

#[derive(Debug)]
struct TextEdit {
    range: Range<usize>,
    new_text: String,
}

/// Format all #[spec] attributes in a Rust source file.
///
/// This is the main entry point for the formatter. It:
/// 1. Parses the source file with syn
/// 2. Collects all #[spec] attributes using a visitor
/// 3. For each attribute:
///    - Extracts comments from the token stream
///    - Formats with comment preservation
///    - Tracks the text replacement
/// 4. Applies all replacements to produce formatted output
pub fn format_file_source(source: &str, config: &Config) -> Result<String, FormatError> {
    let ast = parse_file(source)?;
    let rope = Rope::from(source);
    let spec_attrs = collect_spec_attrs_in_file(&ast, &rope);
    format_source(rope, spec_attrs, config)
}

fn format_source(
    mut rope: Rope,
    spec_attrs: Vec<SpecAttr<'_>>,
    config: &Config,
) -> Result<String, FormatError> {
    let mut edits = Vec::new();

    for spec_attr in spec_attrs {
        let attr = spec_attr.attr;
        let span = attr.span();
        let start = span.start();
        let end = span.end();

        // Extract comments from this attribute's token stream
        let tokens = attr.meta.to_token_stream();
        let comments = extract_whitespace_and_comments(&rope, tokens);

        // Parse the spec arguments
        let spec_args = match attr.parse_args::<anodized_core::annotate::syntax::SpecArgs>() {
            Ok(args) => args,
            Err(_) => continue, // Skip malformed specs
        };

        // Format with comments
        let formatted = format_spec_attribute(
            &spec_args,
            config,
            &spec_attr.base_indent,
            &rope,
            comments,
        );

        let start_byte = line_column_to_byte(&rope, start);
        let end_byte = line_column_to_byte(&rope, end);

        edits.push(TextEdit {
            range: start_byte..end_byte,
            new_text: formatted,
        });
    }

    // Apply all edits with offset tracking
    let mut last_offset: isize = 0;
    for edit in edits {
        let start = edit.range.start;
        let end = edit.range.end;
        let new_text = edit.new_text;

        rope.replace(
            (start as isize + last_offset) as usize..(end as isize + last_offset) as usize,
            &new_text,
        );
        last_offset += new_text.len() as isize - (end as isize - start as isize);
    }

    Ok(rope.to_string())
}

/// Convert a LineColumn position to a byte offset in the Rope.
fn line_column_to_byte(source: &Rope, point: LineColumn) -> usize {
    let line_byte = source.byte_of_line(point.line - 1);
    let line = source.line(point.line - 1);
    let char_byte: usize = line.chars().take(point.column).map(|c| c.len_utf8()).sum();
    line_byte + char_byte
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_simple_spec() {
        let source = r#"
use anodized::spec;

#[spec(requires: x > 0)]
fn foo(x: i32) -> i32 {
    x + 1
}
"#;
        let config = Config::default();
        let result = format_file_source(source, &config);

        assert!(result.is_ok());
        let formatted = result.unwrap();
        // Should contain the formatted spec
        assert!(formatted.contains("#[spec("));
        assert!(formatted.contains("requires: x > 0"));
    }

    #[test]
    fn test_preserves_non_spec_code() {
        let source = r#"
use anodized::spec;

const VALUE: i32 = 42;

#[spec(requires: x > 0)]
fn foo(x: i32) -> i32 {
    x + VALUE
}

#[derive(Debug)]
struct MyStruct {
    field: i32,
}
"#;
        let config = Config::default();
        let result = format_file_source(source, &config);

        assert!(result.is_ok());
        let formatted = result.unwrap();

        // Non-spec code should be preserved
        assert!(formatted.contains("const VALUE: i32 = 42;"));
        assert!(formatted.contains("#[derive(Debug)]"));
        assert!(formatted.contains("struct MyStruct"));
    }

    #[test]
    fn test_multiple_specs() {
        let source = r#"
#[spec(requires: x > 0)]
fn foo(x: i32) -> i32 {
    x + 1
}

#[spec(requires: y > 0)]
fn bar(y: i32) -> i32 {
    y + 2
}
"#;
        let config = Config::default();
        let result = format_file_source(source, &config);

        assert!(result.is_ok());
        let formatted = result.unwrap();

        // Should format both specs
        assert!(formatted.contains("requires: x > 0"));
        assert!(formatted.contains("requires: y > 0"));
    }
}
