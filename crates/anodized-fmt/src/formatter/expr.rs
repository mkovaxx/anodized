use quote::ToTokens;
use syn::{Expr, Pat};

/// Format an expression using prettyplease
/// This properly formats Rust expressions without excessive whitespace
pub fn format_expr(expr: &Expr) -> String {
    // prettyplease::unparse works on syn::File, so we need to wrap the expression
    // in a const item to format it
    let item = syn::Item::Const(syn::ItemConst {
        attrs: vec![],
        vis: syn::Visibility::Inherited,
        const_token: Default::default(),
        ident: syn::Ident::new("DUMMY", proc_macro2::Span::call_site()),
        generics: Default::default(),
        colon_token: Default::default(),
        ty: Box::new(syn::parse_quote!(())),
        eq_token: Default::default(),
        expr: Box::new(expr.clone()),
        semi_token: Default::default(),
    });

    let file = syn::File {
        shebang: None,
        attrs: vec![],
        items: vec![item],
    };

    let formatted = prettyplease::unparse(&file);

    // Extract the expression from "const DUMMY: () = <expr>;"
    let result = formatted
        .strip_prefix("const DUMMY: () = ")
        .and_then(|s| s.strip_suffix(";\n"))
        .unwrap_or(expr.to_token_stream().to_string().as_str())
        .trim()
        .to_string();

    // Remove extra spaces before commas in arrays and tuples
    remove_spaces_before_commas(&result)
}

/// Format a pattern (for binds parameter)
pub fn format_pattern(pat: &Pat) -> String {
    // For patterns, quote works fine
    let result = quote::quote!(#pat).to_string();

    // Remove extra spaces before commas in patterns too
    remove_spaces_before_commas(&result)
}

/// Remove spaces before commas in formatted output
/// This handles prettyplease's formatting of arrays/tuples like `[a , b , c]` -> `[a, b, c]`
fn remove_spaces_before_commas(s: &str) -> String {
    s.replace(" ,", ",")
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::parse_quote;

    #[test]
    fn test_format_simple_expr() {
        let expr: Expr = parse_quote!(x > 0);
        let formatted = format_expr(&expr);
        assert!(formatted.contains("x > 0"));
    }

    #[test]
    fn test_format_complex_expr() {
        let expr: Expr = parse_quote!(x > 0 && y < 100);
        let formatted = format_expr(&expr);
        assert!(formatted.contains("x > 0"));
        assert!(formatted.contains("y < 100"));
    }

    #[test]
    fn test_format_pattern() {
        let pat: Pat = parse_quote!(output);
        let formatted = format_pattern(&pat);
        assert_eq!(formatted, "output");
    }

    #[test]
    fn test_format_tuple_pattern() {
        let pat: Pat = parse_quote!((a, b));
        let formatted = format_pattern(&pat);
        assert!(formatted.contains("a"));
        assert!(formatted.contains("b"));
    }

    #[test]
    fn test_format_deref_expr() {
        let expr: Expr = parse_quote!(*balance);
        let formatted = format_expr(&expr);
        assert_eq!(formatted, "*balance");
    }

    #[test]
    fn test_format_deref_comparison() {
        let expr: Expr = parse_quote!(*balance >= amount);
        let formatted = format_expr(&expr);
        assert!(formatted.contains("*balance"));
        assert!(!formatted.contains("* balance"));
    }

    #[test]
    fn test_format_multiplication() {
        let expr: Expr = parse_quote!(a * b);
        let formatted = format_expr(&expr);
        // Multiplication should keep spaces
        assert!(formatted.contains("a * b"));
    }

    #[test]
    fn test_format_deref_in_expression() {
        let expr: Expr = parse_quote!(*balance == initial_balance - amount);
        let formatted = format_expr(&expr);
        assert!(formatted.contains("*balance"));
        assert!(!formatted.contains("* balance"));
    }

    #[test]
    fn test_format_nested_deref() {
        let expr: Expr = parse_quote!(**ptr);
        let formatted = format_expr(&expr);
        assert_eq!(formatted, "**ptr");
    }
}
