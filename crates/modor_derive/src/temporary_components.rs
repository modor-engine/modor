use crate::common::{generation, idents};
use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
use syn::DeriveInput;

pub(crate) struct TemporaryComponentStruct<'a> {
    crate_name: String,
    input: &'a DeriveInput,
}

impl<'a> TemporaryComponentStruct<'a> {
    pub(crate) fn new(input: &'a DeriveInput) -> Self {
        Self {
            crate_name: idents::crate_name(),
            input,
        }
    }

    pub(crate) fn temporary_component_impl(&self) -> TokenStream {
        let crate_ = Ident::new(&self.crate_name, Span::call_site());
        let header = generation::impl_header(&self.input.generics, &self.input.ident);
        quote! {
            #[#crate_::systems]
            #header {
                #[run]
                fn remove(mut entity: #crate_::EntityMut<'_>) {
                    entity.delete();
                }
            }
        }
    }
}
