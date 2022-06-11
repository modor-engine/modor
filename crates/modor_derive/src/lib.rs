//! Procedural macros of modor.

use proc_macro::TokenStream;
use proc_macro2::Ident;
use proc_macro_error::ResultExt;
use quote::quote;
use syn::parse::Parser;
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::{parse_macro_input, ItemImpl, ItemStruct, Token};

mod attributes;
mod crate_name;
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
#[proc_macro_attribute]
#[proc_macro_error::proc_macro_error]
pub fn action(attr: TokenStream, item: TokenStream) -> TokenStream {
    let item = parse_macro_input!(item as ItemStruct);
    let crate_ident = crate_name::find_crate_ident(item.span());
    let type_name = &item.ident;
    let actions: Vec<_> = Punctuated::<Ident, Token![,]>::parse_terminated
        .parse(attr)
        .unwrap_or_abort()
        .into_iter()
        .collect();
    let output = quote! {
        #item

        impl #crate_ident::Action for #type_name {
            type Constraint = (#(#crate_ident::DependsOn<#actions>,)*);
        }
    };
    output.into()
}

fn implement_entity_main_component(item: TokenStream, is_singleton: bool) -> TokenStream {
    let item = parse_macro_input!(item as ItemImpl);
    let crate_ident = crate_name::find_crate_ident(item.span());
    let cleaned_block = impl_block::clean(&item);
    let type_name = &item.self_ty;
    let entity_type = if is_singleton {
        quote!(#crate_ident::Singleton)
    } else {
        quote!(#crate_ident::NotSingleton)
    };
    let update_statement = systems::generate_update_statement(&item);
    let output = quote! {
        #cleaned_block

        impl #crate_ident::EntityMainComponent for #type_name {
            type Type = #entity_type;

            fn on_update(runner: #crate_ident::SystemRunner<'_>) -> #crate_ident::SystemRunner<'_> {
                #update_statement
            }
        }
    };
    output.into()
}
