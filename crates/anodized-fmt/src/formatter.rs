use anodized_core::annotate::syntax::{SpecArg, SpecArgValue, SpecArgs};
use syn::Meta;

use crate::config::{Config, TrailingComma};
use crate::expr_fmt::{format_expr, format_pattern};

/// Format a complete SpecArgs into a #[spec(...)] attribute string
pub fn format_spec_args(spec_args: &SpecArgs, config: &Config, base_indent: usize) -> String {
    let mut output = String::from("#[spec(");

    if spec_args.args.is_empty() {
        output.push_str(")]");
        return output;
    }

    // For Phase 1, always use vertical layout
    output.push('\n');
    let indent = " ".repeat(base_indent + config.indent);

    // Collect args into a Vec so we can sort if needed
    let mut args: Vec<&SpecArg> = spec_args.args.iter().collect();

    // Sort arguments if configured
    if config.sort_args {
        args.sort_by_key(|arg| &arg.keyword);
    }

    // Format each argument
    for arg in args {
        let formatted = format_spec_arg(arg, config);
        output.push_str(&indent);
        output.push_str(&formatted);
        output.push('\n');
    }

    output.push_str(&" ".repeat(base_indent));
    output.push_str(")]");

    output
}

/// Format a single SpecArg
fn format_spec_arg(arg: &SpecArg, config: &Config) -> String {
    let mut result = String::new();

    // Add cfg attribute if present
    if let Some(cfg_attr) = find_cfg_attribute(&arg.attrs) {
        if let Ok(meta) = cfg_attr.parse_args::<Meta>() {
            result.push_str(&format_cfg_attr(&meta));
            result.push('\n');
            result.push_str(&" ".repeat(config.indent));
        }
    }

    // Format the value based on what it contains
    let value_str = match &arg.value {
        SpecArgValue::Expr(expr) => {
            // Special handling for arrays to format with proper indentation
            if let syn::Expr::Array(array) = expr {
                format_array(array, config)
            } else {
                format_expr(expr)
            }
        }
        SpecArgValue::Pat(pat) => format_pattern(pat),
    };

    result.push_str(&format!("{}: {},", arg.keyword, value_str));

    result
}

/// Format an array expression with proper indentation
fn format_array(array: &syn::ExprArray, config: &Config) -> String {
    if array.elems.is_empty() {
        return "[]".to_string();
    }

    // For single element arrays, keep them compact
    if array.elems.len() == 1 {
        let elem_str = format_expr(&array.elems[0]);
        return format!("[{}]", elem_str);
    }

    // Multi-element arrays: one per line with proper indentation
    let mut result = String::from("[\n");
    let elem_indent = " ".repeat(config.indent * 2);

    // Determine if we should add trailing comma
    let add_trailing_comma = match config.trailing_comma {
        TrailingComma::Always => true,
        TrailingComma::Never => false,
        TrailingComma::Auto => array.elems.len() > 1, // Multi-line = add trailing comma
    };

    for (i, elem) in array.elems.iter().enumerate() {
        let elem_str = format_expr(elem);
        result.push_str(&elem_indent);
        result.push_str(&elem_str);

        // Add comma after each element, including last if configured
        if i < array.elems.len() - 1 || add_trailing_comma {
            result.push(',');
        }
        result.push('\n');
    }

    result.push_str(&" ".repeat(config.indent));
    result.push(']');

    result
}

