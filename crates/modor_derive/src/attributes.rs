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
            AttributeType::Run(_) => "#[run]",
            AttributeType::RunAs(_) => "#[run_as(ActionType)]",
            AttributeType::RunAfter(_) => "#[run_after(ActionType1, ActionType2, ...)]",
            AttributeType::RunAfterPrevious(_) => "#[run_after_previous]",
            AttributeType::RunAfterPreviousAnd(_) => {
                "#[run_after_previous_and(ActionType1, ActionType2, ...)]"
            }
        }
    }
}

pub(crate) enum ParsedAttribute {
    Run,
    RunAs(Path),
    RunAfter(Vec<Path>),
    RunAfterPrevious,
    RunAfterPreviousAnd(Vec<Path>),
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

fn parse_path_argument(attribute: &Attribute) -> Option<Path> {
    match attribute.parse_meta().ok()? {
        Meta::List(list) => (list.nested.len() == 1)
            .then(|| match &list.nested[0] {
                NestedMeta::Meta(Meta::Path(path)) => Some(path.clone()),
                NestedMeta::Meta(_) | NestedMeta::Lit(_) => None,
            })
            .flatten(),
        Meta::Path(_) | Meta::NameValue(_) => None,
    }
}

fn parse_path_arguments(attribute: &Attribute) -> Option<Vec<Path>> {
    match attribute.parse_meta().ok()? {
        Meta::List(list) => Some(
            list.nested
                .iter()
                .map(|n| match &n {
                    NestedMeta::Meta(Meta::Path(path)) => Some(path.clone()),
                    NestedMeta::Meta(_) | NestedMeta::Lit(_) => None,
                })
                .collect::<Option<_>>()?,
        ),
        Meta::Path(_) | Meta::NameValue(_) => None,
    }
}
