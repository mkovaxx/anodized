use syn::{
    Attribute, Expr, Ident, Meta, Pat,
    parse::{Parse, ParseStream, Result},
    parse_quote,
    spanned::Spanned,
};

use crate::{Capture, PostCondition, PreCondition, Spec};

pub mod syntax;
use syntax::Keyword;

#[cfg(test)]
mod tests;

impl Parse for Spec {
    fn parse(input: ParseStream) -> Result<Self> {
        let raw = syntax::SpecArgs::parse(input)?;
        let args = raw.args;

        let mut last_arg_order: Option<Keyword> = None;
        let mut requires: Vec<PreCondition> = vec![];
        let mut maintains: Vec<PreCondition> = vec![];
        let mut captures: Vec<Capture> = vec![];
        let mut binds_pattern: Option<Pat> = None;
        let mut ensures: Vec<PostCondition> = vec![];

        for arg in args {
            let current_arg_order = arg.get_order().clone();
            if let Some(last_order) = last_arg_order {
                if current_arg_order < last_order {
                    return Err(syn::Error::new(
                        arg.get_keyword_span(),
                        "parameters are out of order: their order must be `requires`, `maintains`, `captures`, `binds`, `ensures`",
                    ));
                }
            }
            last_arg_order = Some(current_arg_order);

            match arg.keyword {
                Keyword::Requires => {
                    let cfg_attr = find_cfg_attribute(&arg.attrs)?;
                    let cfg: Option<Meta> = if let Some(attr) = cfg_attr {
                        Some(attr.parse_args()?)
                    } else {
                        None
                    };
                    let expr = arg.value.try_into_expr()?;
                    if let Expr::Array(conditions) = expr {
                        for expr in conditions.elems {
                            requires.push(PreCondition {
                                closure: interpret_expr_as_precondition(expr)?,
                                cfg: cfg.clone(),
                            });
                        }
                    } else {
                        requires.push(PreCondition {
                            closure: interpret_expr_as_precondition(expr)?,
                            cfg,
                        });
                    }
                }
                Keyword::Maintains => {
                    let cfg_attr = find_cfg_attribute(&arg.attrs)?;
                    let cfg: Option<Meta> = if let Some(attr) = cfg_attr {
                        Some(attr.parse_args()?)
                    } else {
                        None
                    };
                    let expr = arg.value.try_into_expr()?;
                    if let Expr::Array(conditions) = expr {
                        for expr in conditions.elems {
                            maintains.push(PreCondition {
                                closure: interpret_expr_as_precondition(expr)?,
                                cfg: cfg.clone(),
                            });
                        }
                    } else {
                        maintains.push(PreCondition {
                            closure: interpret_expr_as_precondition(expr)?,
                            cfg,
                        });
                    }
                }
                Keyword::Captures => {
                    let cfg_attr = find_cfg_attribute(&arg.attrs)?;
                    if cfg_attr.is_some() {
                        return Err(syn::Error::new(
                            cfg_attr.span(),
                            "`cfg` attribute is not supported on `captures`",
                        ));
                    }
                    if !captures.is_empty() {
                        return Err(syn::Error::new(
                            arg.keyword_span,
                            "at most one `captures` parameter is allowed; to capture multiple values, use a list: `captures: [expr1, expr2, ...]`",
                        ));
                    }
                    let expr = arg.value.try_into_expr()?;
                    if let Expr::Array(array) = expr {
                        captures.extend(interpret_array_as_captures(array)?);
                    } else {
                        captures.push(interpret_expr_as_capture(expr)?);
                    }
                }
                Keyword::Binds => {
                    let cfg_attr = find_cfg_attribute(&arg.attrs)?;
                    if cfg_attr.is_some() {
                        return Err(syn::Error::new(
                            cfg_attr.span(),
                            "`cfg` attribute is not supported on `binds`",
                        ));
                    }
                    if binds_pattern.is_some() {
                        return Err(syn::Error::new(
                            arg.keyword_span,
                            "multiple `binds` parameters are not allowed",
                        ));
                    }
                    let pattern = arg.value.try_into_pat()?;
                    binds_pattern = Some(pattern);
                }
                Keyword::Ensures => {
                    let cfg_attr = find_cfg_attribute(&arg.attrs)?;
                    let cfg: Option<Meta> = if let Some(attr) = cfg_attr {
                        Some(attr.parse_args()?)
                    } else {
                        None
                    };
                    let expr = arg.value.try_into_expr()?;
                    let default_pattern = binds_pattern.clone().unwrap_or(parse_quote! { output });
                    if let Expr::Array(conditions) = expr {
                        for expr in conditions.elems {
                            ensures.push(PostCondition {
                                closure: interpret_expr_as_postcondition(
                                    expr,
                                    default_pattern.clone(),
                                )?,
                                cfg: cfg.clone(),
                            });
                        }
                    } else {
                        ensures.push(PostCondition {
                            closure: interpret_expr_as_postcondition(expr, default_pattern)?,
                            cfg,
                        });
                    }
                }
                Keyword::Unknown(ident) => {
                    return Err(syn::Error::new(
                        arg.keyword_span,
                        format!("unknown spec keyword `{ident}`"),
                    ));
                }
            }
        }

        Ok(Spec {
            requires,
            maintains,
            captures,
            ensures,
            span: input.span(),
        })
    }
}