/// Find cfg attribute in the attribute list
fn find_cfg_attribute(attrs: &[syn::Attribute]) -> Option<&syn::Attribute> {
    attrs.iter().find(|attr| attr.path().is_ident("cfg"))
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
        // Parse a spec from tokens using SpecArgs
        let spec_args: SpecArgs = parse_str("requires: x > 0").expect("Failed to parse spec");

        let config = Config::default();
        let formatted = format_spec_args(&spec_args, &config, 0);

        let expected = "#[spec(\n    requires: x > 0,\n)]";
        assert_eq!(formatted, expected);
    }

    #[test]
    fn test_format_spec_with_postcondition() {
        let spec_args: SpecArgs = parse_str("ensures: *output > 0").expect("Failed to parse spec");

        let config = Config::default();
        let formatted = format_spec_args(&spec_args, &config, 0);

        let expected = "#[spec(\n    ensures: *output > 0,\n)]";
        assert_eq!(formatted, expected);
    }

    #[test]
    fn test_format_empty_spec() {
        let spec_args: SpecArgs = parse_str("").expect("Failed to parse spec");

        let config = Config::default();
        let formatted = format_spec_args(&spec_args, &config, 0);

        assert_eq!(formatted, "#[spec()]");
    }

    #[test]
    fn test_format_spec_with_binds() {
        let spec_args: SpecArgs =
            parse_str("binds: result, ensures: result > 0").expect("Failed to parse spec");

        let config = Config::default();
        let formatted = format_spec_args(&spec_args, &config, 0);

        let expected = "#[spec(\n    binds: result,\n    ensures: result > 0,\n)]";
        assert_eq!(formatted, expected);
    }

    #[test]
    fn test_format_spec_with_sorting() {
        // Parse args in wrong order: ensures, binds, requires
        let spec_args: SpecArgs = parse_str("ensures: result > 0, binds: result, requires: x > 0")
            .expect("Failed to parse spec");

        let config = Config::default(); // sort_args = true by default

        let formatted = format_spec_args(&spec_args, &config, 0);

        // Should be sorted: requires, binds, ensures
        let expected =
            "#[spec(\n    requires: x > 0,\n    binds: result,\n    ensures: result > 0,\n)]";
        assert_eq!(formatted, expected);
    }

    #[test]
    fn test_format_spec_without_sorting() {
        // Parse args in wrong order: ensures, binds, requires
        let spec_args: SpecArgs = parse_str("ensures: result > 0, binds: result, requires: x > 0")
            .expect("Failed to parse spec");

        let mut config = Config::default();
        config.sort_args = false;

        let formatted = format_spec_args(&spec_args, &config, 0);

        // Should preserve original order: ensures, binds, requires
        let expected =
            "#[spec(\n    ensures: result > 0,\n    binds: result,\n    requires: x > 0,\n)]";
        assert_eq!(formatted, expected);
    }

    #[test]
    fn test_format_spec_with_array() {
        let spec_args: SpecArgs =
            parse_str("ensures: [a > 0, b > 0]").expect("Failed to parse spec");

        let config = Config::default();
        let formatted = format_spec_args(&spec_args, &config, 0);

        let expected = "#[spec(\n    ensures: [\n        a > 0,\n        b > 0,\n    ],\n)]";
        assert_eq!(formatted, expected);
    }

    #[test]
    fn test_format_spec_with_single_element_array() {
        let spec_args: SpecArgs = parse_str("ensures: [a > 0]").expect("Failed to parse spec");

        let config = Config::default();
        let formatted = format_spec_args(&spec_args, &config, 0);

        let expected = "#[spec(\n    ensures: [a > 0],\n)]";
        assert_eq!(formatted, expected);
    }

    #[test]
    fn test_format_array_trailing_comma_always() {
        let spec_args: SpecArgs =
            parse_str("ensures: [a > 0, b > 0]").expect("Failed to parse spec");

        let mut config = Config::default();
        config.trailing_comma = TrailingComma::Always;
        let formatted = format_spec_args(&spec_args, &config, 0);

        let expected = "#[spec(\n    ensures: [\n        a > 0,\n        b > 0,\n    ],\n)]";
        assert_eq!(formatted, expected);
    }

    #[test]
    fn test_format_array_trailing_comma_never() {
        let spec_args: SpecArgs =
            parse_str("ensures: [a > 0, b > 0]").expect("Failed to parse spec");

        let mut config = Config::default();
        config.trailing_comma = TrailingComma::Never;
        let formatted = format_spec_args(&spec_args, &config, 0);

        let expected = "#[spec(\n    ensures: [\n        a > 0,\n        b > 0\n    ],\n)]";
        assert_eq!(formatted, expected);
    }

    #[test]
    fn test_format_array_trailing_comma_auto() {
        let spec_args: SpecArgs =
            parse_str("ensures: [a > 0, b > 0]").expect("Failed to parse spec");

        let mut config = Config::default();
        config.trailing_comma = TrailingComma::Auto;
        let formatted = format_spec_args(&spec_args, &config, 0);

        // Auto adds trailing comma for multi-line arrays
        let expected = "#[spec(\n    ensures: [\n        a > 0,\n        b > 0,\n    ],\n)]";
        assert_eq!(formatted, expected);
    }
}
