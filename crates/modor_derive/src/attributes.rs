use proc_macro_error::emit_error;
use syn::Path;
use syn::{Attribute, Meta, NestedMeta};

pub(crate) const RUN_ATTRIBUTE_NAME: &str = "run";
pub(crate) const RUN_AS_ATTRIBUTE_NAME: &str = "run_as";
pub(crate) const RUN_AFTER_ATTRIBUTE_NAME: &str = "run_after";
pub(crate) const RUN_AFTER_PREVIOUS_ATTRIBUTE_NAME: &str = "run_after_previous";

const SUPPORTED_ATTRIBUTE_NAMES: [&str; 4] = [
    RUN_ATTRIBUTE_NAME,
    RUN_AS_ATTRIBUTE_NAME,
    RUN_AFTER_ATTRIBUTE_NAME,
    RUN_AFTER_PREVIOUS_ATTRIBUTE_NAME,
];

pub(crate) enum AttributeType {
    Run,
    RunAs(Path),
    RunAfter(Vec<Path>),
    RunAfterPrevious,
}

pub(crate) fn is_supported(attribute: &Attribute) -> bool {
    if attribute.path.segments.len() > 1 {
        return false;
    }
    SUPPORTED_ATTRIBUTE_NAMES
        .iter()
        .any(|n| attribute.path.segments[0].ident == n)
}

pub(crate) fn parse(attribute: &Attribute) -> Option<AttributeType> {
    if attribute.path.segments.len() > 1 {
        return None;
    }
    return match attribute.path.segments[0].ident.to_string().as_str() {
        RUN_ATTRIBUTE_NAME => parse_run(attribute).or_else(|| {
            emit_error!(attribute, "expected syntax: `#[run]`");
            None
        }),
        RUN_AS_ATTRIBUTE_NAME => parse_run_as(attribute).or_else(|| {
            emit_error!(attribute, "expected syntax: `#[run_as(ActionType)]`");
            None
        }),
        RUN_AFTER_ATTRIBUTE_NAME => parse_run_after(attribute).or_else(|| {
            emit_error!(
                attribute,
                "expected syntax: `#[run_after(ActionType1, ActionType2, ...)]`"
            );
            None
        }),
        RUN_AFTER_PREVIOUS_ATTRIBUTE_NAME => parse_run_after_previous(attribute).or_else(|| {
            emit_error!(attribute, "expected syntax: `#[run_after_previous]`");
            None
        }),
        _ => None,
    };
}

fn parse_run(attribute: &Attribute) -> Option<AttributeType> {
    match attribute.parse_meta().ok()? {
        Meta::Path(_) => Some(AttributeType::Run),
        Meta::List(_) | Meta::NameValue(_) => None,
    }
}

fn parse_run_as(attribute: &Attribute) -> Option<AttributeType> {
    match attribute.parse_meta().ok()? {
        Meta::List(list) => (list.nested.len() == 1)
            .then(|| match &list.nested[0] {
                NestedMeta::Meta(Meta::Path(path)) => Some(AttributeType::RunAs(path.clone())),
                NestedMeta::Meta(_) | NestedMeta::Lit(_) => None,
            })
            .flatten(),
        Meta::Path(_) | Meta::NameValue(_) => None,
    }
}

fn parse_run_after(attribute: &Attribute) -> Option<AttributeType> {
    match attribute.parse_meta().ok()? {
        Meta::List(list) => Some(AttributeType::RunAfter(
            list.nested
                .iter()
                .filter_map(|n| match &n {
                    NestedMeta::Meta(Meta::Path(path)) => Some(path.clone()),
                    NestedMeta::Meta(_) | NestedMeta::Lit(_) => None,
                })
                .collect(),
        )),
        Meta::Path(_) | Meta::NameValue(_) => None,
    }
}

fn parse_run_after_previous(attribute: &Attribute) -> Option<AttributeType> {
    match attribute.parse_meta().ok()? {
        Meta::Path(_) => Some(AttributeType::RunAfterPrevious),
        Meta::List(_) | Meta::NameValue(_) => None,
    }
}
