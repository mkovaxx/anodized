//! Core interoperability for the Anodized correctness ecosystem.
//!
#![cfg_attr(not(doctest), doc = include_str!("../README.md"))]

use proc_macro2::Span;
use quote::{ToTokens, quote};
use syn::{
    Attribute, Block, Expr, ExprClosure, Ident, Meta, Pat, Token,
    parse::{Parse, ParseStream, Result},
    parse_quote,
    punctuated::Punctuated,
    spanned::Spanned,
};

/// A spec specifies the intended behavior of a function or method.
#[derive(Debug)]
pub struct Spec {
    /// Preconditions: conditions that must hold when the function is called.
    pub requires: Vec<Condition>,
    /// Invariants: conditions that must hold both when the function is called and when it returns.
    pub maintains: Vec<Condition>,
    /// Clone bindings: expressions to clone at function entry for use in postconditions.
    pub clones: Vec<CloneBinding>,
    /// Postconditions: conditions that must hold when the function returns.
    pub ensures: Vec<ConditionClosure>,
}

/// A condition represented by a `bool`-valued expression.
#[derive(Debug)]
pub struct Condition {
    /// The expression.
    pub expr: Expr,
    /// **Static analyzers can safely ignore this field.**
    ///
    /// Build configuration filter to decide whether to add runtime checks.
    /// Passed to a `cfg!()` guard in the instrumented function.
    pub cfg: Option<Meta>,
}

/// A condition represented by a `bool`-valued closure.
#[derive(Debug)]
pub struct ConditionClosure {
    /// The closure.
    pub closure: ExprClosure,
    /// **Static analyzers can safely ignore this field.**
    ///
    /// Build configuration filter to decide whether to add runtime checks.
    /// Passed to a `cfg!()` guard in the instrumented function.
    pub cfg: Option<Meta>,
}

/// A clone binding that captures an expression's value at function entry.
#[derive(Debug)]
pub struct CloneBinding {
    /// The expression to clone.
    pub expr: Expr,
    /// The identifier to bind the cloned value to.
    pub alias: Ident,
}

impl Parse for Spec {
    fn parse(input: ParseStream) -> Result<Self> {
        let args = Punctuated::<SpecArg, Token![,]>::parse_terminated(input)?;

        let mut last_arg_order: Option<ArgOrder> = None;
        let mut requires: Vec<Condition> = vec![];
        let mut maintains: Vec<Condition> = vec![];
        let mut clones: Vec<CloneBinding> = vec![];
        let mut binds_pattern: Option<Pat> = None;
        let mut ensures_exprs: Vec<Condition> = vec![];

        for arg in args {
            let current_arg_order = arg.get_order();
            if let Some(last_order) = last_arg_order {
                if current_arg_order < last_order {
                    return Err(syn::Error::new(
                        arg.get_keyword_span(),
                        "parameters are out of order: their order must be `requires`, `maintains`, `clones`, `binds`, `ensures`",
                    ));
                }
            }
            last_arg_order = Some(current_arg_order);

            match arg {
                SpecArg::Requires { cfg, expr, .. } => {
                    if let Expr::Array(conditions) = expr {
                        requires.extend(conditions.elems.into_iter().map(|expr| Condition {
                            expr,
                            cfg: cfg.clone(),
                        }));
                    } else {
                        requires.push(Condition { expr, cfg });
                    }
                }
                SpecArg::Maintains { cfg, expr, .. } => {
                    if let Expr::Array(conditions) = expr {
                        maintains.extend(conditions.elems.into_iter().map(|expr| Condition {
                            expr,
                            cfg: cfg.clone(),
                        }));
                    } else {
                        maintains.push(Condition { expr, cfg });
                    }
                }
                SpecArg::Clones { bindings, .. } => {
                    clones.extend(bindings);
                }
                SpecArg::Binds { keyword, pattern } => {
                    if binds_pattern.is_some() {
                        return Err(syn::Error::new(
                            keyword.span(),
                            "multiple `binds` parameters are not allowed",
                        ));
                    }
                    binds_pattern = Some(pattern);
                }
                SpecArg::Ensures { cfg, expr, .. } => {
                    if let Expr::Array(conditions) = expr {
                        ensures_exprs.extend(conditions.elems.into_iter().map(|expr| Condition {
                            expr,
                            cfg: cfg.clone(),
                        }));
                    } else {
                        ensures_exprs.push(Condition { expr, cfg });
                    }
                }
            }
        }

        let default_closure_pattern = binds_pattern
            .as_ref()
            .map(|p| p.to_token_stream())
            .unwrap_or_else(|| quote! { output });

        let ensures = ensures_exprs
            .into_iter()
            .map(|condition| {
                let closure: ExprClosure = if let Expr::Closure(closure) = condition.expr {
                    closure
                } else {
                    // Convert "naked" postcondition to closure
                    let inner_condition = condition.expr;
                    parse_quote! { |#default_closure_pattern| #inner_condition }
                };
                Ok(ConditionClosure {
                    closure,
                    cfg: condition.cfg,
                })
            })
            .collect::<Result<Vec<ConditionClosure>>>()?;

        Ok(Spec {
            requires,
            maintains,
            clones,
            ensures,
        })
    }
}

#[derive(PartialEq, PartialOrd, Clone, Copy, Debug)]
enum ArgOrder {
    Requires,
    Maintains,
    Clones,
    Binds,
    Ensures,
}

/// An intermediate enum to help parse either a condition or a `binds` setting.
enum SpecArg {
    Requires {
        keyword: kw::requires,
        cfg: Option<Meta>,
        expr: Expr,
    },
    Ensures {
        keyword: kw::ensures,
        cfg: Option<Meta>,
        expr: Expr,
    },
    Maintains {
        keyword: kw::maintains,
        cfg: Option<Meta>,
        expr: Expr,
    },
    Clones {
        keyword: kw::clones,
        bindings: Vec<CloneBinding>,
    },
    Binds {
        keyword: kw::binds,
        pattern: Pat,
    },
}

impl SpecArg {
    fn get_order(&self) -> ArgOrder {
        match self {
            SpecArg::Requires { .. } => ArgOrder::Requires,
            SpecArg::Maintains { .. } => ArgOrder::Maintains,
            SpecArg::Clones { .. } => ArgOrder::Clones,
            SpecArg::Binds { .. } => ArgOrder::Binds,
            SpecArg::Ensures { .. } => ArgOrder::Ensures,
        }
    }

