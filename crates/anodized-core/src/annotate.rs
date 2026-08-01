use syn::{
    Attribute, Error, Expr, ExprLet, Meta, Pat, Stmt,
    parse::{Parse, ParseStream, Result},
    spanned::Spanned,
};

use crate::{
    Capture, Condition, DataSpec, LoopSpec, LoopVariant, Spec,
    annotate::syntax::{SpecArg, SpecArgValue},
    qualifiers::FnQualifiers,
};

pub mod syntax;
use syntax::Keyword;

#[cfg(test)]
#[path = "annotate_tests.rs"]
mod annotate_tests;

impl Parse for Spec {
    fn parse(input: ParseStream) -> Result<Self> {
        let raw_spec = syntax::SpecArgs::parse(input)?;

        let mut errors = MultiError::empty();
        let mut qualifiers = FnQualifiers::empty();
        let mut requires: Vec<Condition> = vec![];
        let mut maintains: Vec<Condition> = vec![];
        let mut captures: Vec<Capture> = vec![];
        let mut inspects: Option<Pat> = None;
        let mut ensures: Vec<Condition> = vec![];

        let is_sorted = raw_spec.is_sorted();

        for arg in raw_spec.args {
            match arg.keyword {
                Keyword::Unknown(ident) => {
                    errors.add(Error::new(
                        arg.keyword_span,
                        format!("unknown spec keyword `{ident}`"),
                    ));
                }
                Keyword::Functional => {
                    if let Err(error) =
                        arg.parse_fn_qualifier(FnQualifiers::FUNCTIONAL, &mut qualifiers)
                    {
                        errors.add(error);
                    }
                }
                Keyword::Pure => {
                    if let Err(error) = arg.parse_fn_qualifier(FnQualifiers::PURE, &mut qualifiers)
                    {
                        errors.add(error);
                    }
                }
                Keyword::Total => {
                    if let Err(error) = arg.parse_fn_qualifier(FnQualifiers::TOTAL, &mut qualifiers)
                    {
                        errors.add(error);
                    }
                }
                Keyword::Deterministic => {
                    if let Err(error) =
                        arg.parse_fn_qualifier(FnQualifiers::DETERMINISTIC, &mut qualifiers)
                    {
                        errors.add(error);
                    }
                }
                Keyword::Effectfree => {
                    if let Err(error) =
                        arg.parse_fn_qualifier(FnQualifiers::EFFECTFREE, &mut qualifiers)
                    {
                        errors.add(error);
                    }
                }
                Keyword::Infallible => {
                    if let Err(error) =
                        arg.parse_fn_qualifier(FnQualifiers::INFALLIBLE, &mut qualifiers)
                    {
                        errors.add(error);
                    }
                }
                Keyword::Terminating => {
                    if let Err(error) =
                        arg.parse_fn_qualifier(FnQualifiers::TERMINATING, &mut qualifiers)
                    {
                        errors.add(error);
                    }
                }
                Keyword::Requires => {
                    if let Err(error) = arg.parse_conditions(&mut requires) {
                        errors.add(error);
                    }
                }
                Keyword::Maintains => {
                    if let Err(error) = arg.parse_conditions(&mut maintains) {
                        errors.add(error);
                    }
                }
                Keyword::Captures => {
                    if !captures.is_empty() {
                        errors.add(Error::new(
                            arg.keyword_span,
                            "at most one `captures` parameter is allowed; to capture multiple values, use a list: `captures: [expr1, expr2, ...]`",
                        ));
                    }
                    if let Err(error) = arg.parse_captures(&mut captures) {
                        errors.add(error);
                    }
                }
                Keyword::Binds => {
                    errors.add(Error::new(
                        arg.keyword_span,
                        "the `binds` parameter was renamed to `inspects`",
                    ));
                }
                Keyword::Inspects => {
                    if inspects.is_some() {
                        errors.add(Error::new(
                            arg.keyword_span,
                            "multiple `inspects` parameters are not allowed",
                        ));
                    }
                    if let Err(error) = arg.parse_inspects(&mut inspects) {
                        errors.add(error);
                    }
                }
                Keyword::Ensures => {
                    if let Err(error) = arg.parse_conditions(&mut ensures) {
                        errors.add(error);
                    }
                }
                Keyword::Decreases => {
                    errors.add(Error::new(
                        arg.keyword_span,
                        format!("`{}` parameter is not supported here", arg.keyword),
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
            inspects,
            ensures,
            span: input.span(),
        })
    }
}

impl Parse for DataSpec {
    fn parse(input: ParseStream) -> Result<Self> {
        let raw_spec = syntax::SpecArgs::parse(input)?;

        let mut errors = MultiError::empty();
        let mut maintains: Vec<Condition> = vec![];

        for arg in raw_spec.args {
            match arg.keyword {
                Keyword::Unknown(ident) => {
                    errors.add(Error::new(
                        arg.keyword_span,
                        format!("unknown spec keyword `{ident}`"),
                    ));
                }
                Keyword::Maintains => {
                    if let Err(error) = arg.parse_conditions(&mut maintains) {
                        errors.add(error);
                    }
                }
                _ => {
                    errors.add(Error::new(
                        arg.keyword_span,
                        format!("`{}` parameter is not supported here", arg.keyword),
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
        let raw_spec = syntax::SpecArgs::parse(input)?;

        let is_sorted = raw_spec.is_sorted();

        let mut errors = MultiError::empty();
        let mut decreases = None;
        let mut maintains: Vec<Condition> = vec![];

        for arg in raw_spec.args {
            match arg.keyword {
                Keyword::Unknown(ident) => {
                    errors.add(Error::new(
                        arg.keyword_span,
                        format!("unknown spec keyword `{ident}`"),
                    ));
                }
                Keyword::Maintains => {
                    if let Err(error) = arg.parse_conditions(&mut maintains) {
                        errors.add(error);
                    }
                }
                Keyword::Decreases => {
                    if decreases.is_some() {
                        errors.add(Error::new(
                            arg.keyword_span,
                            "multiple `decreases` parameters are not allowed",
                        ));
                    }
                    if let Err(error) = arg.parse_decreases(&mut decreases) {
                        errors.add(error);
                    }
                }
                _ => {
                    errors.add(Error::new(
                        arg.keyword_span,
                        format!("`{}` parameter is not supported here", arg.keyword),
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

impl SpecArg {
    fn parse_fn_qualifier(self, value: FnQualifiers, qualifiers: &mut FnQualifiers) -> Result<()> {
        if let Some(first_attr) = self.attrs.first() {
            return Err(Error::new_spanned(
                first_attr,
                format!("attributes are not supported on `{}`", self.keyword),
            ));
        }
        if !matches!(self.value, SpecArgValue::None) {
            return Err(Error::new_spanned(
                self.value,
                format!("qualifier `{}` does not take a value", self.keyword),
            ));
        }
        if qualifiers.contains(value) {
            return Err(Error::new(
                self.keyword_span,
                "this qualifier is redundant; remove it",
            ));
        }
        *qualifiers |= value;
        Ok(())
    }

    fn parse_conditions(self, conditions: &mut Vec<Condition>) -> Result<()> {
        let cfg_attr = find_cfg_attribute(&self.attrs)?;
        let cfg: Option<Meta> = if let Some(attr) = cfg_attr {
            Some(attr.parse_args()?)
        } else {
            None
        };
        let expr = self.value.try_into_expr()?;
        if let Expr::Array(items) = expr {
            for expr in items.elems {
                conditions.push(Condition {
                    expr,
                    cfg: cfg.clone(),
                });
            }
        } else {
            conditions.push(Condition { expr, cfg });
        }
        Ok(())
    }

    fn parse_captures(self, captures: &mut Vec<Capture>) -> Result<()> {
        let cfg_attr = find_cfg_attribute(&self.attrs)?;
        if cfg_attr.is_some() {
            return Err(Error::new(
                cfg_attr.span(),
                "`cfg` attribute is not supported on `captures`",
            ));
        }
        let capture_list = self.value.try_into_expr()?;
        match capture_list {
            Expr::Let(let_expr) => {
                captures.push(interpret_let_expr_as_capture(let_expr)?);
            }
            Expr::Block(block) => {
                for stmt in block.block.stmts {
                    let Stmt::Local(local) = stmt else {
                        return Err(Error::new_spanned(stmt, "expected a `let` binding"));
                    };
                    let Some(init) = local.init else {
                        return Err(Error::new_spanned(local, "expected a `let` binding"));
                    };
                    captures.push(Capture {
                        pat: local.pat,
                        expr: *init.expr,
                    });
                }
            }
            _ => {
                return Err(Error::new_spanned(
                    capture_list,
                    "expected a `let` binding or block",
                ));
            }
        }
        Ok(())
    }

    fn parse_inspects(self, pattern: &mut Option<Pat>) -> Result<()> {
        let cfg_attr = find_cfg_attribute(&self.attrs)?;
        if cfg_attr.is_some() {
            return Err(Error::new(
                cfg_attr.span(),
                "`cfg` attribute is not supported on `inspects`",
            ));
        }
        let inspects_pattern = self.value.try_into_pat()?;
        *pattern = Some(inspects_pattern);
        Ok(())
    }

    fn parse_decreases(self, decreases: &mut Option<LoopVariant>) -> Result<()> {
        let cfg_attr = find_cfg_attribute(&self.attrs)?;
        let cfg: Option<Meta> = if let Some(attr) = cfg_attr {
            Some(attr.parse_args()?)
        } else {
            None
        };
        let expr_span = self.value.span();
        let expr = self.value.try_into_expr()?;
        if let Expr::Array(_) = expr {
            return Err(Error::new(expr_span, "expected a single expression"));
        } else {
            *decreases = Some(LoopVariant { expr, cfg });
        }
        Ok(())
    }
}

/// Try to interpret an ExprLet as a single Capture
fn interpret_let_expr_as_capture(let_expr: ExprLet) -> Result<Capture> {
    // TODO: Disallow `mut`, `ref`, etc.
    Ok(Capture {
        pat: *let_expr.pat,
        expr: *let_expr.expr,
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
