/// Information about a located #[spec] attribute
#[derive(Debug, Clone)]
pub struct SpecLocation {
    /// Byte offset where the attribute starts (at '#')
    pub start: usize,
    /// Byte offset where the attribute ends (after the closing ']')
    pub end: usize,
    /// The original attribute text including #[spec(...)]
    pub original_text: String,
    /// The content inside spec(...), without the #[spec( and )]
    pub content: String,
}

#[derive(Debug, thiserror::Error)]
pub enum FindError {
    #[error("Unmatched bracket in spec attribute at position {0}")]
    UnmatchedBracket(usize),

    #[error("Invalid spec attribute structure at position {0}")]
    InvalidStructure(usize),
}

/// Find all #[spec(...)] attributes in the source code
pub fn find_spec_attributes(source: &str) -> Result<Vec<SpecLocation>, FindError> {
    let mut locations = Vec::new();
    let mut search_start = 0;

    while let Some(attr_start) = source[search_start..].find("#[spec(") {
        let abs_start = search_start + attr_start;
        let _remaining = &source[abs_start..];

        // Find the matching closing for the outer brackets [...]
        // We need to find the ] that closes the #[...] attribute
        let content_start = abs_start + "#[spec(".len();

        // Find matching ) for the spec(...)
        let content_end_offset = find_matching_paren(&source[content_start..], '(', ')')
            .map_err(|_| FindError::UnmatchedBracket(content_start))?;
        let abs_content_end = content_start + content_end_offset;

        // Now find the closing ]
        // It should come right after the ), possibly with whitespace
        let after_paren = &source[abs_content_end..];
        let close_bracket_pos = after_paren
            .find(']')
            .ok_or(FindError::InvalidStructure(abs_content_end))?;

        let abs_end = abs_content_end + close_bracket_pos + 1;

        let original_text = source[abs_start..abs_end].to_string();
        let content = source[content_start..abs_content_end].to_string();

        locations.push(SpecLocation {
            start: abs_start,
            end: abs_end,
            original_text,
            content,
        });

        search_start = abs_end;
    }

    Ok(locations)
}

/// Find the position of the matching closing bracket
/// Returns the position relative to the start of the input string
fn find_matching_paren(s: &str, open: char, close: char) -> Result<usize, FindError> {
    let mut depth = 1; // We're already past the opening bracket
    let mut in_string = false;
    let mut in_char = false;
    let mut escape_next = false;
    let mut pos = 0;

    let chars: Vec<char> = s.chars().collect();

    while pos < chars.len() {
        let ch = chars[pos];

        if escape_next {
            escape_next = false;
            pos += 1;
            continue;
        }

        match ch {
            '\\' if in_string || in_char => {
                escape_next = true;
            }
            '"' if !in_char => {
                in_string = !in_string;
            }
            '\'' if !in_string => {
                in_char = !in_char;
            }
            c if c == open && !in_string && !in_char => {
                depth += 1;
            }
            c if c == close && !in_string && !in_char => {
                depth -= 1;
                if depth == 0 {
                    return Ok(pos);
                }
            }
            _ => {}
        }

        pos += 1;
    }

    Err(FindError::UnmatchedBracket(0))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_simple_spec() {
        let source = r#"
#[spec(requires: x > 0)]
fn foo(x: i32) -> i32 { x + 1 }
"#;

        let result = find_spec_attributes(source).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].content, "requires: x > 0");
    }

    #[test]
    fn test_find_multiple_specs() {
        let source = r#"
#[spec(requires: x > 0)]
fn foo(x: i32) -> i32 { x + 1 }

#[spec(ensures: *output > 0)]
fn bar() -> i32 { 42 }
"#;

        let result = find_spec_attributes(source).unwrap();
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn test_find_spec_with_nested_parens() {
        let source = r#"
#[spec(requires: (x > 0 && (y > 0 || z > 0)))]
fn foo(x: i32, y: i32, z: i32) -> i32 { x + y + z }
"#;

        let result = find_spec_attributes(source).unwrap();
        assert_eq!(result.len(), 1);
        assert!(result[0].content.contains("(x > 0 && (y > 0 || z > 0))"));
    }

    #[test]
    fn test_find_spec_with_string() {
        let source = r##"
#[spec(requires: s == "hello (world)")]
fn foo(s: &str) -> bool { true }
"##;

        let result = find_spec_attributes(source).unwrap();
        assert_eq!(result.len(), 1);
        assert!(result[0].content.contains(r#"s == "hello (world)""#));
    }
}
