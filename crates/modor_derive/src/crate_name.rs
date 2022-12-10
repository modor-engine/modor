use proc_macro2::{Ident, Span};
use proc_macro_crate::FoundCrate;

const CORE_CRATE_NAME: &str = "modor";

pub(crate) fn find_crate_ident(span: Span) -> Ident {
    match proc_macro_crate::crate_name(CORE_CRATE_NAME) {
        Ok(FoundCrate::Itself) => Ident::new("crate", span),
        Ok(FoundCrate::Name(name)) => Ident::new(&name, span),
        Err(_) => Ident::new(CORE_CRATE_NAME, span),
    }
}
