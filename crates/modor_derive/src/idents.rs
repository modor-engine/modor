use proc_macro2::Ident;
use proc_macro_crate::FoundCrate;
use proc_macro_error::{abort, OptionExt};
use syn::{Type, TypePath};

// TODO: move in `common` module

const CORE_CRATE_NAME: &str = "modor";

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
