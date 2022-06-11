//! Procedural macros of modor.

use proc_macro::TokenStream;
use proc_macro2::{Ident, Span};
use proc_macro_crate::FoundCrate;
use proc_macro_error::ResultExt;
use quote::quote;
use syn::parse::Parser;
use syn::punctuated::Punctuated;
use syn::{parse_macro_input, ItemImpl, ItemStruct, Token};

mod attributes;
mod impl_block;
mod systems;

const CORE_CRATE_NAME: &str = "modor";

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
    let crate_ident = find_crate_ident();
    let item = parse_macro_input!(item as ItemStruct);
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
    let crate_ident = find_crate_ident();
    let item = parse_macro_input!(item as ItemImpl);
    let cleaned_block = impl_block::clean(&item);
    let type_name = &item.self_ty;
    let entity_type = if is_singleton {
        quote!(#crate_ident::Singleton)
    } else {
        quote!(#crate_ident::NotSingleton)
    };
    let update_statement = systems::generate_update_statement(&item, &crate_ident);
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

fn find_crate_ident() -> Ident {
    match proc_macro_crate::crate_name(CORE_CRATE_NAME) {
        Ok(FoundCrate::Itself) | Err(_) => Ident::new(CORE_CRATE_NAME, Span::call_site()),
        Ok(FoundCrate::Name(name)) => Ident::new(&name, Span::call_site()),
    }
}
