use crate::utils;
use proc_macro2::{Ident, TokenStream};
use quote::quote;
use syn::DeriveInput;

pub(crate) fn impl_block(input: &DeriveInput, trait_ident: &Ident) -> TokenStream {
    let crate_ident = utils::crate_ident();
    let ident = &input.ident;
    let (impl_generics, type_generics, where_clause) = input.generics.split_for_impl();
    quote! {
        #[automatically_derived]
        impl #impl_generics ::#crate_ident::#trait_ident for #ident #type_generics #where_clause {
            const IS_UPDATE_ENABLED: bool = false;

            type Role = ::#crate_ident::NoRole;

            fn update(
                &mut self,
                _access: &mut ::#crate_ident::UpdateContext<'_>
            ) -> ::#crate_ident::Result<()> {
                Ok(())
            }
        }
    }
}
