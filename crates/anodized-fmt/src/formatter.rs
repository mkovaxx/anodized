use std::collections::HashMap;

use anodized_core::annotate::syntax::{SpecArg, SpecArgValue};
use syn::Meta;
use syn::spanned::Spanned;

use crate::config::{Config, TrailingComma};

mod expr;
mod spec;

use expr::{format_expr, format_pattern};

pub use spec::format_spec_attribute;

/// The main formatter struct that tracks state during formatting.
pub struct Formatter<'a> {
    /// The output string being built
    output: String,
    /// Configuration settings
    settings: &'a Config,
    /// Map of line numbers to comments/empty lines
    whitespace_and_comments: HashMap<usize, Option<String>>,
    /// Current line offset (tracks position in source for comment flushing)
    line_offset: Option<usize>,
    /// Current indentation level (in spaces)
    base_indent: usize,
}

impl<'a> Formatter<'a> {
    /// Create a formatter with comment map and base indentation (for comment-preserving formatting).
    pub fn new(
        settings: &'a Config,
        base_indent: usize,
        whitespace_and_comments: HashMap<usize, Option<String>>,
    ) -> Self {
        Self {
            output: String::new(),
            settings,
            whitespace_and_comments,
            line_offset: None,
            base_indent,
        }
    }

    /// Set the current indentation level.
    pub fn set_indent(&mut self, spaces: usize) {
        self.base_indent = spaces;
    }

    /// Write a string to the output.
    pub fn write(&mut self, s: &str) {
        self.output.push_str(s);
    }

    /// Write a newline to the output.
    pub fn newline(&mut self) {
        self.output.push('\n');
    }

    /// Write indentation spaces.
    pub fn write_indent(&mut self) {
        self.output.push_str(&" ".repeat(self.base_indent));
    }

    /// Flush any comments that appear before the given line number.
    ///
    /// This removes comments from the map and writes them to the output,
    /// preserving their position relative to the formatted code.
    pub fn flush_comments(&mut self, line_index: usize, skip_trailing_whitespace: bool) {
        let last = self.line_offset.unwrap_or(0);

        // Collect all comments/empty lines from last position to current line
        let comments_or_empty_lines: Vec<_> = (last..=line_index)
            .filter_map(|l| self.whitespace_and_comments.remove(&l))
            .collect();

        // Calculate how many to take (optionally skip trailing whitespace)
        let take_n = if skip_trailing_whitespace {
            comments_or_empty_lines
                .iter()
                .rev()
                .position(Option::is_some)
                .map(|i| comments_or_empty_lines.len() - i)
                .unwrap_or(comments_or_empty_lines.len())
        } else {
            comments_or_empty_lines.len()
        };

        let mut prev_is_empty_line = false;

        for comment_or_empty in comments_or_empty_lines.into_iter().take(take_n) {
            if let Some(comment) = comment_or_empty {
                self.write_indent();
                self.write("// ");
                self.write(&comment);
                self.newline();
                prev_is_empty_line = false;
            } else if self.line_offset.is_some() {
                // Don't print multiple consecutive empty lines
                if !prev_is_empty_line {
                    self.newline();
                }
                prev_is_empty_line = true;
            }
        }

        self.line_offset = Some(line_index);
    }

    /// Get the formatted output.
    pub fn finish(self) -> String {
        self.output
    }

    /// Format a SpecArg into the output.
    pub fn format_spec_arg(&mut self, arg: &SpecArg) {
        // Add cfg attribute if present
        if let Some(cfg_attr) = Self::find_cfg_attribute(&arg.attrs)
            && let Ok(meta) = cfg_attr.parse_args::<Meta>()
        {
            self.write(&Self::format_cfg_attr(&meta));
            self.newline();
            self.write_indent();
        }

        // Format the value based on what it contains
        let value_str = match &arg.value {
            SpecArgValue::None => String::new(),
            SpecArgValue::Expr(expr) => {
                if let syn::Expr::Array(array) = expr {
                    let elem_strs = Vec::from_iter(array.elems.iter().map(format_expr));
                    let elem_lines: Vec<usize> = array
                        .elems
                        .iter()
                        .map(|e| e.span().start().line.saturating_sub(1))
                        .collect();
                    let bracket_line = array
                        .bracket_token
                        .span
                        .open()
                        .start()
                        .line
                        .saturating_sub(1);
                    self.format_array(&elem_strs, Some(&elem_lines), Some(bracket_line))
                } else {
                    format_expr(expr)
                }
            }
            SpecArgValue::Pat(pat) => format_pattern(pat),
        };

        if value_str.is_empty() {
            self.write(&format!("{},", arg.keyword));
        } else {
            self.write(&format!("{}: {},", arg.keyword, value_str));
        }
    }

    /// Format an array expression with proper indentation.
    ///
    /// When `elem_lines` is provided, comments from `self.whitespace_and_comments`
    /// are flushed before each element at the appropriate line positions.
    /// `bracket_line` is the 0-indexed line of the opening `[`, used to determine
    /// the comment range for the first element.
    fn format_array(
        &mut self,
        elems: &[String],
        elem_lines: Option<&[usize]>,
        bracket_line: Option<usize>,
    ) -> String {
        if elems.is_empty() {
            return "[]".to_string();
        }

        if elems.len() == 1 {
            return elems[0].clone();
        }

        let mut result = String::from("[\n");
        let elem_indent = " ".repeat(self.base_indent + self.settings.tab_spaces);

        let add_trailing_comma = match self.settings.trailing_comma {
            TrailingComma::Always => true,
            TrailingComma::Never => false,
            TrailingComma::Vertical => elems.len() > 1,
        };

        for (i, elem) in elems.iter().enumerate() {
            if let Some(lines) = elem_lines {
                let target_line = lines[i];
                let start_line = if i == 0 {
                    bracket_line.map_or(target_line, |bl| bl + 1)
                } else {
                    lines[i - 1] + 1
                };

                for line_idx in start_line..target_line {
                    if let Some(comment_opt) = self.whitespace_and_comments.remove(&line_idx)
                        && let Some(comment) = comment_opt
                    {
                        result.push_str(&elem_indent);
                        result.push_str("// ");
                        result.push_str(&comment);
                        result.push('\n');
                    }
                }
            }

            result.push_str(&elem_indent);
            result.push_str(elem);

            if i < elems.len() - 1 || add_trailing_comma {
                result.push(',');
            }
            result.push('\n');
        }

        if let Some(lines) = elem_lines
            && let Some(&last) = lines.last()
        {
            self.line_offset = Some(last);
        }

        result.push_str(&" ".repeat(self.base_indent));
        result.push(']');

        result
    }

    /// Find a cfg attribute in the attribute list.
    fn find_cfg_attribute(attrs: &[syn::Attribute]) -> Option<&syn::Attribute> {
        attrs.iter().find(|attr| attr.path().is_ident("cfg"))
    }

    /// Format a cfg attribute.
    fn format_cfg_attr(meta: &Meta) -> String {
        let tokens = quote::quote!(#meta);
        format!("#[cfg({})]", tokens)
    }
}
