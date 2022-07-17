use crate::attributes::{AttributeType, ParsedAttribute};
use crate::{attributes, crate_name};
use proc_macro2::TokenStream;
use proc_macro_error::emit_error;
use quote::{quote, quote_spanned};
use std::cmp::Ordering;
use syn::{Attribute, ImplItem, ImplItemMethod, ItemImpl};

pub(crate) fn generate_update_statement(impl_block: &ItemImpl) -> TokenStream {
    let system_calls = system_call_iter(impl_block);
    quote! {
        runner #(#system_calls)*
    }
}

fn system_call_iter(impl_block: &ItemImpl) -> impl Iterator<Item = TokenStream> + '_ {
    impl_block
        .items
        .iter()
        .filter_map(|i| {
            if let ImplItem::Method(method) = i {
                let attributes = supported_attributes(&method.attrs);
                return match attributes.len().cmp(&1) {
                    Ordering::Equal => Some(generate_system_call(method, &attributes[0])),
                    Ordering::Less => None,
                    Ordering::Greater => {
                        emit_error!(attributes[1].span(), "found more than one `run*` attribute");
                        None
                    }
                };
            }
            None
        })
        .flatten()
}

fn supported_attributes(attributes: &[Attribute]) -> Vec<AttributeType> {
    attributes
        .iter()
        .filter_map(attributes::parse_type)
        .collect()
}

fn generate_system_call(method: &ImplItemMethod, attribute: &AttributeType) -> Option<TokenStream> {
    let crate_ident = crate_name::find_crate_ident(attribute.span());
    let system_name = &method.sig.ident;
    Some(match attributes::parse(attribute)? {
        ParsedAttribute::Run => quote_spanned! { attribute.span() =>
            .run(#crate_ident::system!(Self::#system_name))
        },
        ParsedAttribute::RunAs(action) => quote_spanned! { attribute.span() =>
            .run_as::<#action>(#crate_ident::system!(Self::#system_name))
        },
        ParsedAttribute::RunAfter(actions) => quote_spanned! { attribute.span() =>
            .run_constrained::<(#(#crate_ident::DependsOn<#actions>,)*)>(
                #crate_ident::system!(Self::#system_name)
            )
        },
        ParsedAttribute::RunAfterPrevious => quote_spanned! { attribute.span() =>
            .and_then(#crate_ident::system!(Self::#system_name))
        },
    })
}
