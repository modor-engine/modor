use crate::utils;
use proc_macro2::TokenStream;
use quote::quote;
use syn::DeriveInput;

pub(crate) fn impl_block(input: &DeriveInput) -> TokenStream {
    let crate_ident = utils::crate_ident();
    let ident = &input.ident;
    let (impl_generics, type_generics, where_clause) = input.generics.split_for_impl();
    quote! {
        #[automatically_derived]
        impl #impl_generics ::#crate_ident::Singleton for #ident #type_generics #where_clause {}
    }
}
