use proc_macro2::Span;
use syn::{Ident, Pat, PatIdent, visit_mut::VisitMut};

#[cfg(test)]
#[path = "patterns_tests.rs"]
mod patterns_tests;

/// A 'tame' pattern can be used inside a `#[spec]`.
#[derive(Debug, PartialEq, Eq)]
pub enum TamePat {
    /// The pattern binds only references. It may contain wildcard patterns (`_`).
    Borrowing(Pat),
    /// The deconstructed value can be reconstructed from the pattern's bindings.
    Invertible(Pat),
}

/// Tame an irrefutable pattern, so that it may be used inside a `#[spec]`.
///
/// 1. If the pattern binds *only* references, classify as `Borrowing`.
/// 2. If the pattern binds *any* references or contains the rest pattern (`..`), return `Err`.
/// 3. Replace each wildcard pattern with fresh identifiers, then classify as `Invertible`.
pub fn tame_pattern(id_gen: &mut IdentGenerator, mut pat: Pat) -> syn::Result<TamePat> {
    let mut ident_count: u32 = 0;
    let mut ref_count: u32 = 0;
    let mut has_rest = false;
    ForEachMutPattern::with(|subpat| match subpat {
        Pat::Ident(subpat_ident) => {
            ident_count += 1;
            if subpat_ident.by_ref.is_some() {
                ref_count += 1;
            }
        }
        Pat::Rest(_) => has_rest = true,
        _ => {}
    })
    .visit_pat_mut(&mut pat);

    dbg!(ident_count);
    dbg!(ref_count);
    dbg!(has_rest);

    if ident_count == ref_count {
        Ok(TamePat::Borrowing(pat))
    } else if ref_count == 0 && !has_rest {
        ForEachMutPattern::with(|subpat| {
            if let Pat::Wild(subpat_wild) = subpat {
                *subpat = Pat::Ident(PatIdent {
                    attrs: subpat_wild.attrs.clone(),
                    by_ref: None,
                    mutability: None,
                    ident: id_gen.generate_next(),
                    subpat: None,
                });
            }
        })
        .visit_pat_mut(&mut pat);

        Ok(TamePat::Invertible(pat))
    } else {
        Err(syn::Error::new_spanned(pat, "unsupported inside `#[spec]`"))
    }
}

pub struct IdentGenerator {
    index: usize,
}

impl IdentGenerator {
    pub fn new() -> Self {
        Self { index: 0 }
    }

    pub fn generate_next(&mut self) -> Ident {
        self.index += 1;
        syn::Ident::new(
            &format!("__anodized_ident_{}", self.index),
            Span::mixed_site(),
        )
    }
}

impl Default for IdentGenerator {
    fn default() -> Self {
        Self::new()
    }
}

struct ForEachMutPattern<F> {
    body: F,
}

impl<F> ForEachMutPattern<F>
where
    F: FnMut(&mut Pat),
{
    pub fn with(body: F) -> Self {
        Self { body }
    }
}

impl<F> VisitMut for ForEachMutPattern<F>
where
    F: FnMut(&mut Pat),
{
    fn visit_pat_mut(&mut self, pat: &mut syn::Pat) {
        (self.body)(pat);
        syn::visit_mut::visit_pat_mut(self, pat);
    }
}
