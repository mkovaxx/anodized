use proc_macro2::TokenStream;
use syn::{
    Attribute, Error, Expr, ExprAssign, ExprForLoop, ExprWhile, FieldValue, Fields, FnArg,
    ImplItemFn, ItemEnum, ItemFn, ItemImpl, ItemStruct, ItemTrait, Meta, Token, TraitItemFn,
    parse::Result, parse_quote, punctuated::Punctuated, spanned::Spanned,
};

use crate::{
    Capture, Condition, DataSpec, EmptySpec, FnSpec, InputSpecFlags, LoopSpec, LoopVariant,
    PostCondition,
    qualifiers::FnQualifiers,
    syntax::{Keyword, SpecFields, UnspecArg, UnspecAttr, get_attr_input, remove_unique_attr},
};

#[cfg(test)]
#[path = "annotate_tests.rs"]
mod annotate_tests;

/// Implemented by AST nodes that may have a `#[spec]` attribute.
pub trait Specified {
    /// The spec type associated with this AST node.
    type Spec;

    /// Parse the `#[spec]` from the `Attribute` list.
    ///
    /// - If the AST node has a `#[spec]` attribute, it is removed from the `Attribute` list,
    ///   and its contents are parsed as `Self::Spec`, then attached to the AST node.
    /// - If there's no `#[spec]` attribute, the spec is built from an empty `SpecFields`.
    /// - Multiple `#[spec]` attributes are not allowed and cause an error.
    fn parse_spec_from_attrs(&mut self) -> Result<Self::Spec> {
        let spec_input: TokenStream =
            if let Some(attr) = remove_unique_attr("spec", self.get_attrs_mut())? {
                get_attr_input(attr)?
            } else {
                TokenStream::new()
            };
        let spec_fields: SpecFields = syn::parse2(spec_input)?;
        self.parse_spec_from_fields(spec_fields)
    }

    /// Parse the `#[spec]` from `SpecFields`.
    ///
    /// Implement this to parse a spec in the context of its AST node.
    fn parse_spec_from_fields(&mut self, fields: SpecFields) -> Result<Self::Spec>;

    /// Get a mutable reference to the `Attribute` list.
    ///
    /// Needed to by the default `with_spec_from_attrs`.
    fn get_attrs_mut(&mut self) -> &mut Vec<Attribute>;
}

impl Specified for ItemFn {
    type Spec = FnSpec;

    fn get_attrs_mut(&mut self) -> &mut Vec<Attribute> {
        &mut self.attrs
    }

    fn parse_spec_from_fields(&mut self, fields: SpecFields) -> Result<Self::Spec> {
        FnSpec::from_spec_and_inputs_attrs(fields, &mut self.sig.inputs, &mut self.attrs)
    }
}

impl Specified for ImplItemFn {
    type Spec = FnSpec;

    fn get_attrs_mut(&mut self) -> &mut Vec<Attribute> {
        &mut self.attrs
    }

    fn parse_spec_from_fields(&mut self, fields: SpecFields) -> Result<Self::Spec> {
        FnSpec::from_spec_and_inputs_attrs(fields, &mut self.sig.inputs, &mut self.attrs)
    }
}

impl Specified for TraitItemFn {
    type Spec = FnSpec;

    fn get_attrs_mut(&mut self) -> &mut Vec<Attribute> {
        &mut self.attrs
    }

    fn parse_spec_from_fields(&mut self, fields: SpecFields) -> Result<Self::Spec> {
        FnSpec::from_spec_and_inputs_attrs(fields, &mut self.sig.inputs, &mut self.attrs)
    }
}

impl Specified for ItemImpl {
    type Spec = EmptySpec;

    fn get_attrs_mut(&mut self) -> &mut Vec<Attribute> {
        &mut self.attrs
    }

    fn parse_spec_from_fields(&mut self, fields: SpecFields) -> Result<Self::Spec> {
        fields.try_into()
    }
}

impl Specified for ItemTrait {
    type Spec = EmptySpec;

    fn get_attrs_mut(&mut self) -> &mut Vec<Attribute> {
        &mut self.attrs
    }

    fn parse_spec_from_fields(&mut self, fields: SpecFields) -> Result<Self::Spec> {
        fields.try_into()
    }
}

