use proc_macro2::{Ident, Span};
use proc_macro_crate::FoundCrate;

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
