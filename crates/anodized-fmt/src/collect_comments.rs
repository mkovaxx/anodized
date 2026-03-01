use std::collections::HashMap;

use crop::Rope;
use proc_macro2::{LineColumn, Span, TokenStream};

/// Extract comments and empty lines from the gaps between tokens.
///
/// This function traverses a TokenStream and examines the text between consecutive
/// tokens. When tokens appear on different lines, it extracts any comments (lines
/// starting with //) and records empty lines.
///
/// Returns a HashMap mapping line numbers (0-indexed) to optional comment text:
/// - Some(comment_text) for lines with comments
/// - None for empty lines (preserves vertical spacing)
pub(crate) fn extract_whitespace_and_comments(
    source: &Rope,
    tokens: TokenStream,
) -> HashMap<usize, Option<String>> {
    let mut whitespace_and_comments = HashMap::new();
    let mut last_span: Option<Span> = None;

    traverse_token_stream(tokens, &mut |span: Span| {
        if let Some(last_span) = last_span
            && last_span.end().line != span.start().line
        {
            let text = get_text_between_spans(source, last_span.end(), span.start());
            for (idx, line) in text.lines().enumerate() {
                let comment = line
                    .to_string()
                    .split_once("//")
                    .map(|(_, txt)| txt)
                    .map(str::trim)
                    .map(ToOwned::to_owned);

                let line_index = last_span.end().line - 1 + idx;

                // Skip empty lines at token boundaries, but keep comments
                if comment.is_none()
                    && (line_index == last_span.end().line - 1
                        || line_index == span.start().line - 1)
                {
                    continue;
                }

                whitespace_and_comments.insert(line_index, comment);
            }
        }
        last_span = Some(span);
    });

    whitespace_and_comments
}

/// Recursively traverse a TokenStream, calling the callback for each token's span.
fn traverse_token_stream(tokens: TokenStream, cb: &mut impl FnMut(Span)) {
    for token in tokens {
        match token {
            proc_macro2::TokenTree::Group(group) => {
                cb(group.span_open());
                traverse_token_stream(group.stream(), cb);
                cb(group.span_close());
            }
            _ => cb(token.span()),
        }
    }
}

/// Extract text from the source Rope between two line/column positions.
fn get_text_between_spans(rope: &Rope, start: LineColumn, end: LineColumn) -> String {
    let start_byte = line_column_to_byte(rope, start);
    let end_byte = line_column_to_byte(rope, end);

    rope.byte_slice(start_byte..end_byte).to_string()
}

/// Convert a LineColumn position to a byte offset in the Rope.
pub fn line_column_to_byte(source: &Rope, point: LineColumn) -> usize {
    let line_byte = source.byte_of_line(point.line - 1);
    let line = source.line(point.line - 1);
    let char_byte: usize = line.chars().take(point.column).map(|c| c.len_utf8()).sum();
    line_byte + char_byte
}

/// Check if a TokenStream contains comments inside nested structures.
pub fn has_nested_structure_comments(source: &Rope, tokens: TokenStream) -> bool {
    has_nested_structure_comments_inner(source, tokens, 0)
}

// Recursively check for comments inside nested groups, tracking depth to avoid false positives.
fn has_nested_structure_comments_inner(source: &Rope, tokens: TokenStream, depth: usize) -> bool {
    for token in tokens {
        if let proc_macro2::TokenTree::Group(group) = token {
            // Check for comments inside any nested structure (arrays, blocks, structs, etc.)
            // We need to include the opening delimiter to check for comments after the opening brace/bracket
            let mut group_comments = HashMap::new();
            let opening_span = group.span_open();

            // Check gaps between tokens inside the group, including after the opening delimiter
            let mut last_span: Option<Span> = Some(opening_span);
            traverse_token_stream(group.stream(), &mut |span: Span| {
                if let Some(last) = last_span {
                    if last.end().line != span.start().line {
                        let text = get_text_between_spans(source, last.end(), span.start());
                        for (idx, line) in text.lines().enumerate() {
                            let comment = line
                                .to_string()
                                .split_once("//")
                                .map(|(_, txt)| txt)
                                .map(str::trim)
                                .map(ToOwned::to_owned);

                            let line_index = last.end().line - 1 + idx;

                            if comment.is_some() {
                                group_comments.insert(line_index, comment);
                            }
                        }
                    }
                }
                last_span = Some(span);
            });

            if !group_comments.is_empty() {
                return true;
            }

            // Recursively check nested groups at increased depth
            if has_nested_structure_comments_inner(source, group.stream(), depth + 1) {
                return true;
            }
        }
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_simple_comment() {
        let source_text = r#"
            requires: x > 0,
            // This is a comment
            ensures: *output > 0
            "#;
        let rope = Rope::from(source_text);
        let tokens: TokenStream = source_text.parse().unwrap();

        let comments = extract_whitespace_and_comments(&rope, tokens);

        // Should find the comment on line 2 (0-indexed)
        assert!(comments.contains_key(&2));
        assert_eq!(
            comments.get(&2),
            Some(&Some("This is a comment".to_string()))
        );
    }

    #[test]
    fn test_no_comments() {
        let source_text = r#"requires: x > 0, ensures: *output > 0"#;
        let rope = Rope::from(source_text);
        let tokens: TokenStream = source_text.parse().unwrap();

        let comments = extract_whitespace_and_comments(&rope, tokens);

        // Should be empty - no gaps between lines
        assert!(comments.is_empty());
    }

    #[test]
    fn test_multiple_comments() {
        let source_text = r#"
            requires: x > 0,
            // First comment
            // Second comment
            ensures: *output > 0,
            // Third comment
            binds: result
            "#;
        let rope = Rope::from(source_text);
        let tokens: TokenStream = source_text.parse().unwrap();

        let comments = extract_whitespace_and_comments(&rope, tokens);

        // Should find all three comments
        assert!(comments.len() == 3);
        assert_eq!(comments.get(&2), Some(&Some("First comment".to_string())));
        assert_eq!(comments.get(&3), Some(&Some("Second comment".to_string())));
        assert_eq!(comments.get(&5), Some(&Some("Third comment".to_string())));
    }

    #[test]
    fn test_empty_lines() {
        let source_text = r#"
            requires: x > 0,

            ensures: *output > 0
            "#;
        let rope = Rope::from(source_text);
        let tokens: TokenStream = source_text.parse().unwrap();

        let comments = extract_whitespace_and_comments(&rope, tokens);

        // Should record the empty line as None
        let has_empty_line = comments.values().any(|v| v.is_none());
        assert!(has_empty_line);
    }
}
