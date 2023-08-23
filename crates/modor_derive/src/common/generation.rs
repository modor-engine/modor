use proc_macro2::{Ident, TokenStream};
use quote::quote;
use syn::{parse_quote, Generics, ItemStruct, Type, Visibility};

pub(crate) fn impl_header(generics: &Generics, struct_: &Ident, trait_: &Type) -> TokenStream {
    let (impl_generics, type_generics, where_clause) = generics.split_for_impl();
    quote!(
        impl #impl_generics #trait_ for #struct_ #type_generics #where_clause
    )
}

pub(crate) fn tuple_struct(
    visibility: &Visibility,
    type_: &Ident,
    generics: &Generics,
    field_types: &[Type],
) -> ItemStruct {
    let (impl_generics, _type_generics, where_clause) = generics.split_for_impl();
    parse_quote! {
        #visibility struct #type_ #impl_generics(#(#field_types),*) #where_clause;
    }
}
