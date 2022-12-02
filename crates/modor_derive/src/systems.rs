use crate::attributes::{AttributeType, ParsedAttribute};
use crate::{attributes, crate_name};
use proc_macro2::{Literal, TokenStream, TokenTree};
use proc_macro_error::emit_error;
use quote::{quote, quote_spanned, ToTokens};
use std::cmp::Ordering;
use std::iter;
use syn::{Attribute, ImplItem, ImplItemMethod, ItemImpl, Path};

pub(crate) fn generate_update_statement(impl_block: &ItemImpl) -> TokenStream {
    let system_calls = system_call_iter(impl_block);
    quote! {
        runner #(#system_calls)*
    }
}

pub(crate) fn entity_action_dependencies(impl_block: &ItemImpl) -> Vec<Path> {
    impl_block
        .items
        .iter()
        .filter_map(|i| {
            if let ImplItem::Method(method) = i {
                Some(method)
            } else {
                None
            }
        })
        .flat_map(|m| supported_attributes(&m.attrs))
        .flat_map(|a| match attributes::parse(&a) {
            Some(ParsedAttribute::RunAs(path)) => vec![path],
            Some(ParsedAttribute::RunAfter(paths)) => paths,
            Some(ParsedAttribute::Run | ParsedAttribute::RunAfterPrevious) | None => vec![],
        })
        .collect()
}

fn system_call_iter(impl_block: &ItemImpl) -> impl Iterator<Item = TokenStream> + '_ {
    let token_stream = impl_block.self_ty.to_token_stream().to_string();
    let finish_call = finish_system_call(&token_stream);
    impl_block
        .items
        .iter()
        .filter_map(move |i| {
            if let ImplItem::Method(method) = i {
                let attributes = supported_attributes(&method.attrs);
                match attributes.len().cmp(&1) {
                    Ordering::Equal => {
                        Some(generate_system_call(&token_stream, method, &attributes[0]))
                    }
                    Ordering::Less => None,
                    Ordering::Greater => {
                        emit_error!(attributes[1].span(), "found more than one `run*` attribute");
                        None
                    }
                }
            } else {
                None
            }
        })
        .flatten()
        .chain(iter::once(finish_call))
}

fn supported_attributes(attributes: &[Attribute]) -> Vec<AttributeType> {
    attributes
        .iter()
        .filter_map(attributes::parse_type)
        .collect()
}

fn generate_system_call(
    entity_type: &str,
    method: &ImplItemMethod,
    attribute: &AttributeType,
) -> Option<TokenStream> {
    let crate_ident = crate_name::find_crate_ident(attribute.span());
    let system_name = &method.sig.ident;
    let label = format!("{entity_type}::{}", method.sig.ident);
    let label_tokens = TokenTree::Literal(Literal::string(&label));
    Some(match attributes::parse(attribute)? {
        ParsedAttribute::Run => quote_spanned! { attribute.span() =>
            .run(#crate_ident::system!(Self::#system_name), #label_tokens)
        },
        ParsedAttribute::RunAs(action) => quote_spanned! { attribute.span() =>
            .run_as::<#action>(#crate_ident::system!(Self::#system_name), #label_tokens)
        },
        ParsedAttribute::RunAfter(actions) => quote_spanned! { attribute.span() =>
            .run_constrained::<(#(#crate_ident::DependsOn<#actions>,)*)>(
                #crate_ident::system!(Self::#system_name),
                #label_tokens,
            )
        },
        ParsedAttribute::RunAfterPrevious => quote_spanned! { attribute.span() =>
            .and_then(#crate_ident::system!(Self::#system_name), #label_tokens)
        },
    })
}

fn finish_system_call(entity_type: &str) -> TokenStream {
    let label = format!("{entity_type}::{}", "modor_finish");
    let label_tokens = TokenTree::Literal(Literal::string(&label));
    quote! {
        .finish(#label_tokens)
    }
}
