use crate::idents;
use proc_macro2::{Span, TokenStream};
use proc_macro_error::emit_error;
use quote::{quote, ToTokens};
use syn::spanned::Spanned;
use syn::{Attribute, Meta, MetaList, NestedMeta};

pub(crate) enum AttributeType {
    Run(Attribute),
    RunAs(Attribute),
    RunAfter(Attribute),
    RunAfterPrevious(Attribute),
}

impl AttributeType {
    pub(crate) fn span(&self) -> Span {
        match self {
            Self::Run(attribute)
            | Self::RunAs(attribute)
            | Self::RunAfter(attribute)
            | Self::RunAfterPrevious(attribute) => attribute.path.span(),
        }
    }
}

pub(crate) enum ParsedAttribute {
    Run,
    RunAs(TokenStream),
    RunAfter(Vec<TokenStream>),
    RunAfterPrevious,
}

pub(crate) fn parse_type(attribute: &Attribute) -> Option<AttributeType> {
    if attribute.path.segments.len() > 1 {
        return None;
    }
    let identifier: &str = &attribute.path.segments[0].ident.to_string();
    match identifier {
        "run" => Some(AttributeType::Run(attribute.clone())),
        "run_as" => Some(AttributeType::RunAs(attribute.clone())),
        "run_after" => Some(AttributeType::RunAfter(attribute.clone())),
        "run_after_previous" => Some(AttributeType::RunAfterPrevious(attribute.clone())),
        _ => None,
    }
}

pub(crate) fn parse(attribute: &AttributeType) -> Option<ParsedAttribute> {
    match attribute {
        AttributeType::Run(attribute) => parse_run(attribute).or_else(|| {
            emit_error!(attribute, "expected syntax: `#[run]`");
            None
        }),
        AttributeType::RunAs(attribute) => parse_run_as(attribute).or_else(|| {
            emit_error!(
                attribute,
                "expected syntax: `#[run_as(ActionType)]` or `#[run_as(entity(EntityType))]`"
            );
            None
        }),
        AttributeType::RunAfter(attribute) => parse_run_after(attribute).or_else(|| {
            emit_error!(
                attribute,
                "expected syntax: `#[run_after(ActionType1, ActionType2, entity(EntityType), ...)]`"
            );
            None
        }),
        AttributeType::RunAfterPrevious(attribute) => {
            parse_run_after_previous(attribute).or_else(|| {
                emit_error!(attribute, "expected syntax: `#[run_after_previous]`");
                None
            })
        }
    }
}

fn parse_run(attribute: &Attribute) -> Option<ParsedAttribute> {
    match attribute.parse_meta().ok()? {
        Meta::Path(_) => Some(ParsedAttribute::Run),
        Meta::List(_) | Meta::NameValue(_) => None,
    }
}

fn parse_run_as(attribute: &Attribute) -> Option<ParsedAttribute> {
    match attribute.parse_meta().ok()? {
        Meta::List(list) => (list.nested.len() == 1)
            .then(|| match &list.nested[0] {
                NestedMeta::Meta(Meta::Path(path)) => {
                    Some(ParsedAttribute::RunAs(path.to_token_stream()))
                }
                NestedMeta::Meta(Meta::List(list)) => {
                    parse_entity_meta(list).map(ParsedAttribute::RunAs)
                }
                NestedMeta::Meta(_) | NestedMeta::Lit(_) => None,
            })
            .flatten(),
        Meta::Path(_) | Meta::NameValue(_) => None,
    }
}

fn parse_run_after(attribute: &Attribute) -> Option<ParsedAttribute> {
    match attribute.parse_meta().ok()? {
        Meta::List(list) => Some(ParsedAttribute::RunAfter(
            list.nested
                .iter()
                .map(|n| match &n {
                    NestedMeta::Meta(Meta::Path(path)) => Some(path.to_token_stream()),
                    NestedMeta::Meta(Meta::List(list)) => parse_entity_meta(list),
                    NestedMeta::Meta(_) | NestedMeta::Lit(_) => None,
                })
                .collect::<Option<_>>()?,
        )),
        Meta::Path(_) | Meta::NameValue(_) => None,
    }
}

fn parse_run_after_previous(attribute: &Attribute) -> Option<ParsedAttribute> {
    match attribute.parse_meta().ok()? {
        Meta::Path(_) => Some(ParsedAttribute::RunAfterPrevious),
        Meta::List(_) | Meta::NameValue(_) => None,
    }
}

fn parse_entity_meta(meta: &MetaList) -> Option<TokenStream> {
    if meta.path.segments.len() != 1 {
        return None;
    }
    if meta.path.segments[0].ident != "entity" {
        return None;
    }
    if meta.nested.len() != 1 {
        return None;
    }
    let Some(nested_meta) = meta.nested.first() else { return None };
    let crate_ident = idents::find_crate_ident(nested_meta.span());
    match nested_meta {
        NestedMeta::Meta(Meta::Path(path)) => {
            Some(quote! {<#path as #crate_ident::EntityMainComponent>::Action})
        }
        NestedMeta::Meta(_) | NestedMeta::Lit(_) => None,
    }
}
