#[cfg(test)]
#[path = "loops_tests.rs"]
mod loops_tests;

use proc_macro2::Span;
use syn::{
    Block, Error, Expr, ExprClosure, ExprForLoop, ExprWhile, Ident, ItemFn, Result, Stmt,
    parse_quote,
    visit_mut::{self, VisitMut},
};

use crate::{LoopSpec, annotate::Specified, instrument::Mode};

impl Mode {
    pub fn instrument_loops_in_fn_body(&self, body: &mut Block) -> Result<()> {
        let mut visitor = LoopSpecVisitor::new(self);
        visitor.visit_block_mut(body);
        visitor.finish()
    }

    pub fn instrument_expr_while(&self, spec: LoopSpec, expr_while: &mut ExprWhile) {
        self.instrument_loop_body(spec, &mut expr_while.body.stmts);
    }

    pub fn instrument_expr_for_loop(&self, spec: LoopSpec, expr_for_loop: &mut ExprForLoop) {
        self.instrument_loop_body(spec, &mut expr_for_loop.body.stmts);
    }

    fn instrument_loop_body(&self, spec: LoopSpec, stmts: &mut Vec<Stmt>) {
        if let Self::EmbedSpecs = self {
            let maintains_block = Self::build_precondition_fn_body(&[], &spec.maintains);
            stmts.insert(
                0,
                parse_quote! {
                    let __anodized_loop_maintains = || -> bool #maintains_block;
                },
            );

            let mut variant_stmts: Vec<Stmt> = Vec::new();
            let mut variant_names: Vec<Ident> = Vec::new();
            if let Some(loop_variant) = &spec.decreases {
                let i = variant_names.len();
                let name = Ident::new(&format!("__anodized_value_{}", i + 1), Span::mixed_site());
                let expr = &loop_variant.expr;
                variant_stmts.push(parse_quote! { let #name = (|| #expr)(); });
                variant_names.push(name);
            }
            let variant_expr: Option<Expr> = if !variant_names.is_empty() {
                Some(parse_quote! { (#(#variant_names),*) })
            } else {
                None
            };

            stmts.insert(
                1,
                parse_quote! {
                    let __anodized_loop_decreases = || {
                        #(#variant_stmts)*
                        #variant_expr
                    };
                },
            );
        }
    }
}

struct LoopSpecVisitor<'a> {
    config: &'a Mode,
    errors: Option<Error>,
}

impl<'a> LoopSpecVisitor<'a> {
    fn new(config: &'a Mode) -> Self {
        Self {
            config,
            errors: None,
        }
    }

    fn finish(self) -> Result<()> {
        match self.errors {
            Some(error) => Err(error),
            None => Ok(()),
        }
    }

    fn add_error(&mut self, error: Error) {
        match &mut self.errors {
            Some(existing) => existing.combine(error),
            None => self.errors = Some(error),
        }
    }
}

impl VisitMut for LoopSpecVisitor<'_> {
    fn visit_expr_while_mut(&mut self, expr_while: &mut ExprWhile) {
        let spec = match expr_while.parse_spec_from_attrs() {
            Ok(spec) => spec,
            Err(error) => {
                self.add_error(error);
                return;
            }
        };

        visit_mut::visit_expr_while_mut(self, expr_while);

        if spec.is_empty() {
            return;
        }

        self.config
            .instrument_loop_body(spec, &mut expr_while.body.stmts);
    }

    fn visit_expr_for_loop_mut(&mut self, expr_for_loop: &mut ExprForLoop) {
        let spec = match expr_for_loop.parse_spec_from_attrs() {
            Ok(spec) => spec,
            Err(error) => {
                self.add_error(error);
                return;
            }
        };

        visit_mut::visit_expr_for_loop_mut(self, expr_for_loop);

        if spec.is_empty() {
            return;
        }

        self.config.instrument_expr_for_loop(spec, expr_for_loop);
    }

    // Nested closure scopes are independently analyzed by the outer function macro expansion.
    fn visit_expr_closure_mut(&mut self, _expr_closure: &mut ExprClosure) {}

    // Nested `fn` items are independently analyzed by the outer function macro expansion.
    fn visit_item_fn_mut(&mut self, _item_fn: &mut ItemFn) {}
}
