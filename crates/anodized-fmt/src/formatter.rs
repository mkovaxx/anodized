use anodized_core::{Capture, PostCondition, PreCondition, Spec};
use syn::Meta;

use crate::config::{Config, TrailingComma};
use crate::expr_fmt::format_expr;

/// Format a complete Spec into a #[spec(...)] attribute string
pub fn format_spec(spec: &Spec, config: &Config, base_indent: usize) -> String {
    let mut output = String::from("#[spec(");

    let has_content = !spec.requires.is_empty()
        || !spec.maintains.is_empty()
        || !spec.captures.is_empty()
        || !spec.ensures.is_empty();

    if !has_content {
        output.push_str(")]");
        return output;
    }

    // For Phase 1, always use vertical layout
    output.push('\n');
    let indent = " ".repeat(base_indent + config.indent);

    // Format requires
    for condition in &spec.requires {
        let formatted = format_precondition("requires", condition, config);
        output.push_str(&indent);
        output.push_str(&formatted);
        output.push('\n');
    }

    // Format maintains
    for condition in &spec.maintains {
        let formatted = format_precondition("maintains", condition, config);
        output.push_str(&indent);
        output.push_str(&formatted);
        output.push('\n');
    }

    // Format captures
    if !spec.captures.is_empty() {
        let formatted = format_captures(&spec.captures, config, &indent);
        output.push_str(&indent);
        output.push_str(&formatted);
        output.push('\n');
    }

    // Format ensures
    for condition in &spec.ensures {
        let formatted = format_postcondition("ensures", condition, config);
        output.push_str(&indent);
        output.push_str(&formatted);
        output.push('\n');
    }

    output.push_str(&" ".repeat(base_indent));
    output.push_str(")]");

    output
}

/// Format a precondition (requires or maintains)
fn format_precondition(keyword: &str, condition: &PreCondition, config: &Config) -> String {
    let mut result = String::new();

    // Add cfg attribute if present
    if let Some(ref cfg) = condition.cfg {
        result.push_str(&format_cfg_attr(cfg));
        result.push('\n');
        result.push_str(&" ".repeat(config.indent));
    }

    // Extract the expression from the closure body
    let expr = &condition.closure.body;
    let expr_str = format_expr(expr);

    result.push_str(&format!("{}: {},", keyword, expr_str));

    result
}

/// Format a postcondition (ensures)
fn format_postcondition(keyword: &str, condition: &PostCondition, config: &Config) -> String {
    let mut result = String::new();

    // Add cfg attribute if present
    if let Some(ref cfg) = condition.cfg {
        result.push_str(&format_cfg_attr(cfg));
        result.push('\n');
        result.push_str(&" ".repeat(config.indent));
    }

    // Extract the expression from the closure body
    let expr = &condition.closure.body;
    let expr_str = format_expr(expr);

    result.push_str(&format!("{}: {},", keyword, expr_str));

    result
}

/// Format captures
fn format_captures(captures: &[Capture], config: &Config, indent: &str) -> String {
    if captures.is_empty() {
        return String::new();
    }

    if captures.len() == 1 {
        let capture = &captures[0];
        let expr_str = format_expr(&capture.expr);
        return format!("captures: {} as {},", expr_str, capture.alias);
    }

    // Multiple captures - use array format
    let mut result = String::from("captures: [\n");
    // Note: indent parameter already includes base_indent + config indent
    // For nested items, we need base_indent + 2*config_indent
    let double_indent = " ".repeat(indent.len() + config.indent);

    for (i, capture) in captures.iter().enumerate() {
        let expr_str = format_expr(&capture.expr);
        result.push_str(&double_indent);
        result.push_str(&format!("{} as {}", expr_str, capture.alias));

        if i < captures.len() - 1 || config.trailing_comma == TrailingComma::Always {
            result.push(',');
        }
        result.push('\n');
    }

    result.push_str(indent);
    result.push_str("],");

    result
}

/// Format a cfg attribute
fn format_cfg_attr(meta: &Meta) -> String {
    // For cfg attributes, we can use syn's Display implementation
    // which gives us clean output without extra spaces
    let tokens = quote::quote!(#meta);
    format!("#[cfg({})]", tokens)
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::parse_str;

    #[test]
    fn test_format_simple_spec() {
        // Parse a spec from tokens
        let spec: Spec = parse_str("requires: x > 0").expect("Failed to parse spec");

        let config = Config::default();
        let formatted = format_spec(&spec, &config, 0);

        assert!(formatted.contains("#[spec("));
        assert!(formatted.contains("requires:"));
        assert!(formatted.contains("x > 0"));
        assert!(formatted.ends_with(")]"));
    }

    #[test]
    fn test_format_spec_with_postcondition() {
        let spec: Spec = parse_str("ensures: *output > 0").expect("Failed to parse spec");

        let config = Config::default();
        let formatted = format_spec(&spec, &config, 0);

        assert!(formatted.contains("ensures:"));
        assert!(formatted.contains("> 0"));
    }

    #[test]
    fn test_format_empty_spec() {
        let spec: Spec = parse_str("").expect("Failed to parse spec");

        let config = Config::default();
        let formatted = format_spec(&spec, &config, 0);

        assert_eq!(formatted, "#[spec()]");
    }
}
