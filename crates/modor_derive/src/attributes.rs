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
    RunAfterPreviousAnd(Attribute),
}

impl AttributeType {
    pub(crate) fn span(&self) -> Span {
        match self {
            Self::Run(attribute)
            | Self::RunAs(attribute)
            | Self::RunAfter(attribute)
            | Self::RunAfterPrevious(attribute)
            | Self::RunAfterPreviousAnd(attribute) => attribute.path.span(),
        }
    }

    fn expected_syntax(&self) -> &'static str {
        match self {
            Self::Run(_) => "#[run]",
            Self::RunAs(_) => {
                "expected syntax: `#[run_as(ActionType)]` or `#[run_as(component(ComponentType))]`"
            }
            Self::RunAfter(_) => {
                "#[run_after(ActionType1, ActionType2, component(ComponentType), ...)]`"
            }
            Self::RunAfterPrevious(_) => "#[run_after_previous]",
            Self::RunAfterPreviousAnd(_) => {
                "#[run_after_previous_and(ActionType1, ActionType2, component(ComponentType), ...)]"
            }
        }
    }
}

pub(crate) enum ParsedAttribute {
    Run,
    RunAs(TokenStream),
    RunAfter(Vec<TokenStream>),
    RunAfterPrevious,
    RunAfterPreviousAnd(Vec<TokenStream>),
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
        "run_after_previous_and" => Some(AttributeType::RunAfterPreviousAnd(attribute.clone())),
        _ => None,
    }
}

pub(crate) fn parse(attribute: &AttributeType) -> Option<ParsedAttribute> {
    match attribute {
        AttributeType::Run(attribute) => {
            parse_no_argument(attribute).map(|()| ParsedAttribute::Run)
        }
        AttributeType::RunAs(attribute) => {
            parse_path_argument(attribute).map(ParsedAttribute::RunAs)
        }
        AttributeType::RunAfter(attribute) => {
            parse_path_arguments(attribute).map(ParsedAttribute::RunAfter)
        }
        AttributeType::RunAfterPrevious(attribute) => {
            parse_no_argument(attribute).map(|()| ParsedAttribute::RunAfterPrevious)
        }
        AttributeType::RunAfterPreviousAnd(attribute) => {
            parse_path_arguments(attribute).map(ParsedAttribute::RunAfterPreviousAnd)
        }
    }
    .or_else(|| {
        emit_error!(
            attribute.span(),
            "expected syntax: `{}`",
            attribute.expected_syntax()
        );
        None
    })
}

fn parse_no_argument(attribute: &Attribute) -> Option<()> {
    match attribute.parse_meta().ok()? {
        Meta::Path(_) => Some(()),
        Meta::List(_) | Meta::NameValue(_) => None,
    }
}

fn parse_path_argument(attribute: &Attribute) -> Option<TokenStream> {
    match attribute.parse_meta().ok()? {
        Meta::List(list) => (list.nested.len() == 1)
            .then(|| match &list.nested[0] {
                NestedMeta::Meta(Meta::Path(path)) => Some(path.to_token_stream()),
                NestedMeta::Meta(Meta::List(list)) => parse_component_meta(list),
                NestedMeta::Meta(_) | NestedMeta::Lit(_) => None,
            })
            .flatten(),
        Meta::Path(_) | Meta::NameValue(_) => None,
    }
}

fn parse_path_arguments(attribute: &Attribute) -> Option<Vec<TokenStream>> {
    match attribute.parse_meta().ok()? {
        Meta::List(list) => Some(
            list.nested
                .iter()
                .map(|n| match &n {
                    NestedMeta::Meta(Meta::Path(path)) => Some(path.to_token_stream()),
                    NestedMeta::Meta(Meta::List(list)) => parse_component_meta(list),
                    NestedMeta::Meta(_) | NestedMeta::Lit(_) => None,
                })
                .collect::<Option<_>>()?,
        ),
        Meta::Path(_) | Meta::NameValue(_) => None,
    }
}

fn parse_component_meta(meta: &MetaList) -> Option<TokenStream> {
    if meta.path.segments.len() != 1 {
        return None;
    }
    if meta.path.segments[0].ident != "component" {
        return None;
    }
    if meta.nested.len() != 1 {
        return None;
    }
    let nested_meta = meta.nested.first()?;
    let crate_ident = idents::find_crate_ident(nested_meta.span());
    match nested_meta {
        NestedMeta::Meta(Meta::Path(path)) => {
            Some(quote! {<#path as #crate_ident::ComponentSystems>::Action})
        }
        NestedMeta::Meta(_) | NestedMeta::Lit(_) => None,
    }
}
