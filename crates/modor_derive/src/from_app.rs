use crate::utils;
use proc_macro2::{Literal, TokenStream};
use quote::{quote, ToTokens};
use syn::__private::Span;
use syn::{Data, DeriveInput, Field, Type};

pub(crate) fn impl_block(input: &DeriveInput) -> Result<TokenStream, TokenStream> {
    let crate_ident = utils::crate_ident();
    let ident = &input.ident;
    let (impl_generics, type_generics, where_clause) = input.generics.split_for_impl();
    let fields = fields(input)
        .ok_or_else(|| utils::error(Span::call_site(), "only structs are supported"))?;
    let field_idents = fields.iter().map(|(ident, _)| ident);
    let field_types = fields.iter().map(|(_, ty)| ty);
    Ok(quote! {
        #[automatically_derived]
        impl #impl_generics ::#crate_ident::FromApp for #ident #type_generics #where_clause {
            fn from_app(app: &mut ::#crate_ident::App) -> Self {
                Self {
                    #(#field_idents: <#field_types as ::#crate_ident::FromApp>::from_app(app),)*
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
