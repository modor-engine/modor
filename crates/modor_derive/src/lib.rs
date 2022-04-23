//! Procedural macros of modor.

use proc_macro::TokenStream;
use proc_macro2::Ident;
use proc_macro_error::ResultExt;
use quote::quote;
use syn::parse::Parser;
use syn::punctuated::Punctuated;
use syn::{parse_macro_input, ItemImpl, ItemStruct, Token};

mod attributes;
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
    let type_name = &item.ident;
    let actions: Vec<_> = Punctuated::<Ident, Token![,]>::parse_terminated
        .parse(attr)
        .unwrap_or_abort()
        .into_iter()
        .collect();
    let output = quote! {
        #item

        impl ::modor::Action for #type_name {
            type Constraint = (#(::modor::DependsOn<#actions>,)*);
        }
    };
    output.into()
}

fn implement_entity_main_component(item: TokenStream, is_singleton: bool) -> TokenStream {
    let item = parse_macro_input!(item as ItemImpl);
    let cleaned_block = impl_block::clean(&item);
    let type_name = &item.self_ty;
    let entity_type = if is_singleton {
        quote!(::modor::Singleton)
    } else {
        quote!(::modor::NotSingleton)
    };
    let update_statement = systems::generate_update_statement(&item);
    let output = quote! {
        #cleaned_block

        impl ::modor::EntityMainComponent for #type_name {
            type Type = #entity_type;

            fn on_update(runner: ::modor::SystemRunner<'_>) -> ::modor::SystemRunner<'_> {
                #update_statement
            }
        }
    };
    output.into()
}

// TODO: add to coverage
// TODO: probably requires compile fail tests (see if it can work on coverage/mutation tests)
