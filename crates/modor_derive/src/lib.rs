//! Procedural macros of modor.

use proc_macro::TokenStream;
use proc_macro2::{Ident, Literal, TokenTree};
use proc_macro_error::abort;
use quote::quote;
use syn::spanned::Spanned;
use syn::{parse_macro_input, Data, DeriveInput, Fields, ItemImpl};

mod attributes;
mod idents;
mod impl_block;
mod systems;

#[allow(missing_docs)]
#[proc_macro_attribute]
#[proc_macro_error::proc_macro_error]
pub fn entity(_attr: TokenStream, item: TokenStream) -> TokenStream {
    implement_entity_main_component(item, ObjectType::Entity)
}

#[allow(missing_docs)]
#[proc_macro_attribute]
#[proc_macro_error::proc_macro_error]
pub fn singleton(_attr: TokenStream, item: TokenStream) -> TokenStream {
    implement_entity_main_component(item, ObjectType::Singleton)
}

#[allow(missing_docs)]
#[proc_macro_attribute]
#[proc_macro_error::proc_macro_error]
pub fn component(_attr: TokenStream, item: TokenStream) -> TokenStream {
    implement_entity_main_component(item, ObjectType::Component)
}

#[allow(missing_docs)]
#[proc_macro_derive(Component)]
#[proc_macro_error::proc_macro_error]
pub fn component_derive(item: TokenStream) -> TokenStream {
    let item = parse_macro_input!(item as DeriveInput);
    let crate_ident = idents::find_crate_ident(item.span());
    let ident = &item.ident;
    let action_type_ident = Ident::new(&(ident.to_string() + "Action"), item.span());
    let (impl_generics, type_generics, where_clause) = item.generics.split_for_impl();
    let finish_system_call = finish_system_call(ident);
    let output = quote! {
        impl #impl_generics #crate_ident::Component for #ident #type_generics #where_clause {
            type IsEntityMainComponent = #crate_ident::False;
            type Action = #action_type_ident;

            fn on_update(runner: #crate_ident::SystemRunner<'_>) -> #crate_ident::FinishedSystemRunner {
                runner
                #finish_system_call
            }
        }

        #[doc(hidden)]
        #[non_exhaustive]
        #[derive(#crate_ident::Action)]
        pub struct #action_type_ident;
    };
    output.into()
}

#[allow(missing_docs)]
#[proc_macro_derive(Action)]
#[proc_macro_error::proc_macro_error]
pub fn action_derive(item: TokenStream) -> TokenStream {
    let item = parse_macro_input!(item as DeriveInput);
    let crate_ident = idents::find_crate_ident(item.span());
    let ident = &item.ident;
    let (impl_generics, type_generics, where_clause) = item.generics.split_for_impl();
    let dependencies: Vec<_> = match &item.data {
        Data::Struct(struct_) => match &struct_.fields {
            Fields::Unnamed(fields) => fields.unnamed.iter().map(|f| &f.ty).collect(),
            Fields::Unit => vec![],
            Fields::Named(_) => abort!(item, "structs with named fields cannot be actions"),
        },
        Data::Enum(_) | Data::Union(_) => abort!(item.span(), "only structs can be actions"),
    };
    let output = quote! {
        impl #impl_generics #crate_ident::Action for #ident #type_generics #where_clause {
            fn dependency_types() -> ::std::vec::Vec<::std::any::TypeId> {
                let mut types = vec![#(::std::any::TypeId::of::<#dependencies>()),*];
                #(types.extend(<#dependencies as #crate_ident::Action>::dependency_types());)*
                types
            }
        }
    };
    output.into()
}

fn implement_entity_main_component(item: TokenStream, object_type: ObjectType) -> TokenStream {
    let item = parse_macro_input!(item as ItemImpl);
    let crate_ident = idents::find_crate_ident(item.span());
    let cleaned_block = impl_block::clean(&item);
    let type_ = &item.self_ty;
    let type_ident = idents::extract_type_ident(type_);
    let action_type_ident = Ident::new(&(type_ident.to_string() + "Action"), item.span());
    let (impl_generics, _generics, where_clause) = item.generics.split_for_impl();
    let update_statement = systems::generate_update_statement(&item);
    let finish_system_call = finish_system_call(&type_ident);
    let actions = systems::action_dependencies(&item);
    let entity_impl = match object_type {
        ObjectType::Entity => Some(
            quote! {impl #impl_generics #crate_ident::EntityMainComponent for #type_ #where_clause {
                type IsSingleton = #crate_ident::False;
            }},
        ),
        ObjectType::Singleton => Some(
            quote! {impl #impl_generics #crate_ident::EntityMainComponent for #type_ #where_clause {
                type IsSingleton = #crate_ident::True;
            }},
        ),
        ObjectType::Component => None,
    };
    let is_entity_main_component = match object_type {
        ObjectType::Entity | ObjectType::Singleton => quote! {#crate_ident::True},
        ObjectType::Component => quote! {#crate_ident::False},
    };
    let output = quote! {
        #cleaned_block

        impl #impl_generics #crate_ident::Component for #type_ #where_clause {
            type IsEntityMainComponent = #is_entity_main_component;
            type Action = #action_type_ident;

            fn on_update(runner: #crate_ident::SystemRunner<'_>) -> #crate_ident::FinishedSystemRunner {
                #update_statement
                #finish_system_call
            }
        }

        #entity_impl

        #[doc(hidden)]
        #[non_exhaustive]
        #[derive(#crate_ident::Action)]
        pub struct #action_type_ident(#(#actions),*);
    };
    output.into()
}

fn finish_system_call(entity_type: &Ident) -> proc_macro2::TokenStream {
    let label = format!("{entity_type}::modor_finish");
    let label_tokens = TokenTree::Literal(Literal::string(&label));
    quote! {
        .finish(#label_tokens)
    }
}

enum ObjectType {
    Entity,
    Singleton,
    Component,
}
