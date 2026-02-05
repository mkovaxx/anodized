use anodized_core::annotate::syntax::{SpecArg, SpecArgValue, SpecArgs};
use syn::Meta;

use crate::config::Config;
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

    // Format each argument
    for arg in &spec_args.args {
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
        SpecArgValue::Expr(expr) => format_expr(expr),
        SpecArgValue::Pat(pat) => format_pattern(pat),
    };

    result.push_str(&format!("{}: {},", arg.keyword, value_str));

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

        assert!(formatted.contains("#[spec("));
        assert!(formatted.contains("requires:"));
        assert!(formatted.contains("x > 0"));
        assert!(formatted.ends_with(")]"));
    }

    #[test]
    fn test_format_spec_with_postcondition() {
        let spec_args: SpecArgs = parse_str("ensures: *output > 0").expect("Failed to parse spec");

        let config = Config::default();
        let formatted = format_spec_args(&spec_args, &config, 0);

        assert!(formatted.contains("ensures:"));
        assert!(formatted.contains("> 0"));
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

        assert!(formatted.contains("binds:"));
        assert!(formatted.contains("result"));
        assert!(formatted.contains("ensures:"));
    }
}
