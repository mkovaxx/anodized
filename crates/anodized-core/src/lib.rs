//! Core interoperability for the Anodized correctness ecosystem.
//!
#![doc = include_str!("../README.md")]

use proc_macro2::Span;
use quote::{ToTokens, quote};
use syn::{
    Attribute, Block, Expr, Ident, Meta, Pat, Token,
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
    /// Captures: expressions to snapshot at function entry for use in postconditions.
    pub captures: Vec<Capture>,
    /// Postconditions: conditions that must hold when the function returns.
    pub ensures: Vec<PostCondition>,
}

/// A condition represented by a `bool`-valued expression.
#[derive(Debug)]
pub struct Condition {
    /// The `bool`-valued expression.
    pub expr: Expr,
    /// **Static analyzers can safely ignore this field.**
    ///
    /// Build configuration filter to decide whether to add runtime checks.
    /// Passed to a `cfg!()` guard in the instrumented function.
    pub cfg: Option<Meta>,
}

/// A postcondition represented by a closure that takes the return value as a reference.
#[derive(Debug)]
pub struct PostCondition {
    /// The closure that validates the postcondition, e.g. `|output| output > 0`.
    /// The closure always receives the return value as a reference.
    pub closure: syn::ExprClosure,
    /// **Static analyzers can safely ignore this field.**
    ///
    /// Build configuration filter to decide whether to add runtime checks.
    /// Passed to a `cfg!()` guard in the instrumented function.
    pub cfg: Option<Meta>,
}

/// Captures an expression's value at function entry.
#[derive(Debug)]
pub struct Capture {
    /// The expression to capture.
    pub expr: Expr,
    /// The identifier to bind the captured value to.
    pub alias: Ident,
}

