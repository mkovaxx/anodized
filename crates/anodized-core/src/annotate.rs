use quote::ToTokens;
use syn::{
    Attribute, Error, Expr, ExprAssign, FieldValue, Meta,
    parse::{Parse, ParseStream, Result},
    parse_quote,
    spanned::Spanned,
};

use crate::{
    Capture, Condition, DataSpec, LoopSpec, LoopVariant, PostCondition, Spec,
    qualifiers::FnQualifiers,
};

pub mod syntax;
use syntax::Keyword;

#[cfg(test)]
#[path = "annotate_tests.rs"]
mod annotate_tests;

impl Parse for Spec {
    fn parse(input: ParseStream) -> Result<Self> {
        let raw_spec = syntax::SpecFields::parse(input)?;

        let mut errors = MultiError::empty();
        let mut qualifiers = FnQualifiers::empty();
        let mut requires: Vec<Condition> = vec![];
        let mut maintains: Vec<Condition> = vec![];
        let mut captures: Vec<Capture> = vec![];
        let mut ensures: Vec<PostCondition> = vec![];

        let is_sorted = raw_spec.is_sorted();

        for field in raw_spec.fields {
            let keyword = Keyword::from(&field.member);
            match keyword {
                Keyword::Unknown(ident) => {
                    errors.add(Error::new_spanned(
                        &field.member,
                        format!("unknown spec keyword `{ident}`"),
                    ));
                }
                Keyword::Functional => {
                    if let Err(error) =
                        parse_fn_qualifier(field, FnQualifiers::FUNCTIONAL, &mut qualifiers)
                    {
                        errors.add(error);
                    }
                }
                Keyword::Pure => {
                    if let Err(error) =
                        parse_fn_qualifier(field, FnQualifiers::PURE, &mut qualifiers)
                    {
                        errors.add(error);
                    }
                }
                Keyword::Total => {
                    if let Err(error) =
                        parse_fn_qualifier(field, FnQualifiers::TOTAL, &mut qualifiers)
                    {
                        errors.add(error);
                    }
                }
                Keyword::Deterministic => {
                    if let Err(error) =
                        parse_fn_qualifier(field, FnQualifiers::DETERMINISTIC, &mut qualifiers)
                    {
                        errors.add(error);
                    }
                }
                Keyword::Effectfree => {
                    if let Err(error) =
                        parse_fn_qualifier(field, FnQualifiers::EFFECTFREE, &mut qualifiers)
                    {
                        errors.add(error);
                    }
                }
                Keyword::Infallible => {
                    if let Err(error) =
                        parse_fn_qualifier(field, FnQualifiers::INFALLIBLE, &mut qualifiers)
                    {
                        errors.add(error);
                    }
                }
                Keyword::Terminating => {
                    if let Err(error) =
                        parse_fn_qualifier(field, FnQualifiers::TERMINATING, &mut qualifiers)
                    {
                        errors.add(error);
                    }
                }
                Keyword::Requires => {
                    if let Err(error) = parse_conditions(field, &mut requires) {
                        errors.add(error);
                    }
                }
                Keyword::Maintains => {
                    if let Err(error) = parse_conditions(field, &mut maintains) {
                        errors.add(error);
                    }
                }
                Keyword::Captures => {
                    if !captures.is_empty() {
                        errors.add(Error::new_spanned(
                            &field.member,
                            "at most one `captures` parameter is allowed; to capture multiple values, use a list: `captures: [binding1 = expr1, binding2 = expr2, ...]`",
                        ));
                    }
                    if let Err(error) = parse_captures(field, &mut captures) {
                        errors.add(error);
                    }
                }
                Keyword::Binds | Keyword::Inspects => {
                    errors.add(Error::new_spanned(
                        &field.member,
                        "no longer supported, use the following form instead: `ensures: |PAT| [EXPR, EXPR, ...]`",
                    ));
                }
                Keyword::Ensures => {
                    if let Err(error) = parse_postconds(field, &mut ensures) {
                        errors.add(error);
                    }
                }
                Keyword::Decreases => {
                    errors.add(Error::new_spanned(
                        &field.member,
                        format!(
                            "`{}` parameter is not supported here",
                            field.member.to_token_stream(),
                        ),
                    ));
                }
            }
        }

        if !is_sorted {
            errors.add(Error::new(
                input.span(),
                "parameters are out of order: the expected order is: `<QUALIFIERS>`, `requires`, `maintains`, `captures`, `inspects`, `ensures`, where `<QUALIFIERS>` are:\n
`functional` (`pure` and `total`),\n
`pure` (`deterministic` and `effectfree`),\n
`total` (`infallible` and `terminating`)",
            ));
        }

        if let Some(combined_error) = errors.get_combined() {
            return Err(combined_error);
        }

        Ok(Self {
            qualifiers,
            requires,
            maintains,
            captures,
            ensures,
            span: input.span(),
        })
    }
}

