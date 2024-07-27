use crate::utils;
use proc_macro2::{Ident, Literal, TokenStream};
use quote::{quote, quote_spanned, ToTokens};
use syn::__private::Span;
use syn::spanned::Spanned;
use syn::{Data, DeriveInput, Field, Type};

pub(crate) fn impl_block(input: &DeriveInput) -> Result<TokenStream, TokenStream> {
    let crate_ident = utils::crate_ident();
    let ident = &input.ident;
    let (impl_generics, type_generics, where_clause) = input.generics.split_for_impl();
    let fields = fields(input)
        .ok_or_else(|| utils::error(Span::call_site(), "only structs are supported"))?;
    let statements = fields
        .iter()
        .map(|(ident, ty)| create_statement(&crate_ident, ident, ty));
    Ok(quote! {
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl #impl_generics ::#crate_ident::FromApp for #ident #type_generics #where_clause {
            fn from_app(app: &mut ::#crate_ident::App) -> Self {
                Self {
                    #(#statements)*
                }
            }
        }
    })
}

fn fields(input: &DeriveInput) -> Option<Vec<(TokenStream, &Type)>> {
    match &input.data {
        Data::Struct(data) => Some(
            data.fields
                .iter()
                .enumerate()
                .map(|(index, field)| (field_ident(index, field), &field.ty))
                .collect(),
        ),
        Data::Enum(_) | Data::Union(_) => None,
    }
}

fn field_ident(index: usize, field: &Field) -> TokenStream {
    if let Some(ident) = &field.ident {
        ident.to_token_stream()
    } else {
        Literal::usize_unsuffixed(index).to_token_stream()
    }
}

fn create_statement(crate_ident: &Ident, ident: &TokenStream, type_: &Type) -> TokenStream {
    if utils::has_type_name_without_generic(type_, "Instant") {
        quote_spanned! {
            type_.span() =>
            #ident: #type_::now(),
        }
    } else {
        quote_spanned! {
            type_.span() =>
            #ident: <#type_ as ::#crate_ident::FromApp>::from_app(app),
        }
    }
}

#[cfg(test)]
mod tests {
    use syn::DeriveInput;

    #[test]
    fn derive_enum() -> syn::Result<()> {
        let input = syn::parse_str::<DeriveInput>("enum Test {}")?;
        assert!(super::impl_block(&input).is_err());
        Ok(())
    }
}
