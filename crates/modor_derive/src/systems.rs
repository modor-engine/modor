use crate::attributes::{AttributeType, ParsedAttribute};
use crate::{attributes, idents};
use proc_macro2::{Literal, TokenStream, TokenTree};
use proc_macro_error::emit_error;
use quote::{quote, quote_spanned, ToTokens};
use std::cmp::Ordering;
use syn::{Attribute, ImplItem, ImplItemMethod, ItemImpl};

pub(crate) fn generate_update_statement(impl_block: &ItemImpl) -> TokenStream {
    let system_calls = system_call_iter(impl_block);
    quote! {
        runner #(#system_calls)*
    }
}

pub(crate) fn action_dependencies(impl_block: &ItemImpl) -> Vec<TokenStream> {
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
            Some(
                ParsedAttribute::RunAfter(paths) | ParsedAttribute::RunAfterPreviousAnd(paths),
            ) => paths,
            Some(ParsedAttribute::Run | ParsedAttribute::RunAfterPrevious) | None => vec![],
        })
        .collect()
}

fn system_call_iter(impl_block: &ItemImpl) -> impl Iterator<Item = TokenStream> + '_ {
    let token_stream = impl_block.self_ty.to_token_stream().to_string();
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
}

fn supported_attributes(attributes: &[Attribute]) -> Vec<AttributeType> {
    attributes
        .iter()
        .filter_map(attributes::parse_type)
        .collect()
}

fn generate_system_call(
    type_: &str,
    method: &ImplItemMethod,
    attribute: &AttributeType,
) -> Option<TokenStream> {
    let crate_ident = idents::find_crate_ident(attribute.span());
    let system_name = &method.sig.ident;
    let label = format!("{type_}::{}", method.sig.ident);
    let label_tokens = TokenTree::Literal(Literal::string(&label));
    Some(match attributes::parse(attribute)? {
        ParsedAttribute::Run => quote_spanned! { attribute.span() =>
            .run(#crate_ident::system!(Self::#system_name), #label_tokens)
        },
        ParsedAttribute::RunAs(action) => quote_spanned! { attribute.span() =>
            .run_as::<#action>(#crate_ident::system!(Self::#system_name), #label_tokens)
        },
        ParsedAttribute::RunAfter(actions) => {
            let constraint = create_constraint(actions);
            quote_spanned! { attribute.span() =>
                .run_constrained::<#constraint>(
                    #crate_ident::system!(Self::#system_name),
                    #label_tokens,
                )
            }
        }
        ParsedAttribute::RunAfterPrevious => quote_spanned! { attribute.span() =>
            .and_then::<()>(#crate_ident::system!(Self::#system_name), #label_tokens)
        },
        ParsedAttribute::RunAfterPreviousAnd(actions) => {
            let constraint = create_constraint(actions);
            quote_spanned! { attribute.span() =>
                .and_then::<#constraint>(
                    #crate_ident::system!(Self::#system_name),
                    #label_tokens,
                )
            }
        }
    })
}

fn create_constraint(actions: Vec<TokenStream>) -> TokenStream {
    if actions.is_empty() {
        return quote!(());
    }
    let mut constraint = quote! {};
    for action in actions {
        constraint = quote! { (#action, #constraint) };
    }
    constraint
}
