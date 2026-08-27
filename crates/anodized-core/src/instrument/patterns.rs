use proc_macro2::Span;
use syn::{
    Ident, Pat, PatConst, PatIdent, PatLit, PatParen, PatPath, PatRange, PatSlice, PatStruct,
    PatTuple, PatTupleStruct, visit_mut::VisitMut,
};

#[cfg(test)]
#[path = "patterns_tests.rs"]
mod patterns_tests;

pub enum PatClass {
    /// The pattern binds only references. It may contain wildcard patterns (`_`).
    Borrowing(Pat),
    /// The deconstructed value can be reconstructed from the pattern's bindings.
    Invertible(Pat),
}

/// Preprocess an irrefutable pattern, so that it may be used inside a `#[spec]`.
///
/// 1. If the pattern binds *only* references, classify as `Borrowing`.
/// 2. If the pattern binds *any* references or contains the rest pattern (`..`), return `Err`.
/// 3. Replace each wildcard pattern with fresh identifiers, then classify as `Invertible`.
pub fn classify_pattern(id_gen: &mut IdentGenerator, mut pat: Pat) -> syn::Result<PatClass> {
    let mut collector = BindingCollector::new();
    collector.visit_pat_mut(&mut pat);

    let has_only_refs = collector.bindings.iter().all(|binding| match binding {
        Pat::Ident(pat_ident) => pat_ident.by_ref.is_some(),
        _ => true,
    });
    if has_only_refs {
        return Ok(PatClass::Borrowing(pat));
    }

    for binding in collector.bindings {
        match binding {
            Pat::Ident(pat_ident) if pat_ident.by_ref.is_none() => todo!(),
            Pat::Wild(pat_wild) => {
                *binding = Pat::Ident(PatIdent {
                    attrs: pat_wild.attrs.clone(),
                    by_ref: None,
                    mutability: None,
                    ident: id_gen.next(),
                    subpat: None,
                })
            }
            _ => {
                return Err(syn::Error::new_spanned(
                    binding,
                    "not supported inside `#[spec]`",
                ));
            }
        }
    }

    Ok(PatClass::Invertible(pat))
}

pub struct IdentGenerator {
    index: usize,
}

impl IdentGenerator {
    pub fn new() -> Self {
        Self { index: 1 }
    }

    pub fn next(&mut self) -> Ident {
        self.index += 1;
        syn::Ident::new(
            &format!("__anodized_ident_{}", self.index),
            Span::mixed_site(),
        )
    }
}

struct BindingCollector<'a> {
    pub bindings: Vec<&'a mut Pat>,
}

impl BindingCollector<'_> {
    pub fn new() -> Self {
        Self { bindings: vec![] }
    }
}

impl<'a> VisitMut for BindingCollector<'a> {}