impl Parse for Spec {
    fn parse(input: ParseStream) -> Result<Self> {
        let args = Punctuated::<SpecArg, Token![,]>::parse_terminated(input)?;

        let mut last_arg_order: Option<ArgOrder> = None;
        let mut requires: Vec<Condition> = vec![];
        let mut maintains: Vec<Condition> = vec![];
        let mut captures: Vec<Capture> = vec![];
        let mut binds_pattern: Option<Pat> = None;
        let mut ensures: Vec<PostCondition> = vec![];

        for arg in args {
            let current_arg_order = arg.get_order();
            if let Some(last_order) = last_arg_order {
                if current_arg_order < last_order {
                    return Err(syn::Error::new(
                        arg.get_keyword_span(),
                        "parameters are out of order: their order must be `requires`, `maintains`, `captures`, `binds`, `ensures`",
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
                SpecArg::Captures { keyword, expr } => {
                    if !captures.is_empty() {
                        return Err(syn::Error::new(
                            keyword.span(),
                            "at most one `captures` parameter is allowed; to capture multiple values, use a list: `captures: [expr1, expr2, ...]`",
                        ));
                    }
                    if let Expr::Array(array) = expr {
                        captures.extend(interpret_array_as_captures(array)?);
                    } else {
                        captures.push(interpret_expr_as_capture(expr)?);
                    }
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
                    let default_pattern = binds_pattern.clone().unwrap_or(parse_quote! { output });

                    if let Expr::Array(conditions) = expr {
                        ensures.extend(conditions.elems.into_iter().map(|expr| PostCondition {
                            closure: interpret_expr_as_postcondition(expr, default_pattern.clone()),
                            cfg: cfg.clone(),
                        }));
                    } else {
                        ensures.push(PostCondition {
                            closure: interpret_expr_as_postcondition(expr, default_pattern),
                            cfg,
                        });
                    }
                }
            }
        }

        Ok(Spec {
            requires,
            maintains,
            captures,
            ensures,
        })
    }
}

#[derive(PartialEq, PartialOrd, Clone, Copy, Debug)]
enum ArgOrder {
    Requires,
    Maintains,
    Captures,
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
    Captures {
        keyword: kw::captures,
        expr: Expr,
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
            SpecArg::Captures { .. } => ArgOrder::Captures,
            SpecArg::Binds { .. } => ArgOrder::Binds,
            SpecArg::Ensures { .. } => ArgOrder::Ensures,
        }
    }

    fn get_keyword_span(&self) -> Span {
        match self {
            SpecArg::Requires { keyword, .. } => keyword.span,
            SpecArg::Ensures { keyword, .. } => keyword.span,
            SpecArg::Maintains { keyword, .. } => keyword.span,
            SpecArg::Captures { keyword, .. } => keyword.span,
            SpecArg::Binds { keyword, .. } => keyword.span,
        }
    }
}

impl Parse for SpecArg {
    fn parse(input: ParseStream) -> Result<Self> {
        let attrs = input.call(Attribute::parse_outer)?;
        let cfg = parse_cfg_attribute(&attrs)?;

        let lookahead = input.lookahead1();
        if lookahead.peek(kw::captures) {
            if cfg.is_some() {
                return Err(syn::Error::new(
                    attrs[0].span(),
                    "`cfg` attribute is not supported on `captures`",
                ));
            }

            // Parse `captures: <captures>`
            let keyword = input.parse::<kw::captures>()?;
            input.parse::<Token![:]>()?;
            Ok(SpecArg::Captures {
                keyword,
                expr: input.parse()?,
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

fn interpret_expr_as_postcondition(expr: Expr, default_pattern: Pat) -> syn::ExprClosure {
    match expr {
        // Already a closure - use as-is
        Expr::Closure(closure) => closure,
        // Naked expression - wrap in a closure with default binding
        expr => {
            parse_quote! { |#default_pattern| #expr }
        }
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
    syn::custom_keyword!(captures);
    syn::custom_keyword!(binds);
    syn::custom_keyword!(ensures);
}

/// Takes the spec and the original body and returns a new instrumented function body.
pub fn instrument_fn_body(
    spec: &Spec,
    original_body: &Block,
    is_async: bool,
    return_type: &syn::Type,
) -> Result<Block> {
    // The identifier for the return value binding. It's hygienic to prevent collisions.
    let binding_ident = Ident::new("__anodized_output", Span::mixed_site());

    // --- Generate Precondition Checks ---
    let preconditions = spec
        .requires
        .iter()
        .map(|condition| {
            let expr = &condition.expr;
            let expr_str = expr.to_token_stream().to_string();
            let assert = quote! { assert!(#expr, "Precondition failed: {}", #expr_str); };
            if let Some(cfg) = &condition.cfg {
                quote! { if cfg!(#cfg) { #assert } }
            } else {
                assert
            }
        })
        .chain(spec.maintains.iter().map(|condition| {
            let expr = &condition.expr;
            let expr_str = expr.to_token_stream().to_string();
            let assert = quote! { assert!(#expr, "Pre-invariant failed: {}", #expr_str); };
            if let Some(cfg) = &condition.cfg {
                quote! { if cfg!(#cfg) { #assert } }
            } else {
                assert
            }
        }));

    // --- Generate Combined Body and Capture Statement ---
    // Capture values and execute body in a single tuple assignment
    // This ensures captured values aren't accessible to the body itself

    // Chain capture aliases with output binding
    let aliases = spec
        .captures
        .iter()
        .map(|cb| &cb.alias)
        .chain(std::iter::once(&binding_ident));

    // Chain capture expressions with body expression
    let capture_exprs = spec.captures.iter().map(|cb| {
        let expr = &cb.expr;
        quote! { #expr }
    });

    // Chain underscore types with return type for tuple type annotation
    let types = spec
        .captures
        .iter()
        .map(|_| quote! { _ })
        .chain(std::iter::once(quote! { #return_type }));

    let body_expr = if is_async {
        quote! { async #original_body.await }
    } else {
        quote! { #original_body }
    };

    let exprs = capture_exprs.chain(std::iter::once(body_expr));

    // Build tuple assignment with type annotation on the tuple
    let body_and_captures = quote! {
        let (#(#aliases),*): (#(#types),*) = (#(#exprs),*);
    };

    // --- Generate Postcondition Checks ---
    let postconditions = spec
        .maintains
        .iter()
        .map(|condition| {
            let expr = &condition.expr;
            let expr_str = expr.to_token_stream().to_string();
            let assert = quote! { assert!(#expr, "Post-invariant failed: {}", #expr_str); };
            if let Some(cfg) = &condition.cfg {
                quote! { if cfg!(#cfg) { #assert } }
            } else {
                assert
            }
        })
        .chain(spec.ensures.iter().map(|postcondition| {
            let pattern = &postcondition.pattern;
            let expr = &postcondition.expr;
            // Format error message with explicit binding syntax
            let pattern_str = pattern.to_token_stream().to_string();
            let expr_str = expr.to_token_stream().to_string();
            let error_msg = format!("{} => {}", pattern_str, expr_str);
            let assert = quote! {
                {
                    let #pattern = #binding_ident;
                    assert!(#expr, "Postcondition failed: {}", #error_msg);
                }
            };
            if let Some(cfg) = &postcondition.cfg {
                quote! { if cfg!(#cfg) { #assert } }
            } else {
                assert
            }
        }));

    Ok(parse_quote! {
        {
            #(#preconditions)*
            #body_and_captures
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
