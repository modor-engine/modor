use proc_macro2::Span;
use proc_macro_error::emit_error;
use syn::spanned::Spanned;
use syn::Path;
use syn::{Attribute, Meta, NestedMeta};

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
    RunAs(Path),
    RunAfter(Vec<Path>),
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
            emit_error!(attribute, "expected syntax: `#[run_as(ActionType)]`");
            None
        }),
        AttributeType::RunAfter(attribute) => parse_run_after(attribute).or_else(|| {
            emit_error!(
                attribute,
                "expected syntax: `#[run_after(ActionType1, ActionType2, ...)]`"
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
                NestedMeta::Meta(Meta::Path(path)) => Some(ParsedAttribute::RunAs(path.clone())),
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
                    NestedMeta::Meta(Meta::Path(path)) => Some(path.clone()),
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