/// Sanitize a pattern to be valid as an expression that reconstructs the matched value.
pub fn sanitize_pat_as_expr(pat: &Pat) -> syn::Result<Pat> {
    match pat {
        Pat::Const(pat_const) => Ok(Pat::Const(PatConst {
            attrs: vec![],
            const_token: pat_const.const_token,
            block: pat_const.block.clone(),
        })),
        Pat::Ident(pat_ident) if pat_ident.by_ref.is_none() => {
            let None = pat_ident.by_ref else {
                return Err(syn::Error::new_spanned(
                    pat_ident.by_ref,
                    "not allowed here due to `#[spec]`",
                ));
            };
            Ok(Pat::Ident(PatIdent {
                attrs: vec![],
                by_ref: None,
                mutability: None,
                ident: pat_ident.ident.clone(),
                subpat: None,
            }))
        }
        Pat::Lit(pat_lit) => Ok(Pat::Lit(PatLit {
            attrs: vec![],
            lit: pat_lit.lit.clone(),
        })),
        Pat::Path(pat_path) => Ok(Pat::Path(PatPath {
            attrs: vec![],
            qself: pat_path.qself.clone(),
            path: pat_path.path.clone(),
        })),
        Pat::Range(pat_range) => Ok(Pat::Range(PatRange {
            attrs: vec![],
            start: pat_range.start.clone(),
            limits: pat_range.limits,
            end: pat_range.end.clone(),
        })),
        Pat::Paren(pat_paren) => Ok(Pat::Paren(PatParen {
            attrs: vec![],
            paren_token: pat_paren.paren_token,
            pat: sanitize_pat_as_expr(&pat_paren.pat)?.into(),
        })),
        Pat::Reference(pat_reference) => sanitize_pat_as_expr(&pat_reference.pat),
        Pat::Type(pat_type) => sanitize_pat_as_expr(&pat_type.pat),
        Pat::Struct(pat_struct) => {
            let None = pat_struct.rest else {
                return Err(syn::Error::new_spanned(
                    &pat_struct.rest,
                    "not allowed here due to `#[spec]`",
                ));
            };
            let mut fields = syn::punctuated::Punctuated::<syn::FieldPat, syn::token::Comma>::new();
            for field_pat in &pat_struct.fields {
                let field_value = syn::FieldPat {
                    attrs: vec![],
                    member: field_pat.member.clone(),
                    colon_token: field_pat.colon_token,
                    pat: sanitize_pat_as_expr(&field_pat.pat)?.into(),
                };
                fields.push(field_value);
            }
            Ok(Pat::Struct(PatStruct {
                attrs: vec![],
                qself: pat_struct.qself.clone(),
                path: pat_struct.path.clone(),
                brace_token: pat_struct.brace_token,
                fields,
                rest: None,
            }))
        }
        Pat::Tuple(pat_tuple) => {
            let mut elems = syn::punctuated::Punctuated::<syn::Pat, syn::token::Comma>::new();
            for elem_pat in &pat_tuple.elems {
                let elem_value = sanitize_pat_as_expr(elem_pat)?;
                elems.push(elem_value);
            }
            Ok(Pat::Tuple(PatTuple {
                attrs: vec![],
                paren_token: pat_tuple.paren_token,
                elems,
            }))
        }
        Pat::TupleStruct(pat_tuple_struct) => {
            let mut elems = syn::punctuated::Punctuated::<syn::Pat, syn::token::Comma>::new();
            for elem_pat in &pat_tuple_struct.elems {
                let elem_value = sanitize_pat_as_expr(elem_pat)?;
                elems.push(elem_value);
            }
            Ok(Pat::TupleStruct(PatTupleStruct {
                attrs: vec![],
                qself: pat_tuple_struct.qself.clone(),
                path: pat_tuple_struct.path.clone(),
                paren_token: pat_tuple_struct.paren_token,
                elems,
            }))
        }
        Pat::Slice(pat_slice) => {
            let mut elems = syn::punctuated::Punctuated::<syn::Pat, syn::token::Comma>::new();
            for elem_pat in &pat_slice.elems {
                let elem_value = sanitize_pat_as_expr(elem_pat)?;
                elems.push(elem_value);
            }
            Ok(Pat::Slice(PatSlice {
                attrs: vec![],
                bracket_token: pat_slice.bracket_token,
                elems,
            }))
        }
        Pat::Verbatim(token_stream) => Ok(Pat::Verbatim(token_stream.clone())),
        Pat::Macro(pat_macro) => Err(syn::Error::new_spanned(
            pat_macro,
            "not allowed here due to `#[spec]`",
        )),
        Pat::Or(pat_or) => Err(syn::Error::new_spanned(
            pat_or,
            "or-pattern not allowed here due to `#[spec]`",
        )),
        Pat::Rest(pat_rest) => Err(syn::Error::new_spanned(
            pat_rest,
            "not allowed here due to `#[spec]`",
        )),
        Pat::Wild(pat_wild) => Err(syn::Error::new_spanned(
            pat_wild,
            "not allowed here due to `#[spec]`",
        )),
        _ => Err(syn::Error::new_spanned(
            pat,
            "not allowed here due to `#[spec]`",
        )),
    }
}
