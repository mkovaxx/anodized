use std::ops::Range;

use crop::Rope;
use quote::ToTokens;
use syn::{parse_file, spanned::Spanned};

use crate::{
    FormatError,
    collect::{SpecAttr, collect_spec_attrs_in_file},
    collect_comments::{
        extract_whitespace_and_comments, has_nested_structure_comments, line_column_to_byte,
    },
    config::Config,
    formatter::format_spec_attribute,
};

use anodized_core::annotate::syntax::SpecArgs;

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
pub fn format_file(source: &str, config: &Config) -> Result<String, FormatError> {
    let ast = parse_file(source)?;
    let rope = Rope::from(source);
    let spec_attrs = collect_spec_attrs_in_file(&ast, &rope);
    format_source(rope, spec_attrs, config)
}

fn format_source(
    mut source: Rope,
    spec_attrs: Vec<SpecAttr<'_>>,
    config: &Config,
) -> Result<String, FormatError> {
    let mut edits = Vec::new();

    for spec_attr in spec_attrs {
        let attr = spec_attr.attr;
        let span = attr.span();
        let start = span.start();
        let end = span.end();

        // Extract the full token stream for comment extraction
        // This includes the `spec` identifier and the parenthesized content
        let tokens = attr.meta.to_token_stream();

        // For nested structure detection, use only the inner tokens (inside spec(...))
        let inner_tokens = if let syn::Meta::List(meta_list) = &attr.meta {
            meta_list.tokens.clone()
        } else {
            // If it's not a list format, skip this attribute
            continue;
        };

        // For inline comments, we need to check the original source text
        // This catches inline comments anywhere in the attribute, including in nested structures
        let start_byte = line_column_to_byte(&source, start);
        let end_byte = line_column_to_byte(&source, end);
        let attr_source = source.byte_slice(start_byte..end_byte).to_string();

        // Check if this spec has inline comments or nested structure comments
        // If it does, we skip formatting to avoid losing comment positions
        if has_inline_comments(&attr_source) || has_nested_structure_comments(&source, inner_tokens)
        {
            continue;
        }

        // Extract comments for this attribute
        let comments = extract_whitespace_and_comments(&source, tokens);

        // Parse the spec arguments
        let spec_args = match attr.parse_args::<SpecArgs>() {
            Ok(args) => args,
            Err(_) => continue, // Skip malformed specs
        };

        // Format with comments
        let formatted = format_spec_attribute(&spec_args, config, &spec_attr.base_indent, comments);

        let start_byte = line_column_to_byte(&source, start);
        let end_byte = line_column_to_byte(&source, end);

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

        source.replace(
            (start as isize + last_offset) as usize..(end as isize + last_offset) as usize,
            &new_text,
        );
        last_offset += new_text.len() as isize - (end as isize - start as isize);
    }

    Ok(source.to_string())
}

/// Check if the source text contains inline comments.
fn has_inline_comments(source_text: &str) -> bool {
    source_text.lines().any(|line| {
        // Check if line has code followed by //
        if let Some((before, _after)) = line.split_once("//") {
            // Inline comment if there's non-whitespace before the //
            // and it doesn't start with // (which would be a regular comment)
            !before.trim().is_empty() && !before.trim().starts_with("//")
        } else {
            false
        }
    })
}

/// Check if a file's #[spec] attributes are formatted correctly
///
/// Returns true if the file is already formatted according to the config
pub fn check_file(source: &str, config: &Config) -> Result<bool, FormatError> {
    let formatted = format_file(source, config)?;
    Ok(formatted == source)
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
        let result = format_file(source, &config);

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
        let result = format_file(source, &config);

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
        let result = format_file(source, &config);

        assert!(result.is_ok());
        let formatted = result.unwrap();

        // Should format both specs
        assert!(formatted.contains("requires: x > 0"));
        assert!(formatted.contains("requires: y > 0"));
    }

    #[test]
    fn test_has_inline_comments_in_nested_structures() {
        // Test that inline comments are detected in nested structures
        let source_with_inline_in_array = r#"captures: [x as val, // inline comment
        y as other]"#;
        assert!(has_inline_comments(source_with_inline_in_array));

        // Test that inline comments are detected at top level
        let source_with_inline_top = "requires: x > 0, // inline comment";
        assert!(has_inline_comments(source_with_inline_top));

        // Test that regular comments (not inline) are not flagged
        let source_with_regular_comment = r#"
        // This is a regular comment
        requires: x > 0,
        // Another regular comment
        ensures: result > 0"#;
        assert!(!has_inline_comments(source_with_regular_comment));

        // Test that no comments means no inline comments
        let source_no_comments = "requires: x > 0, ensures: result > 0";
        assert!(!has_inline_comments(source_no_comments));
    }
}
