pub mod anodized;

use syn::ItemFn;

use crate::Spec;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Backend {
    /// Anodized instrumentation with runtime checks.
    Default,
    /// Anodized instrumentation with no runtime checks.
    NoChecks,
}

pub fn handle_fn(backend: Backend, spec: Spec, mut func: ItemFn) -> syn::Result<ItemFn> {
    let is_async = func.sig.asyncness.is_some();

    // Extract the return type from the function signature
    let return_type = match &func.sig.output {
        syn::ReturnType::Default => syn::parse_quote!(()),
        syn::ReturnType::Type(_, ty) => ty.as_ref().clone(),
    };

    // Generate the new, instrumented function body.
    let disable_runtime_checks = backend != Backend::Default;
    let new_body = anodized::instrument_fn_body(
        &spec,
        &func.block,
        is_async,
        &return_type,
        disable_runtime_checks,
    )?;

    // Replace the old function body with the new one.
    *func.block = new_body;

    match backend {
        Backend::Default | Backend::NoChecks => Ok(func),
    }
}
