//! Procedural macros of Modor.

use crate::actions::ActionStruct;
use crate::components::ComponentType;
use crate::system_params::SystemParamStruct;
use crate::tests::TestFunction;
use proc_macro::TokenStream;
use proc_macro2::{Ident, Literal, TokenTree};
use quote::quote;
use syn::spanned::Spanned;
use syn::{parse_macro_input, AttributeArgs, DeriveInput, ItemFn, ItemImpl};

#[macro_use]
mod common;

mod actions;
mod attributes;
mod components;
mod idents;
mod impl_block;
mod system_params;
mod systems;
mod tests;

#[allow(missing_docs)] // doc available in `modor` crate
#[proc_macro_attribute]
#[proc_macro_error::proc_macro_error]
pub fn modor_test(attr: TokenStream, item: TokenStream) -> TokenStream {
    let function = parse_macro_input!(item as ItemFn);
    let args = parse_macro_input!(attr as AttributeArgs);
    let (Ok(output) | Err(output)) = TestFunction::new(&function, &args).map(|f| f.annotated());
    output.into()
}

#[allow(missing_docs)] // doc available in `modor` crate
#[proc_macro_derive(Action)]
#[proc_macro_error::proc_macro_error]
pub fn action_derive(item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as DeriveInput);
    ActionStruct::new(&input).action_impl().into()
}

#[allow(missing_docs)] // doc available in `modor` crate
#[proc_macro_derive(Component)]
#[proc_macro_error::proc_macro_error]
pub fn component_derive(item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as DeriveInput);
    ComponentType::new(&input).component_impl().into()
}

#[allow(missing_docs)] // doc available in `modor` crate
#[proc_macro_derive(SingletonComponent)]
#[proc_macro_error::proc_macro_error]
pub fn singleton_component_derive(item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as DeriveInput);
    ComponentType::new(&input).singleton_component_impl().into()
}

#[allow(missing_docs)] // doc available in `modor` crate
#[proc_macro_derive(NoSystem)]
#[proc_macro_error::proc_macro_error]
pub fn no_system_derive(item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as DeriveInput);
    ComponentType::new(&input).no_system_impl().into()
}

#[allow(missing_docs)] // doc available in `modor` crate
#[proc_macro_attribute]
#[proc_macro_error::proc_macro_error]
pub fn systems(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let item = parse_macro_input!(item as ItemImpl);
    let crate_ident = idents::crate_ident(item.span());
    let cleaned_block = impl_block::clean(&item);
    let type_ = &item.self_ty;
    let type_ident = idents::extract_type_ident(type_);
    let generics = &item.generics;
    let (impl_generics, type_generics, where_clause) = generics.split_for_impl();
    let action_type_ident = Ident::new(&(type_ident.to_string() + "Action"), item.span());
    let actions = systems::action_dependencies(&item);
    let generic_type_names: Vec<&Ident> = generics.type_params().map(|g| &g.ident).collect();
    let action_phantom = generics
        .lt_token
        .is_some()
        .then(|| quote!(std::marker::PhantomData <(#(#generic_type_names,)*)>));
    let update_statement = systems::generate_update_statement(&item);
    let finish_system_call = finish_system_call(&type_ident);
    let output = quote! {
        #cleaned_block

        impl #impl_generics #crate_ident::ComponentSystems for #type_ #where_clause {
            type Action = #action_type_ident #type_generics;

            fn on_update(runner: #crate_ident::SystemRunner<'_>) -> #crate_ident::FinishedSystemRunner {
                #update_statement
                #finish_system_call
            }
        }

        #[doc(hidden)]
        #[non_exhaustive]
        #[derive(#crate_ident::Action)]
        pub struct #action_type_ident #impl_generics(#(#actions,)* #action_phantom) #where_clause;
    };
    output.into()
}

#[allow(missing_docs)] // doc available in `modor` crate
#[proc_macro_derive(SystemParam)]
#[proc_macro_error::proc_macro_error]
pub fn system_param_derive(item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as DeriveInput);
    SystemParamStruct::new(&input)
        .custom_system_param_impl()
        .into()
}

#[allow(missing_docs)] // doc available in `modor` crate
#[proc_macro_derive(QuerySystemParam)]
#[proc_macro_error::proc_macro_error]
pub fn query_system_param_derive(item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as DeriveInput);
    SystemParamStruct::new(&input)
        .custom_query_system_param_impl()
        .into()
}

fn finish_system_call(entity_type: &Ident) -> proc_macro2::TokenStream {
    let label = format!("{entity_type}::modor_finish");
    let label_tokens = TokenTree::Literal(Literal::string(&label));
    quote! {
        .finish(#label_tokens)
    }
}
