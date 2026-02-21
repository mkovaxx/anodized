use std::collections::HashMap;

use anodized_core::annotate::syntax::{SpecArg, SpecArgs};
use crop::Rope;

use crate::{collect::ParentIndent, config::Config};

use super::Formatter;

/// Format a complete #[spec(...)] attribute with comment preservation.
///
/// This is the main entry point for formatting a spec attribute. It:
/// 1. Creates a formatter with the comment map
/// 2. Formats the spec args with proper indentation
/// 3. Returns the complete #[spec(...)] string
pub fn format_spec_attribute(
    spec_args: &SpecArgs,
    config: &Config,
    base_indent: &ParentIndent,
    source: &Rope,
    comments: HashMap<usize, Option<String>>,
) -> String {
    let mut formatter = Formatter::with_source(config, source, comments);
    let indent_spaces = base_indent.total_spaces(config.tab_spaces);
    formatter.set_indent(indent_spaces);

    format_spec_args_internal(&mut formatter, spec_args, indent_spaces);
    formatter.finish()
}

/// Internal function to format SpecArgs with the formatter.
fn format_spec_args_internal(formatter: &mut Formatter, spec_args: &SpecArgs, base_indent: usize) {
    formatter.write("#[spec(");

    if spec_args.args.is_empty() {
        formatter.write(")]");
        return;
    }

    // Use vertical layout
    formatter.newline();
    let arg_indent = base_indent + formatter.settings.tab_spaces;
    formatter.set_indent(arg_indent);

    // Collect args with their original line numbers for comment association
    let mut args_with_lines: Vec<(&SpecArg, usize)> = spec_args
        .args
        .iter()
        .map(|arg| {
            let line = arg.keyword_span.start().line.saturating_sub(1);
            (arg, line)
        })
        .collect();

    // Sort if reordering is enabled
    if formatter.settings.reorder_spec_items {
        args_with_lines.sort_by_key(|(arg, _line)| formatter.keyword_order(&arg.keyword));
    }

    // Format each arg with its associated comments
    for (arg, original_line) in args_with_lines {
        // Flush comments that appeared before this arg in the original source
        // This makes comments "stick" to their arg when reordering
        formatter.flush_comments(original_line, false);

        formatter.write_indent();
        formatter.format_spec_arg(arg);
        formatter.newline();
    }

    // Return to base indentation for closing bracket
    formatter.set_indent(base_indent);
    formatter.write_indent();
    formatter.write(")]");
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::collect_comments::extract_whitespace_and_comments;
    use syn::parse_str;

    #[test]
    fn test_format_simple_spec() {
        let spec_args: SpecArgs = parse_str("requires: x > 0").unwrap();
        let config = Config::default();
        let source = Rope::from("requires: x > 0");
        let comments = HashMap::new();
        let indent = ParentIndent::default();

        let formatted = format_spec_attribute(&spec_args, &config, &indent, &source, comments);

        assert_eq!(formatted, "#[spec(\n    requires: x > 0,\n)]");
    }

    #[test]
    fn test_format_with_comment() {
        // Note: parse_str doesn't give us proper span line numbers,
        // so comments won't be properly associated. This is mainly
        // testing that the formatter doesn't crash with comments present.
        let source_text = r#"
            // This is a comment
            requires: x > 0,
            ensures: *output > 0
            "#;
        let spec_args: SpecArgs = parse_str(source_text).unwrap();
        let config = Config::default();
        let source = Rope::from(source_text);
        let tokens = source_text.parse().unwrap();
        let comments = extract_whitespace_and_comments(&source, tokens);
        let indent = ParentIndent::default();

        let formatted = format_spec_attribute(&spec_args, &config, &indent, &source, comments);

        // Should format the spec args (comment preservation is tested in integration tests)
        assert!(formatted.contains("requires: x > 0"));
    }

    #[test]
    fn test_format_empty_spec() {
        let spec_args: SpecArgs = parse_str("").unwrap();
        let config = Config::default();
        let source = Rope::from("");
        let comments = HashMap::new();
        let indent = ParentIndent::default();

        let formatted = format_spec_attribute(&spec_args, &config, &indent, &source, comments);

        assert_eq!(formatted, "#[spec()]");
    }
}
