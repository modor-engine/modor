use crate::attributes;
use crate::attributes::AttributeType;
use proc_macro2::TokenStream;
use proc_macro_error::emit_error;
use quote::{quote, quote_spanned};
use std::cmp::Ordering;
use syn::spanned::Spanned;
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
                return match method.attrs.len().cmp(&1) {
                    Ordering::Equal => Some(generate_system_call(method, &method.attrs[0])),
                    Ordering::Less => None,
                    Ordering::Greater => {
                        emit_error!(method.attrs[1], "found more than one `run*` attribute");
                        None
                    }
                };
            }
            None
        })
        .flatten()
}

fn generate_system_call(method: &ImplItemMethod, attribute: &Attribute) -> Option<TokenStream> {
    let system_name = &method.sig.ident;
    Some(match attributes::parse(attribute)? {
        AttributeType::Run => quote_spanned! { attribute.span() =>
            .run(::modor::system!(Self::#system_name))
        },
        AttributeType::RunAs(action) => quote_spanned! { attribute.span() =>
            .run_as::<#action>(::modor::system!(Self::#system_name))
        },
        AttributeType::RunAfter(actions) => quote_spanned! { attribute.span() =>
            .run_constrained::<(#(::modor::DependsOn<#actions>,)*)>(
                ::modor::system!(Self::#system_name)
            )
        },
        AttributeType::RunAfterPrevious => quote_spanned! { attribute.span() =>
            .and_then(::modor::system!(Self::#system_name))
        },
    })
}
