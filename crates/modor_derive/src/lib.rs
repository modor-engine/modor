//! Procedural macros of modor.

use proc_macro::TokenStream;
use proc_macro2::Ident;
use proc_macro_error::ResultExt;
use quote::{quote, ToTokens};
use syn::parse::Parser;
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::{parse_macro_input, ImplGenerics, ItemImpl, ItemStruct, Path, Token, WhereClause};

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
    let generics = &item.generics;
    let (impl_generics, _generics, where_clause) = generics.split_for_impl();
    let actions: Vec<_> = Punctuated::<Path, Token![,]>::parse_terminated
        .parse(attr)
        .unwrap_or_abort()
        .into_iter()
        .collect();
    let impl_block = action_impl_block(
        &crate_ident,
        &item.ident,
        &impl_generics,
        where_clause,
        &actions,
    );
    let output = quote! {
        #item

        #impl_block
    };
    output.into()
}

fn implement_entity_main_component(item: TokenStream, is_singleton: bool) -> TokenStream {
    let item = parse_macro_input!(item as ItemImpl);
    let crate_ident = crate_name::find_crate_ident(item.span());
    let cleaned_block = impl_block::clean(&item);
    let type_name = &item.self_ty;
    let generics = &item.generics;
    let (impl_generics, _generics, where_clause) = generics.split_for_impl();
    let entity_type = if is_singleton {
        quote!(#crate_ident::Singleton)
    } else {
        quote!(#crate_ident::NotSingleton)
    };
    let update_statement = systems::generate_update_statement(&item);
    let actions = systems::entity_action_dependencies(&item);
    let action_impl_block = action_impl_block(
        &crate_ident,
        type_name,
        &impl_generics,
        where_clause,
        &actions,
    );
    let output = quote! {
        #cleaned_block

        impl #impl_generics #crate_ident::EntityMainComponent for #type_name #where_clause {
            type Type = #entity_type;

            fn on_update(runner: #crate_ident::SystemRunner<'_>) -> #crate_ident::FinishedSystemRunner {
                #update_statement
            }
        }

        #action_impl_block
    };
    output.into()
}

fn action_impl_block(
    crate_ident: &Ident,
    item_name: impl ToTokens,
    impl_generics: &ImplGenerics<'_>,
    where_clause: Option<&WhereClause>,
    actions: &[Path],
) -> proc_macro2::TokenStream {
    let mut final_actions = if actions.is_empty() {
        quote!(())
    } else {
        quote!()
    };
    for action in actions {
        final_actions = quote!((#crate_ident::DependsOn<#action>, #final_actions));
    }
    quote! {
        impl #impl_generics #crate_ident::Action for #item_name #where_clause {
            type Constraint = #final_actions;
        }
    }
}
