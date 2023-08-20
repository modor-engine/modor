use proc_macro2::{Ident, Span};
use proc_macro_crate::FoundCrate;
use proc_macro_error::{abort, OptionExt};
use std::sync::OnceLock;
use syn::{Type, TypePath};

// TODO: move in `common` module

const CORE_CRATE_NAME: &str = "modor";

// TODO: remove
pub(crate) fn crate_ident(span: Span) -> Ident {
    static CRATE_NAME: OnceLock<String> = OnceLock::new();
    let crate_name =
        CRATE_NAME.get_or_init(|| match proc_macro_crate::crate_name(CORE_CRATE_NAME) {
            Ok(FoundCrate::Itself) | Err(_) => CORE_CRATE_NAME.to_string(),
            Ok(FoundCrate::Name(name)) => name,
        });
    Ident::new(crate_name, span)
}

pub(crate) fn crate_name() -> String {
    match proc_macro_crate::crate_name(CORE_CRATE_NAME) {
        Ok(FoundCrate::Itself) | Err(_) => CORE_CRATE_NAME.to_string(),
        Ok(FoundCrate::Name(name)) => name,
    }
}

pub(crate) fn extract_type_ident(type_: &Type) -> Ident {
    let Type::Path(TypePath { path, .. }) = type_ else {
        abort!(type_, "only path types are supported (for example, `module::Type<GenericType>`)");
    };
    path.segments
        .last()
        .expect_or_abort("type name cannot be parsed")
        .ident
        .clone()
}
