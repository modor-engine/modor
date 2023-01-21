//! Procedural macros of modor.

use proc_macro::TokenStream;
use proc_macro2::Ident;
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
    implement_entity_main_component(item, false)
}

#[allow(missing_docs)]
#[proc_macro_attribute]
#[proc_macro_error::proc_macro_error]
pub fn singleton(_attr: TokenStream, item: TokenStream) -> TokenStream {
    implement_entity_main_component(item, true)
}

#[allow(missing_docs)]
#[proc_macro_derive(Component)]
#[proc_macro_error::proc_macro_error]
pub fn component_derive(item: TokenStream) -> TokenStream {
    let item = parse_macro_input!(item as DeriveInput);
    let crate_ident = idents::find_crate_ident(item.span());
    let ident = &item.ident;
    let (impl_generics, type_generics, where_clause) = item.generics.split_for_impl();
    let output = quote! {
        impl #impl_generics #crate_ident::Component for #ident #type_generics #where_clause {
            type IsEntityMainComponent = #crate_ident::False;
        }
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

fn implement_entity_main_component(item: TokenStream, is_singleton: bool) -> TokenStream {
    let item = parse_macro_input!(item as ItemImpl);
    let crate_ident = idents::find_crate_ident(item.span());
    let cleaned_block = impl_block::clean(&item);
    let type_ = &item.self_ty;
    let type_ident = idents::extract_type_ident(type_);
    let action_type_ident = Ident::new(&(type_ident.to_string() + "Action"), item.span());
    let (impl_generics, _generics, where_clause) = item.generics.split_for_impl();
    let entity_type = if is_singleton {
        quote!(#crate_ident::True)
    } else {
        quote!(#crate_ident::False)
    };
    let update_statement = systems::generate_update_statement(&item);
    let actions = systems::entity_action_dependencies(&item);
    let output = quote! {
        #cleaned_block

        impl #impl_generics #crate_ident::EntityMainComponent for #type_ #where_clause {
            type IsSingleton = #entity_type;
            type Action = #action_type_ident;

            fn on_update(runner: #crate_ident::SystemRunner<'_>) -> #crate_ident::FinishedSystemRunner {
                #update_statement
            }
        }

        #[doc(hidden)]
        #[non_exhaustive]
        #[derive(#crate_ident::Action)]
        pub struct #action_type_ident(#(#actions),*);
    };
    output.into()
}
