use proc_macro2::{Ident, Span, TokenStream};
use proc_macro_crate::FoundCrate;
use syn::{GenericArgument, GenericParam, Generics, PathArguments, Type};

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

pub(crate) fn first_generic_type(ty: Type) -> Option<Type> {
    if let Type::Path(path) = ty {
        for segment in path.path.segments {
            if let PathArguments::AngleBracketed(angle_bracketed) = segment.arguments {
                for arg in angle_bracketed.args {
                    if let GenericArgument::Type(generic_type) = arg {
                        return Some(generic_type);
                    }
                }
            }
        }
    }
    None
}

pub(crate) fn generic_type_idents(generics: &Generics) -> Vec<&Ident> {
    generics
        .params
        .iter()
        .filter_map(|param| {
            if let GenericParam::Type(type_param) = param {
                Some(&type_param.ident)
            } else {
                None
            }
        })
        .collect()
}
