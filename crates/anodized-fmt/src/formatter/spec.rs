use std::collections::HashMap;

use anodized_core::annotate::syntax::{SpecArg, SpecArgValue, SpecArgs};
use syn::spanned::Spanned;

use crate::{collect::ParentIndent, config::Config};

use super::Formatter;

fn arg_end_line(arg: &SpecArg) -> usize {
    match &arg.value {
        SpecArgValue::None => arg.keyword_span.end().line.saturating_sub(1),
        SpecArgValue::Expr(expr) => expr.span().end().line.saturating_sub(1),
        SpecArgValue::Pat(pat) => pat.span().end().line.saturating_sub(1),
    }
}

/// Format a complete #[spec(...)] attribute with comment preservation.
///
/// This is the main entry point for formatting a spec attribute. It:
/// 1. Creates a formatter with the comment map and base indentation
/// 2. Formats the spec args
/// 3. Returns the complete #[spec(...)] string
pub fn format_spec_attribute(
    spec_args: &SpecArgs,
    config: &Config,
    base_indent: &ParentIndent,
    comments: HashMap<usize, Option<String>>,
) -> String {
    let indent_spaces = base_indent.total_spaces(config.tab_spaces);
    let mut formatter = Formatter::new(config, indent_spaces, comments);
    formatter.spec_args(spec_args);
    formatter.finish()
}

impl Formatter<'_> {
    /// Format SpecArgs into the output.
    pub fn spec_args(&mut self, spec_args: &SpecArgs) {
        let base_indent = self.base_indent;
        self.write("#[spec(");

        if spec_args.args.is_empty() {
            self.write(")]");
            return;
        }

        // Use vertical layout
        self.newline();
        let arg_indent = base_indent + self.settings.tab_spaces;
        self.set_indent(arg_indent);

        // Collect args with their original line numbers for comment association
        let args_with_lines: Vec<(&SpecArg, usize)> = spec_args
            .args
            .iter()
            .map(|arg| {
                let line = arg.keyword_span.start().line.saturating_sub(1);
                (arg, line)
            })
            .collect();

        // Associate comments with their corresponding args before sorting
        // For each arg, find comments that appear between the previous arg's end and this arg's keyword
        type ArgWithComments<'a> = (&'a SpecArg, usize, Vec<(usize, Option<String>)>);
        let args_with_comments: Vec<ArgWithComments> = if self.settings.reorder_spec_items {
            args_with_lines
                .iter()
                .enumerate()
                .map(|(idx, (arg, line))| {
                    let start_line = if idx == 0 {
                        0
                    } else {
                        arg_end_line(args_with_lines[idx - 1].0) + 1
                    };
                    let end_line = *line;

                    let mut comments = Vec::new();
                    for l in start_line..end_line {
                        if let Some(comment) = self.whitespace_and_comments.get(&l) {
                            comments.push((l, comment.clone()));
                        }
                    }

                    (*arg, *line, comments)
                })
                .collect()
        } else {
            // No reordering, so no need to pre-collect comments
            args_with_lines
                .into_iter()
                .map(|(arg, line)| (arg, line, Vec::new()))
                .collect()
        };

        // Sort if reordering is enabled (comments are now bundled with args)
        let mut final_args = args_with_comments;
        if self.settings.reorder_spec_items {
            final_args.sort_by_key(|(arg, _line, _comments)| &arg.keyword);
        }

        // Format each arg with its associated comments
        for (arg, original_line, comments) in final_args {
            if self.settings.reorder_spec_items {
                // Write the pre-collected comments for this arg
                for (_line, comment_opt) in comments {
                    if let Some(comment) = comment_opt {
                        self.write_indent();
                        self.write("// ");
                        self.write(&comment);
                        self.newline();
                    }
                }
            } else {
                // Flush comments in the original order
                self.flush_comments(original_line, false);
            }

            self.write_indent();
            self.format_spec_arg(arg);
            self.newline();
        }

        // Return to base indentation for closing bracket
        self.set_indent(base_indent);
        self.write_indent();
        self.write(")]");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::parse_str;

    #[test]
    fn test_format_simple_spec() {
        let spec_args: SpecArgs = parse_str("requires: x > 0").unwrap();
        let config = Config::default();
        let comments = HashMap::new();
        let indent = ParentIndent::default();

        let formatted = format_spec_attribute(&spec_args, &config, &indent, comments);

        assert_eq!(formatted, "#[spec(\n    requires: x > 0,\n)]");
    }

    #[test]
    fn test_format_empty_spec() {
        let spec_args: SpecArgs = parse_str("").unwrap();
        let config = Config::default();
        let comments = HashMap::new();
        let indent = ParentIndent::default();

        let formatted = format_spec_attribute(&spec_args, &config, &indent, comments);

        assert_eq!(formatted, "#[spec()]");
    }
}
