use proc_macro2::{Ident, Span, TokenStream};
use proc_macro_crate::FoundCrate;
use quote::ToTokens;
use syn::Type;

pub(crate) fn crate_ident() -> Ident {
    const CORE_CRATE_NAME: &str = "modor";
    Ident::new(
        match &proc_macro_crate::crate_name(CORE_CRATE_NAME) {
            Ok(FoundCrate::Itself) | Err(_) => CORE_CRATE_NAME, // no-coverage (never reached)
            Ok(FoundCrate::Name(name)) => name,
        },
        Span::call_site(),
    )
}

pub(crate) fn error(span: Span, error: &str) -> TokenStream {
    syn::Error::new(span, error).into_compile_error()
}

pub(crate) fn has_type_name_without_generic(type_: &Type, type_str: &str) -> bool {
    let actual_type_str = type_.to_token_stream().to_string();
    actual_type_str == type_str || actual_type_str.ends_with(&format!(":: {type_str}"))
}