    fn get_keyword_span(&self) -> Span {
        match self {
            SpecArg::Requires { keyword, .. } => keyword.span,
            SpecArg::Ensures { keyword, .. } => keyword.span,
            SpecArg::Maintains { keyword, .. } => keyword.span,
            SpecArg::Clones { keyword, .. } => keyword.span,
            SpecArg::Binds { keyword, .. } => keyword.span,
        }
    }
}

impl Parse for SpecArg {
    fn parse(input: ParseStream) -> Result<Self> {
        let attrs = input.call(Attribute::parse_outer)?;
        let cfg = parse_cfg_attribute(&attrs)?;

        let lookahead = input.lookahead1();
        if lookahead.peek(kw::clones) {
            if cfg.is_some() {
                return Err(syn::Error::new(
                    attrs[0].span(),
                    "`cfg` attribute is not supported on `clones`",
                ));
            }
            
            // Parse `clones: <bindings>`
            let keyword = input.parse::<kw::clones>()?;
            input.parse::<Token![:]>()?;
            
            // Parse an expression and interpret as binding(s)
            let expr: Expr = input.parse()?;
            
            let bindings = match expr {
                // Array: interpret as list of bindings
                Expr::Array(array) => interpret_array_as_clone_bindings(array)?,
                // Single expression: interpret as single binding
                _ => vec![interpret_as_clone_binding(expr)?],
            };
            
            Ok(SpecArg::Clones {
                keyword,
                bindings,
            })
        } else if lookahead.peek(kw::binds) {
            if cfg.is_some() {
                return Err(syn::Error::new(
                    attrs[0].span(),
                    "`cfg` attribute is not supported on `binds`",
                ));
            }

            // Parse `binds: <pattern>`
            let keyword = input.parse::<kw::binds>()?;
            input.parse::<Token![:]>()?;
            Ok(SpecArg::Binds {
                keyword,
                pattern: Pat::parse_single(input)?,
            })
        } else if lookahead.peek(kw::requires) {
            // Parse `requires: <conditions>`
            let keyword = input.parse::<kw::requires>()?;
            input.parse::<Token![:]>()?;
            Ok(SpecArg::Requires {
                keyword,
                cfg,
                expr: input.parse()?,
            })
        } else if lookahead.peek(kw::maintains) {
            // Parse `maintains: <conditions>`
            let keyword = input.parse::<kw::maintains>()?;
            input.parse::<Token![:]>()?;
            Ok(SpecArg::Maintains {
                keyword,
                cfg,
                expr: input.parse()?,
            })
        } else if lookahead.peek(kw::ensures) {
            // Parse `ensures: <conditions>`
            let keyword = input.parse::<kw::ensures>()?;
            input.parse::<Token![:]>()?;
            Ok(SpecArg::Ensures {
                keyword,
                cfg,
                expr: input.parse()?,
            })
        } else {
            Err(lookahead.error())
        }
    }
}

/// Try to interpret an Expr::Array as a list of CloneBindings
fn interpret_array_as_clone_bindings(array: syn::ExprArray) -> Result<Vec<CloneBinding>> {
    let mut bindings = Vec::new();
    
    for elem in array.elems {
        // Try to interpret each element as a binding
        // If any fails, propagate that error immediately
        bindings.push(interpret_as_clone_binding(elem)?);
    }
    
    Ok(bindings)
}

/// Try to interpret an Expr as a single CloneBinding
fn interpret_as_clone_binding(expr: Expr) -> Result<CloneBinding> {
    match expr {
        // Simple identifier: count -> old_count
        Expr::Path(ref path) 
            if path.path.segments.len() == 1 
            && path.path.leading_colon.is_none()
            && path.attrs.is_empty()
            && path.qself.is_none() => 
        {
            let ident = &path.path.segments[0].ident;
            let alias = Ident::new(&format!("old_{}", ident), ident.span());
            Ok(CloneBinding { expr, alias })
        }
        // Cast expression: value as old_value
        Expr::Cast(cast) => {
            // The cast.ty should be a simple identifier that we use as the alias
            if let syn::Type::Path(ref type_path) = *cast.ty {
                if type_path.path.segments.len() == 1 
                    && type_path.path.leading_colon.is_none()
                    && type_path.qself.is_none()
                {
                    let alias = type_path.path.segments[0].ident.clone();
                    return Ok(CloneBinding {
                        expr: *cast.expr,
                        alias,
                    });
                }
            }
            Err(syn::Error::new_spanned(
                cast,
                "alias must be a simple identifier"
            ))
        }
        // Any other expression requires an explicit alias
        _ => Err(syn::Error::new_spanned(
            expr,
            "complex expressions require an explicit alias using `as`"
        ))
    }
}


fn parse_cfg_attribute(attrs: &[Attribute]) -> Result<Option<Meta>> {
    let mut cfg_attrs: Vec<Meta> = vec![];

    for attr in attrs {
        if attr.path().is_ident("cfg") {
            cfg_attrs.push(attr.parse_args()?);
        } else {
            return Err(syn::Error::new(
                attr.span(),
                "unsupported attribute; only `cfg` is allowed",
            ));
        }
    }

    if cfg_attrs.len() > 1 {
        return Err(syn::Error::new(
            cfg_attrs[1].span(),
            "multiple `cfg` attributes are not supported",
        ));
    }

    Ok(cfg_attrs.pop())
}

/// Custom keywords for parsing. This allows us to use `requires`, `ensures`, etc.,
/// as if they were built-in Rust keywords during parsing.
mod kw {
    syn::custom_keyword!(requires);
    syn::custom_keyword!(maintains);
    syn::custom_keyword!(clones);
    syn::custom_keyword!(binds);
    syn::custom_keyword!(ensures);
}

/// Takes the spec and the original body and returns a new instrumented function body.
pub fn instrument_fn_body(spec: &Spec, original_body: &Block, is_async: bool) -> Result<Block> {
    // The identifier for the return value binding. It's hygienic to prevent collisions.
    let binding_ident = Ident::new("__anodized_output", Span::mixed_site());

    // --- Generate Precondition Checks ---
    let preconditions = spec
        .requires
        .iter()
        .map(|condition| {
            let expr = &condition.expr;
            let msg = format!("Precondition failed: {}", expr.to_token_stream());
            let assert = quote! { assert!(#expr, #msg); };
            if let Some(cfg) = &condition.cfg {
                quote! { if cfg!(#cfg) { #assert } }
            } else {
                assert
            }
        })
        .chain(spec.maintains.iter().map(|condition| {
            let expr = &condition.expr;
            let msg = format!("Pre-invariant failed: {}", expr.to_token_stream());
            let assert = quote! { assert!(#expr, #msg); };
            if let Some(cfg) = &condition.cfg {
                quote! { if cfg!(#cfg) { #assert } }
            } else {
                assert
            }
        }));

    // --- Generate Postcondition Checks ---
    let postconditions = spec
        .maintains
        .iter()
        .map(|condition| {
            let expr = &condition.expr;
            let msg = format!("Post-invariant failed: {}", expr.to_token_stream());
            let assert = quote! { assert!(#expr, #msg); };
            if let Some(cfg) = &condition.cfg {
                quote! { if cfg!(#cfg) { #assert } }
            } else {
                assert
            }
        })
        .chain(spec.ensures.iter().map(|condition_closure| {
            let closure = &condition_closure.closure;
            let msg = format!("Postcondition failed: {}", closure.to_token_stream());
            let assert = quote! { assert!((#closure)(#binding_ident), #msg); };
            if let Some(cfg) = &condition_closure.cfg {
                quote! { if cfg!(#cfg) { #assert } }
            } else {
                assert
            }
        }));

    // --- Construct the New Body ---
    let body_expr = if is_async {
        quote! { async #original_body.await }
    } else {
        quote! { #original_body }
    };

    Ok(parse_quote! {
        {
            #(#preconditions)*
            let #binding_ident = #body_expr;
            #(#postconditions)*
            #binding_ident
        }
    })
}

#[cfg(test)]
mod test_parse_spec;

#[cfg(test)]
mod test_instrument_fn;

#[cfg(test)]
mod test_util;
