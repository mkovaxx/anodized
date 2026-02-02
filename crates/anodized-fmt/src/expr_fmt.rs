use proc_macro2::{Punct, Spacing, TokenStream};
use quote::ToTokens;
use syn::{Expr, Pat, UnOp};

/// Format an expression
/// Uses quote! but handles dereference specially to avoid space after *
pub fn format_expr(expr: &Expr) -> String {
    format_expr_impl(expr).to_string()
}

/// Internal implementation that builds TokenStream with special dereference handling
fn format_expr_impl(expr: &Expr) -> TokenStream {
    match expr {
        // Handle unary dereference specially to avoid space after *
        Expr::Unary(expr_unary) if matches!(expr_unary.op, UnOp::Deref(_)) => {
            let inner = format_expr_impl(&expr_unary.expr);
            // Manually construct * without space
            let mut tokens = TokenStream::new();
            tokens.extend(std::iter::once(proc_macro2::TokenTree::Punct(Punct::new(
                '*',
                Spacing::Joint,
            ))));
            tokens.extend(inner);
            tokens
        }
        // Handle binary expressions recursively to process any nested derefs
        Expr::Binary(expr_binary) => {
            let left = format_expr_impl(&expr_binary.left);
            let op = &expr_binary.op;
            let right = format_expr_impl(&expr_binary.right);
            quote::quote!(#left #op #right)
        }
        // Handle parenthesized expressions recursively
        Expr::Paren(expr_paren) => {
            let inner = format_expr_impl(&expr_paren.expr);
            quote::quote!((#inner))
        }
        // For all other expressions, use their default token representation
        _ => expr.to_token_stream(),
    }
}

/// Format a pattern (for binds parameter)
pub fn format_pattern(pat: &Pat) -> String {
    quote::quote!(#pat).to_string()
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