impl Parse for DataSpec {
    fn parse(input: ParseStream) -> Result<Self> {
        let raw_spec = syntax::SpecFields::parse(input)?;

        let mut errors = MultiError::empty();
        let mut maintains: Vec<Condition> = vec![];

        for field in raw_spec.fields {
            let keyword = Keyword::from(&field.member);
            match keyword {
                Keyword::Unknown(ident) => {
                    errors.add(Error::new_spanned(
                        &field.member,
                        format!("unknown spec keyword `{ident}`"),
                    ));
                }
                Keyword::Maintains => {
                    if let Err(error) = parse_conditions(field, &mut maintains) {
                        errors.add(error);
                    }
                }
                _ => {
                    errors.add(Error::new_spanned(
                        &field.member,
                        format!(
                            "`{}` parameter is not supported here",
                            field.member.to_token_stream(),
                        ),
                    ));
                }
            }
        }

        if let Some(combined_error) = errors.get_combined() {
            return Err(combined_error);
        }

        Ok(Self {
            maintains,
            span: input.span(),
        })
    }
}

impl Parse for LoopSpec {
    fn parse(input: ParseStream) -> Result<Self> {
        let raw_spec = syntax::SpecFields::parse(input)?;

        let is_sorted = raw_spec.is_sorted();

        let mut errors = MultiError::empty();
        let mut decreases = None;
        let mut maintains: Vec<Condition> = vec![];

        for field in raw_spec.fields {
            let keyword = Keyword::from(&field.member);
            match keyword {
                Keyword::Unknown(ident) => {
                    errors.add(Error::new_spanned(
                        &field.member,
                        format!("unknown spec keyword `{ident}`"),
                    ));
                }
                Keyword::Maintains => {
                    if let Err(error) = parse_conditions(field, &mut maintains) {
                        errors.add(error);
                    }
                }
                Keyword::Decreases => {
                    if decreases.is_some() {
                        errors.add(Error::new_spanned(
                            &field.member,
                            "multiple `decreases` parameters are not allowed",
                        ));
                    }
                    if let Err(error) = parse_decreases(field, &mut decreases) {
                        errors.add(error);
                    }
                }
                _ => {
                    errors.add(Error::new_spanned(
                        &field.member,
                        format!(
                            "`{}` parameter is not supported here",
                            field.member.to_token_stream(),
                        ),
                    ));
                }
            }
        }

        if !is_sorted {
            errors.add(Error::new(
                input.span(),
                "parameters are out of order: the expected order is `maintains`, `decreases`",
            ));
        }

        if let Some(combined_error) = errors.get_combined() {
            return Err(combined_error);
        }

        Ok(Self {
            maintains,
            decreases,
            span: input.span(),
        })
    }
}

fn parse_fn_qualifier(
    field: FieldValue,
    value: FnQualifiers,
    qualifiers: &mut FnQualifiers,
) -> Result<()> {
    if let Some(first_attr) = field.attrs.first() {
        return Err(Error::new_spanned(
            first_attr,
            format!(
                "attributes are not supported on `{}`",
                field.member.to_token_stream()
            ),
        ));
    }
    if field.colon_token.is_some() {
        return Err(Error::new_spanned(
            &field.member,
            format!(
                "qualifier `{}` does not take a value",
                field.member.to_token_stream()
            ),
        ));
    }
    if qualifiers.contains(value) {
        return Err(Error::new_spanned(
            &field.member,
            "this qualifier is redundant; remove it",
        ));
    }
    *qualifiers |= value;
    Ok(())
}

fn parse_conditions(field: FieldValue, conditions: &mut Vec<Condition>) -> Result<()> {
    let cfg_attr = find_cfg_attribute(&field.attrs)?;
    let cfg: Option<Meta> = if let Some(attr) = cfg_attr {
        Some(attr.parse_args()?)
    } else {
        None
    };
    if field.colon_token.is_none() {
        return Err(Error::new_spanned(field.expr, "expected an expression"));
    };
    if let Expr::Array(items) = field.expr {
        for expr in items.elems {
            conditions.push(Condition {
                expr,
                cfg: cfg.clone(),
            });
        }
    } else {
        conditions.push(Condition {
            expr: field.expr,
            cfg,
        });
    }
    Ok(())
}

