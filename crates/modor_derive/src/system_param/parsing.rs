use proc_macro2::Ident;
use proc_macro_error::abort;
use syn::{Data, DeriveInput, Field, Fields, FieldsNamed, FieldsUnnamed, Type};

#[derive(Debug)]
pub(super) struct SystemParamStruct {
    pub(super) input: DeriveInput,
    pub(super) fields: SystemParamStructFields,
}

impl SystemParamStruct {
    pub(super) fn from_input(input: &DeriveInput) -> Self {
        Self {
            input: input.clone(),
            fields: match &input.data {
                Data::Struct(data) => SystemParamStructFields::from_fields(&data.fields),
                Data::Enum(data) => {
                    abort!(data.enum_token, "custom system param cannot be an enum")
                }
                Data::Union(data) => {
                    abort!(data.union_token, "custom system param cannot be a union")
                }
            },
        }
    }
}

#[derive(Debug)]
pub(super) enum SystemParamStructFields {
    Named(Vec<SystemParamStructNamedField>),
    Unnamed(Vec<SystemParamStructUnnamedField>),
    Unit,
}

impl SystemParamStructFields {
    fn from_fields(fields: &Fields) -> Self {
        match fields {
            Fields::Named(fields) => Self::from_named_fields(fields),
            Fields::Unnamed(fields) => Self::from_unnamed_fields(fields),
            Fields::Unit => Self::Unit,
        }
    }

    fn from_named_fields(fields: &FieldsNamed) -> Self {
        Self::Named(
            fields
                .named
                .iter()
                .map(SystemParamStructNamedField::from_field)
                .collect(),
        )
    }

    fn from_unnamed_fields(fields: &FieldsUnnamed) -> Self {
        Self::Unnamed(
            fields
                .unnamed
                .iter()
                .map(SystemParamStructUnnamedField::from_field)
                .collect(),
        )
    }
}

#[derive(Debug)]
pub(super) struct SystemParamStructNamedField {
    pub(super) ident: Ident,
    pub(super) type_: Type,
}

impl SystemParamStructNamedField {
    fn from_field(field: &Field) -> Self {
        Self {
            ident: field
                .ident
                .clone()
                .expect("internal error: named field has no name"),
            type_: field.ty.clone(),
        }
    }
}

#[derive(Debug)]
pub(super) struct SystemParamStructUnnamedField {
    pub(super) type_: Type,
}

impl SystemParamStructUnnamedField {
    fn from_field(field: &Field) -> Self {
        Self {
            type_: field.ty.clone(),
        }
    }
}
