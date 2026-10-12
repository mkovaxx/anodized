use quote::ToTokens;
use syn::{
    Ident, Macro, MacroDelimiter, Token, Type, TypeMacro,
    parse::{Parse, ParseStream, Result},
};

use crate::syntax::path_matches_name;

/// A `Spec!`-marked type.
///
/// Has three valid forms:
/// - `Spec!(T)`
/// - `Spec!(T, out)`
/// - `Spec!(T, inout)`
#[derive(Debug)]
pub struct SpecMarker {
    pub spec: Ident,
    pub bang_token: Token![!],
    pub delimiter: MacroDelimiter,
    pub args: SpecMarkerArgs,
}

/// The arguments of a `Spec!` marker.
#[derive(Debug)]
pub struct SpecMarkerArgs {
    pub ty: Type,
    /// Must be `None` on the output of a `fn` or a field of a `struct` or `enum`.
    pub mode: Option<(Token![,], FnArgMode)>,
}

/// The enforcement mode used optionally inside a `Spec!` marker.
#[derive(Debug)]
pub enum FnArgMode {
    Out(kw::out),
    InOut(kw::inout),
}

impl From<SpecMarker> for Type {
    fn from(marker: SpecMarker) -> Self {
        Type::Macro(TypeMacro {
            mac: Macro {
                path: marker.spec.into(),
                bang_token: marker.bang_token,
                delimiter: marker.delimiter,
                tokens: marker.args.into_token_stream(),
            },
        })
    }
}

impl ToTokens for SpecMarkerArgs {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        self.ty.to_tokens(tokens);
        if let Some((comma, mode)) = &self.mode {
            comma.to_tokens(tokens);
            mode.to_tokens(tokens);
        }
    }
}

impl ToTokens for FnArgMode {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        match self {
            FnArgMode::Out(out) => out.to_tokens(tokens),
            FnArgMode::InOut(inout) => inout.to_tokens(tokens),
        }
    }
}

/// Removes a `Spec!(...)` marker from a type, if it has one.
pub fn extract_spec_marker(ty: &mut Type) -> Result<Option<SpecMarker>> {
    let Type::Macro(TypeMacro { mac }) = ty else {
        return Ok(None);
    };
    if !path_matches_name(&mac.path, "Spec") {
        return Ok(None);
    }

    let mac = mac.clone();
    let args: SpecMarkerArgs = syn::parse2(mac.tokens.clone())?;
    *ty = args.ty.clone();

    Ok(Some(SpecMarker {
        spec: mac.path.require_ident()?.clone(),
        bang_token: mac.bang_token,
        delimiter: mac.delimiter.clone(),
        args,
    }))
}

impl Parse for SpecMarkerArgs {
    fn parse(input: ParseStream) -> Result<Self> {
        let ty = input.parse()?;
        let mode = if input.is_empty() {
            None
        } else {
            let comma = input.parse::<Token![,]>()?;
            if input.peek(kw::out) {
                Some((comma, FnArgMode::Out(input.parse()?)))
            } else if input.peek(kw::inout) {
                Some((comma, FnArgMode::InOut(input.parse()?)))
            } else {
                return Err(input.error("expected a mode, `out` or `inout`"));
            }
        };

        if !input.is_empty() {
            return Err(input.error("expected exactly one mode, `out` or `inout`"));
        }

        Ok(Self { ty, mode })
    }
}

mod kw {
    syn::custom_keyword!(out);
    syn::custom_keyword!(inout);
}
