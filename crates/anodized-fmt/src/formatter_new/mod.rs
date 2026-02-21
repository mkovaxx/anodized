use std::collections::HashMap;

use anodized_core::annotate::syntax::{CaptureExpr, Captures, Keyword, SpecArg, SpecArgValue};
use crop::Rope;
use syn::Meta;

use crate::{
    config::{Config, TrailingComma},
    expr_fmt::{format_expr, format_pattern},
};

mod spec;

pub use spec::format_spec_attribute;

/// The main formatter struct that tracks state during formatting.
pub struct Formatter<'a> {
    /// The output string being built
    output: String,
    /// Configuration settings
    settings: &'a Config,
    /// Optional source rope for accessing original text
    source: Option<&'a Rope>,
    /// Map of line numbers to comments/empty lines
    whitespace_and_comments: HashMap<usize, Option<String>>,
    /// Current line offset (tracks position in source for comment flushing)
    line_offset: Option<usize>,
    /// Current indentation level (in spaces)
    current_indent: usize,
}

impl<'a> Formatter<'a> {
    /// Create a formatter with source and comment map (for comment-preserving formatting).
    pub fn with_source(
        settings: &'a Config,
        source: &'a Rope,
        comments: HashMap<usize, Option<String>>,
    ) -> Self {
        Self {
            output: String::new(),
            settings,
            source: Some(source),
            whitespace_and_comments: comments,
            line_offset: None,
            current_indent: 0,
        }
    }

    /// Set the current indentation level.
    pub fn set_indent(&mut self, spaces: usize) {
        self.current_indent = spaces;
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
        self.output.push_str(&" ".repeat(self.current_indent));
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
        if let Some(cfg_attr) = Self::find_cfg_attribute(&arg.attrs) {
            if let Ok(meta) = cfg_attr.parse_args::<Meta>() {
                self.write(&Self::format_cfg_attr(&meta));
                self.newline();
                self.write_indent();
            }
        }

        // Format the value based on what it contains
        let value_str = match &arg.value {
            SpecArgValue::Expr(expr) => {
                // Special handling for arrays to format with proper indentation
                if let syn::Expr::Array(array) = expr {
                    let elem_strs = Vec::from_iter(array.elems.iter().map(format_expr));
                    self.format_array(&elem_strs)
                } else {
                    format_expr(expr)
                }
            }
            SpecArgValue::Pat(pat) => format_pattern(pat),
            SpecArgValue::Captures(captures) => self.format_captures(captures),
        };

        self.write(&format!("{}: {},", arg.keyword, value_str));
    }

    /// Format a group of captures.
    fn format_captures(&self, captures: &Captures) -> String {
        match captures {
            Captures::One(capture_expr) => Self::format_capture(capture_expr),
            Captures::Many { elems, .. } => {
                let elems = Vec::from_iter(elems.iter().map(Self::format_capture));
                self.format_array(&elems)
            }
        }
    }

    /// Format a single capture expression.
    fn format_capture(capture_expr: &CaptureExpr) -> String {
        let mut parts = vec![];
        if let Some(expr) = &capture_expr.expr {
            parts.push(format_expr(expr));
        }
        if capture_expr.as_.is_some() {
            parts.push("as".into());
        }
        if let Some(pat) = &capture_expr.pat {
            parts.push(format_pattern(pat));
        }
        parts.join(" ")
    }

    /// Format an array expression with proper indentation.
    fn format_array(&self, elems: &[String]) -> String {
        if elems.is_empty() {
            return "[]".to_string();
        }

        // For single element arrays, keep them compact
        if elems.len() == 1 {
            return elems[0].clone();
        }

        // Multi-element arrays: one per line with proper indentation
        let mut result = String::from("[\n");
        let elem_indent = " ".repeat(self.current_indent + self.settings.tab_spaces);

        // Determine if we should add trailing comma
        let add_trailing_comma = match self.settings.trailing_comma {
            TrailingComma::Always => true,
            TrailingComma::Never => false,
            TrailingComma::Vertical => elems.len() > 1,
        };

        for (i, elem) in elems.iter().enumerate() {
            result.push_str(&elem_indent);
            result.push_str(elem);

            if i < elems.len() - 1 || add_trailing_comma {
                result.push(',');
            }
            result.push('\n');
        }

        result.push_str(&" ".repeat(self.current_indent));
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

    /// Get the order index for a keyword (for sorting).
    pub fn keyword_order(&self, keyword: &Keyword) -> usize {
        match keyword {
            Keyword::Requires => 0,
            Keyword::Maintains => 1,
            Keyword::Captures => 2,
            Keyword::Binds => 3,
            Keyword::Ensures => 4,
            Keyword::Unknown(_) => 999,
        }
    }
}