fn parse_postconds(field: FieldValue, postconds: &mut Vec<PostCondition>) -> Result<()> {
    let cfg_attr = find_cfg_attribute(&field.attrs)?;
    let cfg: Option<Meta> = if let Some(attr) = cfg_attr {
        Some(attr.parse_args()?)
    } else {
        None
    };
    if field.colon_token.is_none() {
        return Err(Error::new_spanned(&field.expr, "expected an expression"));
    };
    match field.expr {
        Expr::Closure(mut closure) => {
            if closure.inputs.len() != 1 {
                return Err(Error::new_spanned(
                    closure,
                    "postcondition closure must have exactly one input",
                ));
            }
            let (pat, _) = closure.inputs.pop().unwrap().into_tuple();
            if let Expr::Array(array) = *closure.body {
                for expr in array.elems {
                    postconds.push(PostCondition {
                        pat: Some(pat.clone()),
                        expr,
                        cfg: cfg.clone(),
                    });
                }
            } else {
                postconds.push(PostCondition {
                    pat: Some(pat),
                    expr: *closure.body,
                    cfg: cfg.clone(),
                });
            }
        }
        Expr::Array(array) => {
            for expr in array.elems {
                if let Expr::Closure(mut closure) = expr {
                    if closure.inputs.len() != 1 {
                        return Err(Error::new_spanned(
                            closure,
                            "postcondition closure must have exactly one input",
                        ));
                    }
                    let (pat, _) = closure.inputs.pop().unwrap().into_tuple();
                    postconds.push(PostCondition {
                        pat: Some(pat),
                        expr: *closure.body,
                        cfg: cfg.clone(),
                    });
                } else {
                    postconds.push(PostCondition {
                        pat: None,
                        expr,
                        cfg: cfg.clone(),
                    });
                }
            }
        }
        _ => {
            postconds.push(PostCondition {
                pat: None,
                expr: field.expr,
                cfg: cfg.clone(),
            });
        }
    }
    Ok(())
}

fn parse_captures(field: FieldValue, captures: &mut Vec<Capture>) -> Result<()> {
    let cfg_attr = find_cfg_attribute(&field.attrs)?;
    if cfg_attr.is_some() {
        return Err(Error::new(
            cfg_attr.span(),
            "`cfg` attribute is not supported on `captures`",
        ));
    }
    if field.colon_token.is_none() {
        return Err(Error::new_spanned(field.expr, "expected an expression"));
    };
    match field.expr {
        Expr::Assign(assignment) => {
            captures.push(interpret_assignment_as_capture(assignment)?);
        }
        Expr::Array(array) => {
            for elem in array.elems {
                let Expr::Assign(assignment) = elem else {
                    return Err(Error::new_spanned(elem, "expected an assignment"));
                };
                captures.push(interpret_assignment_as_capture(assignment)?);
            }
        }
        _ => {
            return Err(Error::new_spanned(
                field.expr,
                "expected an assignment or block",
            ));
        }
    }
    Ok(())
}

fn parse_decreases(field: FieldValue, decreases: &mut Option<LoopVariant>) -> Result<()> {
    let cfg_attr = find_cfg_attribute(&field.attrs)?;
    let cfg: Option<Meta> = if let Some(attr) = cfg_attr {
        Some(attr.parse_args()?)
    } else {
        None
    };
    if field.colon_token.is_none() {
        return Err(Error::new_spanned(field.expr, "expected an expression"));
    };
    if let Expr::Array(_) = field.expr {
        return Err(Error::new_spanned(
            field.expr,
            "expected a single expression",
        ));
    } else {
        *decreases = Some(LoopVariant {
            expr: field.expr,
            cfg,
        });
    }
    Ok(())
}

/// Try to interpret an ExprAssign as a single Capture.
fn interpret_assignment_as_capture(assignment: ExprAssign) -> Result<Capture> {
    let left = assignment.left;
    Ok(Capture {
        // TODO: Make this less janky.
        pat: parse_quote! { #left },
        expr: *assignment.right,
    })
}

fn find_cfg_attribute(attrs: &[Attribute]) -> Result<Option<&Attribute>> {
    let mut cfg_attr: Option<&Attribute> = None;

    for attr in attrs {
        if attr.path().is_ident("cfg") {
            if cfg_attr.is_some() {
                return Err(Error::new(
                    attr.span(),
                    "multiple `cfg` attributes are not supported",
                ));
            }
            cfg_attr = Some(attr);
        } else {
            return Err(Error::new(
                attr.span(),
                "unsupported attribute; only `cfg` is allowed",
            ));
        }
    }

    Ok(cfg_attr)
}

struct MultiError(Option<Error>);

impl MultiError {
    fn empty() -> Self {
        Self(None)
    }

    fn get_combined(self) -> Option<Error> {
        self.0
    }

    fn add(&mut self, error: Error) {
        match &mut self.0 {
            Some(acc) => acc.combine(error),
            None => self.0 = Some(error),
        }
    }
}
