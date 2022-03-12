//! Procedural macros of modor.

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemImpl};

mod attributes;
mod impl_block;
mod systems;

// TODO: improve doc

/// Defines an entity.
#[proc_macro_attribute]
#[proc_macro_error::proc_macro_error]
pub fn entity(_attr: TokenStream, item: TokenStream) -> TokenStream {
    implement_entity_main_component(item, false)
}

/// Defines a singleton entity.
#[proc_macro_attribute]
#[proc_macro_error::proc_macro_error]
pub fn singleton(_attr: TokenStream, item: TokenStream) -> TokenStream {
    implement_entity_main_component(item, true)
}

fn implement_entity_main_component(item: TokenStream, is_singleton: bool) -> TokenStream {
    let item = parse_macro_input!(item as ItemImpl);
    let cleaned_block = impl_block::clean(&item);
    let type_name = &item.self_ty;
    let entity_type = if is_singleton {
        quote!(::modor::Singleton)
    } else {
        quote!(())
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
