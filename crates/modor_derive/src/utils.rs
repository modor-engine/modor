use proc_macro2::{Ident, Span, TokenStream};
use proc_macro_crate::FoundCrate;
use quote::{quote_spanned, ToTokens};
use syn::spanned::Spanned;
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

pub(crate) fn has_type_name_without_generic(type_: &Type, type_str: &str) -> bool {
    let actual_type_str = type_.to_token_stream().to_string();
    actual_type_str == type_str || actual_type_str.ends_with(&format!(":: {type_str}"))
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

pub(crate) fn generic_phantom_type(generics: &Generics) -> TokenStream {
    let types = generics.params.iter().filter_map(|param| match param {
        GenericParam::Lifetime(lt) => Some(quote_spanned! {lt.span() => &#lt ()}),
        GenericParam::Type(ty) => {
            let ident = &ty.ident;
            Some(quote_spanned! {ty.span() => fn(#ident)})
        }
        GenericParam::Const(_) => None,
    });
    quote_spanned! {generics.span() => ::std::marker::PhantomData<(#(#types,)*)>}
}

#[cfg(test)]
mod tests {
    use syn::Type;

    #[test]
    fn extract_first_generic_type() -> syn::Result<()> {
        assert!(super::first_generic_type(syn::parse_str("[usize; 2]")?).is_none());
        assert!(super::first_generic_type(syn::parse_str("fn(usize)")?).is_none());
        assert!(super::first_generic_type(syn::parse_str("T<'a>")?).is_none());
        assert_eq!(
            super::first_generic_type(syn::parse_str("T<U>")?),
            Some(syn::parse_str::<Type>("U")?)
        );
        Ok(())
    }
}