impl Specified for ItemStruct {
    type Spec = DataSpec;

    fn get_attrs_mut(&mut self) -> &mut Vec<Attribute> {
        &mut self.attrs
    }

    fn parse_spec_from_fields(&mut self, fields: SpecFields) -> Result<Self::Spec> {
        let variants = std::iter::once(&mut self.fields);
        DataSpec::from_spec_and_variants(fields, variants)
    }
}

impl Specified for ItemEnum {
    type Spec = DataSpec;

    fn get_attrs_mut(&mut self) -> &mut Vec<Attribute> {
        &mut self.attrs
    }

    fn parse_spec_from_fields(&mut self, fields: SpecFields) -> Result<Self::Spec> {
        let variants = self.variants.iter_mut().map(|variant| &mut variant.fields);
        DataSpec::from_spec_and_variants(fields, variants)
    }
}

impl Specified for ExprForLoop {
    type Spec = LoopSpec;

    fn get_attrs_mut(&mut self) -> &mut Vec<Attribute> {
        &mut self.attrs
    }

    fn parse_spec_from_fields(&mut self, fields: SpecFields) -> Result<Self::Spec> {
        fields.try_into()
    }
}

impl Specified for ExprWhile {
    type Spec = LoopSpec;

    fn get_attrs_mut(&mut self) -> &mut Vec<Attribute> {
        &mut self.attrs
    }

    fn parse_spec_from_fields(&mut self, fields: SpecFields) -> Result<Self::Spec> {
        fields.try_into()
    }
}

impl FnSpec {
    pub fn from_spec_and_inputs_attrs(
        raw_spec: SpecFields,
        inputs: &mut Punctuated<FnArg, Token![,]>,
        attrs: &mut Vec<Attribute>,
    ) -> Result<Self> {
        let (input_specs, output_spec_on_exit) = Self::extract_unspec_info(inputs, attrs)?;
        Self::from_spec_and_unspec_info(raw_spec, input_specs, output_spec_on_exit)
    }

    fn extract_unspec_info(
        inputs: &mut Punctuated<FnArg, Token![,]>,
        attrs: &mut Vec<Attribute>,
    ) -> Result<(Vec<InputSpecFlags>, bool)> {
        let mut input_specs = Vec::with_capacity(inputs.len());

        for input in inputs {
            let attrs = match input {
                FnArg::Receiver(receiver) => &mut receiver.attrs,
                FnArg::Typed(pat_type) => &mut pat_type.attrs,
            };

            let input_spec = if let Some(attr) = remove_unique_attr("unspec", attrs)? {
                let unspec: UnspecAttr = attr.try_into()?;
                match unspec.arg {
                    None => InputSpecFlags {
                        on_entry: false,
                        on_exit: false,
                    },
                    Some((_, UnspecArg::In(_))) => InputSpecFlags {
                        on_entry: false,
                        on_exit: true,
                    },
                    Some((_, UnspecArg::Out(_))) => InputSpecFlags {
                        on_entry: true,
                        on_exit: false,
                    },
                }
            } else {
                InputSpecFlags::default()
            };
            input_specs.push(input_spec);
        }

        let output_spec_on_exit = if let Some(attr) = remove_unique_attr("unspec", attrs)? {
            let unspec = UnspecAttr::try_from(attr)?;
            match unspec.arg {
                Some((_, UnspecArg::Out(_))) => false,
                _ => {
                    return Err(Error::new_spanned(
                        Attribute::from(unspec),
                        "only `#[unspec(out)]` is allowed on a `fn` output",
                    ));
                }
            }
        } else {
            true
        };

        Ok((input_specs, output_spec_on_exit))
    }

    fn from_spec_and_unspec_info(
        raw_spec: SpecFields,
        input_specs: Vec<InputSpecFlags>,
        output_spec_on_exit: bool,
    ) -> Result<FnSpec> {
        let span = raw_spec.span();

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
                Keyword::Unknown(_) => {
                    // TODO: Check if it seems like a typo of a known keyword.
                    errors.add(Error::new_spanned(&field.member, "unknown spec field"));
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
                            "at most one `captures` field is allowed; to capture multiple values, use a list: `captures: [binding1 = expr1, binding2 = expr2, ...]`",
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
                    errors.add(Error::new_spanned(&field.member, "not allowed here"));
                }
            }
        }

        if !is_sorted {
            errors.add(Error::new(
                span,
                "fields are out of order: the expected order is: `<QUALIFIERS>`, `requires`, `maintains`, `captures`, `inspects`, `ensures`, where `<QUALIFIERS>` are:\n
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
            input_spec_flags: input_specs,
            output_spec_flag: output_spec_on_exit,
            requires,
            maintains,
            captures,
            ensures,
            span,
        })
    }
}

