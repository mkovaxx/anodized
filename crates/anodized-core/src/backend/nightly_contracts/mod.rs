#[cfg(test)]
mod tests;

use crate::Spec;

use syn::{Error, Expr, ExprClosure, ItemFn, Result, parse_quote};

/// Takes the spec and the original function and returns the instrumented function.
pub fn instrument_fn(spec: &Spec, mut func: ItemFn) -> Result<ItemFn> {
    if let Some(capture) = spec.captures.first() {
        return Err(Error::new_spanned(
            &capture.expr,
            "`captures` are not supported by the nightly contracts backend",
        ));
    }

    let mut attrs = Vec::new();

    for condition in &spec.requires {
        let expr = &condition.expr;

        let attr = if let Some(cfg) = &condition.cfg {
            parse_quote! { #[cfg_attr(#cfg, contracts::requires(#expr))] }
        } else {
            parse_quote! { #[contracts::requires(#expr)] }
        };

        attrs.push(attr);
    }

    for condition in &spec.maintains {
        let expr = &condition.expr;

        let requires_attr = if let Some(cfg) = &condition.cfg {
            parse_quote! { #[cfg_attr(#cfg, contracts::requires(#expr))] }
        } else {
            parse_quote! { #[contracts::requires(#expr)] }
        };
        attrs.push(requires_attr);

        let ensures_closure: Expr = parse_quote! { |_| #expr };
        let ensures_attr = if let Some(cfg) = &condition.cfg {
            parse_quote! { #[cfg_attr(#cfg, contracts::ensures(#ensures_closure))] }
        } else {
            parse_quote! { #[contracts::ensures(#ensures_closure)] }
        };
        attrs.push(ensures_attr);
    }

    for postcondition in &spec.ensures {
        let closure: ExprClosure = postcondition.closure.clone();

        let attr = if let Some(cfg) = &postcondition.cfg {
            parse_quote! { #[cfg_attr(#cfg, contracts::ensures(#closure))] }
        } else {
            parse_quote! { #[contracts::ensures(#closure)] }
        };
        attrs.push(attr);
    }

    attrs.extend(func.attrs);
    func.attrs = attrs;

    Ok(func)
}
