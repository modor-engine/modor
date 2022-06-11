use crate::attributes;
use crate::attributes::{AttributeType, ParsedAttribute};
use proc_macro2::{Ident, TokenStream};
use proc_macro_error::emit_error;
use quote::{quote, quote_spanned};
use std::cmp::Ordering;
use syn::{Attribute, ImplItem, ImplItemMethod, ItemImpl};

pub(crate) fn generate_update_statement(impl_block: &ItemImpl, crate_ident: &Ident) -> TokenStream {
    let system_calls = system_call_iter(impl_block, crate_ident);
    quote! {
        runner #(#system_calls)*
    }
}

fn system_call_iter<'a>(
    impl_block: &'a ItemImpl,
    crate_ident: &'a Ident,
) -> impl Iterator<Item = TokenStream> + 'a {
    impl_block
        .items
        .iter()
        .filter_map(|i| {
            if let ImplItem::Method(method) = i {
                let attributes = supported_attributes(&method.attrs);
                return match attributes.len().cmp(&1) {
                    Ordering::Equal => {
                        Some(generate_system_call(method, &attributes[0], crate_ident))
                    }
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

fn generate_system_call(
    method: &ImplItemMethod,
    attribute: &AttributeType,
    crate_ident: &Ident,
) -> Option<TokenStream> {
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