impl DataSpec {
    pub fn from_spec_and_variants<'a>(
        raw_spec: SpecFields,
        variants: impl Iterator<Item = &'a mut Fields>,
    ) -> Result<Self> {
        let field_specs = variants
            .map(Self::extract_unspec_info)
            .collect::<Result<_>>()?;
        Self::from_spec_and_unspec_info(raw_spec, field_specs)
    }

    fn extract_unspec_info(fields: &mut Fields) -> Result<Vec<bool>> {
        fields
            .iter_mut()
            .map(|field| {
                let Some(attr) = remove_unique_attr("unspec", &mut field.attrs)? else {
                    return Ok(true);
                };

                let unspec = UnspecAttr::try_from(attr)?;
                if unspec.arg.is_some() {
                    return Err(Error::new_spanned(
                        Attribute::from(unspec),
                        "expected `#[unspec]` on a struct or enum field",
                    ));
                }
                Ok(false)
            })
            .collect()
    }

    fn from_spec_and_unspec_info(
        raw_spec: SpecFields,
        field_specs: Vec<Vec<bool>>,
    ) -> Result<Self> {
        let span = raw_spec.span();

        let mut errors = MultiError::empty();
        let mut maintains: Vec<Condition> = vec![];

        for field in raw_spec.fields {
            let keyword = Keyword::from(&field.member);
            match keyword {
                Keyword::Unknown(_) => {
                    errors.add(Error::new_spanned(&field.member, "unknown spec field"));
                }
                Keyword::Maintains => {
                    if let Err(error) = parse_conditions(field, &mut maintains) {
                        errors.add(error);
                    }
                }
                _ => {
                    errors.add(Error::new_spanned(&field.member, "not allowed here"));
                }
            }
        }

        if let Some(combined_error) = errors.get_combined() {
            return Err(combined_error);
        }

        Ok(Self {
            field_spec_flags: field_specs,
            maintains,
            span,
        })
    }
}

impl TryFrom<SpecFields> for LoopSpec {
    type Error = Error;

    fn try_from(raw_spec: SpecFields) -> Result<Self> {
        let span = raw_spec.span();
        let is_sorted = raw_spec.is_sorted();

        let mut errors = MultiError::empty();
        let mut decreases = None;
        let mut maintains: Vec<Condition> = vec![];

        for field in raw_spec.fields {
            let keyword = Keyword::from(&field.member);
            match keyword {
                Keyword::Unknown(_) => {
                    errors.add(Error::new_spanned(&field.member, "unknown spec field"));
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
                            "multiple `decreases` fields are not allowed",
                        ));
                    }
                    if let Err(error) = parse_decreases(field, &mut decreases) {
                        errors.add(error);
                    }
                }
                _ => {
                    errors.add(Error::new_spanned(&field.member, "not allowed here"));
                }
            }
        }

        if !is_sorted {
            errors.add(Error::new(
                span,
                "fields are out of order: the expected order is `maintains`, `decreases`",
            ));
        }

        if let Some(combined_error) = errors.get_combined() {
            return Err(combined_error);
        }

        Ok(Self {
            maintains,
            decreases,
            span,
        })
    }
}

impl TryFrom<SpecFields> for EmptySpec {
    type Error = Error;

    fn try_from(fields: SpecFields) -> Result<Self> {
        if fields.fields.is_empty() {
            Ok(Self)
        } else {
            Err(Error::new_spanned(
                fields,
                "this `#[spec]` attribute must have no fields",
            ))
        }
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
            "attributes are not supported here",
        ));
    }
    if field.colon_token.is_some() {
        return Err(Error::new_spanned(
            field.member,
            "qualifier does not take a value",
        ));
    }
    if qualifiers.contains(value) {
        return Err(Error::new_spanned(
            field.member,
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
            "`cfg` attribute is not supported here",
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
