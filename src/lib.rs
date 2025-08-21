#![doc = include_str!("../README.md")]
use proc_macro::TokenStream;
use proc_macro2::{Ident, Span};
use quote::{ToTokens, quote};
use std::convert::TryFrom;
use syn::spanned::Spanned;
use syn::{
    Expr, ExprClosure, ItemFn, Pat, Result, Token,
    parse::{Parse, ParseStream},
    parse_macro_input, parse_quote,
    punctuated::Punctuated,
};

mod macro;
mod syntax;

pub use macro::contract;
