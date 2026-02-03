use syn::{
    Attribute, Expr, Ident, Meta, Pat, PatIdent,
    parse::{Parse, ParseStream, Result},
    parse_quote,
    spanned::Spanned,
};

use crate::{Capture, PostCondition, PreCondition, Spec, annotate::syntax::CaptureExpr};

pub mod syntax;
use syntax::{CaptureList, Keyword};

#[cfg(test)]
mod tests;

impl Parse for Spec {
    fn parse(input: ParseStream) -> Result<Self> {
        let raw_spec = syntax::SpecArgs::parse(input)?;

        let mut prev_keyword: Option<Keyword> = None;
        let mut requires: Vec<PreCondition> = vec![];
        let mut maintains: Vec<PreCondition> = vec![];
        let mut captures: Vec<Capture> = vec![];
        let mut binds_pattern: Option<Pat> = None;
        let mut ensures: Vec<PostCondition> = vec![];

        for arg in raw_spec.args {
            match &arg.keyword {
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
                    let capture_list = arg.value.try_into_captures()?;
                    match capture_list {
                        CaptureList::Single(capture_expr) => {
                            captures.push(interpret_capture_expr_as_capture(capture_expr)?);
                        }
                        CaptureList::Array { elems, .. } => {
                            for capture_expr in elems {
                                captures.push(interpret_capture_expr_as_capture(capture_expr)?);
                            }
                        }
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

            if let Some(prev_keyword) = prev_keyword {
                if arg.keyword < prev_keyword {
                    return Err(syn::Error::new(
                        arg.keyword_span,
                        "parameters are out of order: their order must be `requires`, `maintains`, `captures`, `binds`, `ensures`",
                    ));
                }
            }
            prev_keyword = Some(arg.keyword);
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

/// Try to interpret an Expr as a single Capture
fn interpret_capture_expr_as_capture(capture_expr: CaptureExpr) -> Result<Capture> {
    match capture_expr {
        // Simple identifier: count -> old_count
        CaptureExpr::Ident(ref path)
            if path.path.segments.len() == 1
                && path.path.leading_colon.is_none()
                && path.attrs.is_empty()
                && path.qself.is_none() =>
        {
            let ident = &path.path.segments[0].ident;
            let ident_alias = Ident::new(&format!("old_{}", ident), ident.span());
            let pattern_alias = Pat::Ident(PatIdent {
                ident: ident_alias,
                attrs: vec![],
                mutability: None,
                by_ref: None,
                subpat: None,
            });
            Ok(Capture {
                expr: Expr::Path(path.clone()),
                pat: pattern_alias,
            })
        }
        // Cast expression: value as old_value
        CaptureExpr::Cast(cast) => {
            match *cast.ty {
                syn::Type::Path(ref type_path)
                    if type_path.path.segments.len() == 1
                        && type_path.path.leading_colon.is_none()
                        && type_path.qself.is_none() =>
                {
                    // The cast.ty is a simple identifier that we use as the alias
                    let ident = type_path.path.segments[0].ident.clone();
                    let pat = Pat::Ident(PatIdent {
                        attrs: vec![],
                        by_ref: None,
                        mutability: None,
                        ident,
                        subpat: None,
                    });
                    Ok(Capture {
                        expr: *cast.expr,
                        pat,
                    })
                }
                _ => Err(syn::Error::new_spanned(
                    cast.expr,
                    "Invalid pattern for alias",
                )),
            }
        }
        // Handle ident as pattern (stored internally as Pat::Ident with subpat)
        CaptureExpr::Pattern(Pat::Ident(pat_ident)) if pat_ident.subpat.is_some() => {
            let (_, subpat) = pat_ident.subpat.unwrap();
            let ident = pat_ident.ident;
            let expr = Expr::Path(syn::ExprPath {
                attrs: vec![],
                qself: None,
                path: ident.into(),
            });
            Ok(Capture { expr, pat: *subpat })
        }
        CaptureExpr::Pattern(pat) => Err(syn::Error::new_spanned(
            pat,
            "pattern captures must use `ident as [pattern]` syntax",
        )),
        // General expression with cast: `complex_expr as alias`
        CaptureExpr::Expr(Expr::Cast(cast)) => match *cast.ty {
            syn::Type::Path(ref type_path)
                if type_path.path.segments.len() == 1
                    && type_path.path.leading_colon.is_none()
                    && type_path.qself.is_none() =>
            {
                let ident = type_path.path.segments[0].ident.clone();
                let pat = Pat::Ident(PatIdent {
                    attrs: vec![],
                    by_ref: None,
                    mutability: None,
                    ident,
                    subpat: None,
                });
                Ok(Capture {
                    expr: *cast.expr,
                    pat,
                })
            }
            _ => Err(syn::Error::new_spanned(
                cast.ty,
                "alias must be a simple identifier",
            )),
        },
        CaptureExpr::Ident(path) => Err(syn::Error::new_spanned(
            path,
            "complex expressions require an explicit alias using `as`",
        )),
        // General expression without alias - error
        CaptureExpr::Expr(expr) => Err(syn::Error::new_spanned(
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
                    closure.or1_token,
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

fn find_cfg_attribute(attrs: &[Attribute]) -> Result<Option<&Attribute>> {
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
