use std::collections::HashMap;

use anodized_core::syntax::{Keyword, SpecFields};
use syn::FieldValue;
use syn::spanned::Spanned;

use crate::{collect::ParentIndent, config::Config};

use super::Formatter;

fn field_end_line(field: &FieldValue) -> usize {
    field.expr.span().end().line.saturating_sub(1)
}

/// Format a complete #[spec(...)] attribute with comment preservation.
///
/// This is the main entry point for formatting a spec attribute. It:
/// 1. Creates a formatter with the comment map and base indentation
/// 2. Formats the spec fields
/// 3. Returns the complete #[spec(...)] string
pub fn format_spec_attribute(
    spec_fields: &SpecFields,
    config: &Config,
    base_indent: &ParentIndent,
    comments: HashMap<usize, Option<String>>,
) -> String {
    let indent_spaces = base_indent.total_spaces(config.tab_spaces);
    let mut formatter = Formatter::new(config, indent_spaces, comments);
    formatter.spec_fields(spec_fields);
    formatter.finish()
}

impl Formatter<'_> {
    /// Format SpecFields into the output.
    pub fn spec_fields(&mut self, spec_fields: &SpecFields) {
        let base_indent = self.base_indent;
        self.write("#[spec(");

        if spec_fields.fields.is_empty() {
            self.write(")]");
            return;
        }

        // Use vertical layout
        self.newline();
        let field_indent = base_indent + self.settings.tab_spaces;
        self.set_indent(field_indent);

        // Collect fields with their original line numbers for comment association
        let fields_with_lines: Vec<(&FieldValue, usize)> = spec_fields
            .fields
            .iter()
            .map(|field| {
                let line = field.member.span().start().line.saturating_sub(1);
                (field, line)
            })
            .collect();

        // Associate comments with their corresponding fields before sorting.
        type FieldWithComments<'a> = (&'a FieldValue, usize, Vec<(usize, Option<String>)>);
        let fields_with_comments: Vec<FieldWithComments> = if self.settings.reorder_spec_items {
            fields_with_lines
                .iter()
                .enumerate()
                .map(|(idx, (field, line))| {
                    let start_line = if idx == 0 {
                        0
                    } else {
                        field_end_line(fields_with_lines[idx - 1].0) + 1
                    };
                    let end_line = *line;

                    let mut comments = Vec::new();
                    for l in start_line..end_line {
                        if let Some(comment) = self.whitespace_and_comments.get(&l) {
                            comments.push((l, comment.clone()));
                        }
                    }

                    (*field, *line, comments)
                })
                .collect()
        } else {
            // No reordering, so no need to pre-collect comments
            fields_with_lines
                .into_iter()
                .map(|(field, line)| (field, line, Vec::new()))
                .collect()
        };

        // Sort if reordering is enabled (comments are now bundled with fields).
        let mut final_fields = fields_with_comments;
        if self.settings.reorder_spec_items {
            final_fields.sort_by_key(|(field, _line, _comments)| Keyword::from(&field.member));
        }

        // Format each field with its associated comments.
        for (field, original_line, comments) in final_fields {
            if self.settings.reorder_spec_items {
                // Write the pre-collected comments for this field.
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
            self.format_spec_field(field);
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
        let spec_fields: SpecFields = parse_str("requires: x > 0").unwrap();
        let config = Config::default();
        let comments = HashMap::new();
        let indent = ParentIndent::default();

        let formatted = format_spec_attribute(&spec_fields, &config, &indent, comments);

        assert_eq!(formatted, "#[spec(\n    requires: x > 0,\n)]");
    }

    #[test]
    fn test_format_empty_spec() {
        let spec_fields: SpecFields = parse_str("").unwrap();
        let config = Config::default();
        let comments = HashMap::new();
        let indent = ParentIndent::default();

        let formatted = format_spec_attribute(&spec_fields, &config, &indent, comments);

        assert_eq!(formatted, "#[spec()]");
    }
}
