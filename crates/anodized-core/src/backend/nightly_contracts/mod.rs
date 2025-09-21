use crate::Spec;

use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::{ToTokens, quote};
use syn::{Block, Ident, ItemFn, Meta, parse::Result, parse_quote};

/// Takes the spec and the original function and returns a new instrumented function.
pub fn instrument_fn(spec: &Spec, original_fn: ItemFn) -> Result<ItemFn> {
    todo!()
}
