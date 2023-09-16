use crate::common::{generation, idents};
use proc_macro2::{Ident, Literal, Span, TokenStream};
use quote::quote;
use syn::{parse_quote, DeriveInput, Type};

pub(crate) struct ComponentType<'a> {
    crate_name: String,
    input: &'a DeriveInput,
}

impl<'a> ComponentType<'a> {
    pub(crate) fn new(input: &'a DeriveInput) -> Self {
        Self {
            crate_name: idents::crate_name(),
            input,
        }
    }

    pub(crate) fn component_impl(&self) -> TokenStream {
        let crate_ = Ident::new(&self.crate_name, Span::call_site());
        self.component_impl_block(parse_quote! { #crate_::False })
    }

    pub(crate) fn singleton_component_impl(&self) -> TokenStream {
        let crate_ = Ident::new(&self.crate_name, Span::call_site());
        self.component_impl_block(parse_quote! { #crate_::True })
    }

    pub(crate) fn no_system_impl(&self) -> TokenStream {
        let crate_ = Ident::new(&self.crate_name, Span::call_site());
        let impl_header = generation::trait_impl_header(
            &self.input.generics,
            &self.input.ident,
            &parse_quote! { #crate_::ComponentSystems },
        );
        let finish_label = Literal::string(&format!("{}::modor_finish", self.input.ident));
        quote! {
           #impl_header {
                type Action = std::marker::PhantomData<()>;

                fn on_update(runner: #crate_::SystemRunner<'_>) -> #crate_::FinishedSystemRunner {
                    runner.finish(#finish_label)
                }
            }
        }
    }

    fn component_impl_block(&self, is_singleton: Type) -> TokenStream {
        let crate_ = Ident::new(&self.crate_name, Span::call_site());
        let impl_header = generation::trait_impl_header(
            &self.input.generics,
            &self.input.ident,
            &parse_quote! { #crate_::Component },
        );
        quote! {
            #impl_header {
                type IsSingleton = #is_singleton;
            }
        }
    }
}