/// Try to interpret an Expr::Array as a list of Captures
fn interpret_array_as_captures(array: syn::ExprArray) -> Result<Vec<Capture>> {
    let mut bindings = Vec::new();

    for elem in array.elems {
        // Try to interpret each element as a capture
        // If any fails, propagate that error immediately
        bindings.push(interpret_expr_as_capture(elem)?);
    }

    Ok(bindings)
}

/// Try to interpret an Expr as a single Capture
fn interpret_expr_as_capture(expr: Expr) -> Result<Capture> {
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
            Ok(Capture { expr, alias })
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
                    return Ok(Capture {
                        expr: *cast.expr,
                        alias,
                    });
                }
            }
            Err(syn::Error::new_spanned(
                cast,
                "alias must be a simple identifier",
            ))
        }
        // Any other expression requires an explicit alias
        _ => Err(syn::Error::new_spanned(
            expr,
            "complex expressions require an explicit alias using `as`",
        )),
    }
}

/// Interpret expression as a zero-parameter closure, wrapping if necessary.
/// Used for preconditions which don't need access to the return value.
fn interpret_expr_as_precondition(expr: Expr) -> Result<syn::ExprClosure> {
    match expr {
        // Already a closure, validate it has no arguments.
        Expr::Closure(closure) => {
            if closure.inputs.is_empty() {
                Ok(closure)
            } else {
                Err(syn::Error::new_spanned(
                    closure.or1_token,
                    format!(
                        "precondition closure must have no arguments, found {}",
                        closure.inputs.len()
                    ),
                ))
            }
        }
        // Naked expression, wrap in an argumentless closure.
        expr => Ok(syn::ExprClosure {
            attrs: vec![],
            lifetimes: None,
            constness: None,
            movability: None,
            asyncness: None,
            capture: None,
            or1_token: Default::default(),
            inputs: syn::punctuated::Punctuated::new(),
            or2_token: Default::default(),
            output: syn::ReturnType::Default,
            body: Box::new(expr),
        }),
    }
}

/// Interpret expression as a closure with a single argument (eg the list of
/// aliases and function result), wrapping if necessary.
/// Used for postconditions which take the return value as an argument.
fn interpret_expr_as_postcondition(expr: Expr, default_binding: Pat) -> Result<syn::ExprClosure> {
    match expr {
        // Already a closure, validate it has exactly one argument.
        Expr::Closure(closure) => {
            if closure.inputs.len() == 1 {
                Ok(closure)
            } else {
                Err(syn::Error::new_spanned(
                    &closure.or1_token,
                    format!(
                        "postcondition closure must have exactly one argument, found {}",
                        closure.inputs.len()
                    ),
                ))
            }
        }
        // Naked expression, wrap in a closure with default binding.
        expr => Ok(syn::ExprClosure {
            attrs: vec![],
            lifetimes: None,
            constness: None,
            movability: None,
            asyncness: None,
            capture: None,
            or1_token: Default::default(),
            inputs: syn::punctuated::Punctuated::from_iter([default_binding]),
            or2_token: Default::default(),
            output: syn::ReturnType::Default,
            body: Box::new(expr),
        }),
    }
}

pub fn find_cfg_attribute(attrs: &[Attribute]) -> Result<Option<&Attribute>> {
    let mut cfg_attr: Option<&Attribute> = None;

    for attr in attrs {
        if attr.path().is_ident("cfg") {
            if cfg_attr.is_some() {
                return Err(syn::Error::new(
                    attr.span(),
                    "multiple `cfg` attributes are not supported",
                ));
            }
            cfg_attr = Some(attr);
        } else {
            return Err(syn::Error::new(
                attr.span(),
                "unsupported attribute; only `cfg` is allowed",
            ));
        }
    }

    Ok(cfg_attr)
}
