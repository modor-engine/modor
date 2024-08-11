#![allow(clippy::manual_unwrap_or_default)] // caused by #[darling(default)]

use crate::utils;
use darling::ast::Data;
use darling::{FromDeriveInput, FromField};
use proc_macro2::Span;
use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote, quote_spanned};
use syn::spanned::Spanned;
use syn::{parse_quote, Attribute, DeriveInput, GenericParam, Generics, Type};

pub(crate) fn impl_block(input: &DeriveInput) -> Result<TokenStream, TokenStream> {
    let crate_ident = utils::crate_ident();
    let ident = &input.ident;
    let vis = &input.vis;
    let updater_generics = updater_generics(input);
    let (updater_impl_generics, updater_type_generics, updater_where_clause) =
        updater_generics.split_for_impl();
    let phantom_type = utils::generic_phantom_type(&updater_generics);
    let parsed: ParsedUpdaterStruct = UpdaterStruct::from_derive_input(input)
        .map_err(darling::Error::write_errors)?
        .try_into()?;
    let field_idents = field_idents(&parsed.fields);
    let field_types = field_types(&parsed.fields);
    let updater_fns = all_field_fns(input, &crate_ident, &parsed.fields);
    let updater_ident = format_ident!("{}Updater", ident);
    let updater_doc = format!("An updater for [`{ident}`].");
    Ok(quote! {
        #[doc = #updater_doc]
        #[must_use]
        #vis struct #updater_ident #updater_generics {
            #(#field_idents: ::#crate_ident::Update<'closures, #field_types>,)*
            phantom: #phantom_type,
        }

        #[automatically_derived]
        #[allow(dead_code)]
        impl #updater_impl_generics ::std::default::Default
            for #updater_ident #updater_type_generics #updater_where_clause
        {
            fn default() -> Self {
                Self {
                    #(#field_idents: ::#crate_ident::Update::default(),)*
                    phantom: ::std::marker::PhantomData
                }
            }
        }

        #[automatically_derived]
        #[allow(dead_code)]
        impl #updater_impl_generics #updater_ident #updater_type_generics #updater_where_clause {
            #(#updater_fns)*
        }
    })
}

fn updater_generics(input: &DeriveInput) -> Generics {
    let mut updater_generics = input.generics.clone();
    updater_generics
        .params
        .insert(0, GenericParam::Lifetime(parse_quote! {'closures}));
    updater_generics
}

fn field_idents(fields: &[ParsedUpdaterField]) -> Vec<&Ident> {
    fields
        .iter()
        .filter(|field| field.is_field_method_generated || field.is_for_field_method_generated)
        .map(|field| &field.ident)
        .collect()
}

fn field_types(fields: &[ParsedUpdaterField]) -> Vec<&Type> {
    fields
        .iter()
        .filter(|field| field.is_field_method_generated || field.is_for_field_method_generated)
        .map(|field| &field.type_)
        .collect()
}

fn all_field_fns(
    input: &DeriveInput,
    crate_ident: &Ident,
    fields: &[ParsedUpdaterField],
) -> Vec<TokenStream> {
    fields
        .iter()
        .flat_map(|field| field_fns(input, crate_ident, field))
        .collect()
}

fn field_fns(
    input: &DeriveInput,
    crate_ident: &Ident,
    field: &ParsedUpdaterField,
) -> Vec<TokenStream> {
    let vis = &input.vis;
    let type_ = &field.type_;
    let ident = &field.ident;
    let doc_attrs = &field.doc_attrs;
    let mut fns = vec![];
    if field.is_field_method_generated {
        let doc = format!("Sets the value of `{ident}`.",);
        fns.push(quote_spanned! {
            ident.span() =>
            #[doc=#doc]
            #[doc=""]
            #(#doc_attrs)*
            #vis fn #ident(mut self, #ident: impl ::std::convert::Into<#type_>) -> Self {
                self.#ident = ::#crate_ident::Update::Value(#ident.into());
                self
            }
        });
    }
    if field.is_for_field_method_generated {
        let fn_ident = format_ident!("for_{}", ident);
        let doc = format!("Runs `f` on the current value of `{ident}`.",);
        fns.push(quote_spanned! {
            ident.span() =>
            #[doc=#doc]
            #[doc=""]
            #(#doc_attrs)*
            #vis fn #fn_ident<O>(
                mut self,
                f: impl ::std::ops::FnOnce(&mut #type_) -> O + 'closures
            ) -> Self {
                self.#ident = ::#crate_ident::Update::Closure(Box::new(|field| {
                    f(field);
                }));
                self
            }
        });
    }
    fns
}

#[derive(Debug, FromDeriveInput)]
struct UpdaterStruct {
    data: Data<(), UpdaterField>,
}

#[derive(Debug, FromField)]
#[darling(attributes(updater), forward_attrs(doc))]
struct UpdaterField {
    ident: Option<Ident>,
    ty: Type,
    attrs: Vec<Attribute>,
    #[darling(default)]
    inner_type: bool,
    #[darling(default)]
    field: bool,
    #[darling(default)]
    for_field: bool,
}

#[derive(Debug)]
struct ParsedUpdaterStruct {
    fields: Vec<ParsedUpdaterField>,
}

impl TryFrom<UpdaterStruct> for ParsedUpdaterStruct {
    type Error = TokenStream;

    fn try_from(struct_: UpdaterStruct) -> Result<Self, Self::Error> {
        Ok(Self {
            fields: match struct_.data {
                Data::Enum(_) => Err(utils::error(Span::call_site(), "enums are not supported"))?,
                Data::Struct(data) => data
                    .fields
                    .into_iter()
                    .map(ParsedUpdaterField::try_from)
                    .collect::<Result<Vec<_>, _>>()?,
            },
        })
    }
}

#[derive(Debug)]
struct ParsedUpdaterField {
    ident: Ident,
    type_: Type,
    doc_attrs: Vec<Attribute>,
    is_field_method_generated: bool,
    is_for_field_method_generated: bool,
}

impl TryFrom<UpdaterField> for ParsedUpdaterField {
    type Error = TokenStream;

    fn try_from(field: UpdaterField) -> Result<Self, Self::Error> {
        let ident = Self::parse_ident(field.ident)?;
        Ok(Self {
            ident,
            type_: Self::parse_type(field.inner_type, field.ty)?,
            doc_attrs: Self::parse_doc_attributes(field.attrs),
            is_field_method_generated: field.field,
            is_for_field_method_generated: field.for_field,
        })
    }
}

impl ParsedUpdaterField {
    fn parse_ident(ident: Option<Ident>) -> Result<Ident, TokenStream> {
        ident.ok_or_else(|| {
            utils::error(
                Span::call_site(),
                "only structs with named fields are supported",
            )
        })
    }

    fn parse_type(has_inner_type: bool, type_: Type) -> Result<Type, TokenStream> {
        if has_inner_type {
            let span = type_.span();
            utils::first_generic_type(type_)
                .ok_or_else(|| utils::error(span, "`inner_type` requires one generic type"))
        } else {
            Ok(type_)
        }
    }

    fn parse_doc_attributes(mut attrs: Vec<Attribute>) -> Vec<Attribute> {
        attrs.retain(|attr| attr.path().is_ident("doc"));
        attrs
    }
}

#[cfg(test)]
mod tests {
    use syn::DeriveInput;

    #[test]
    fn derive_enum() -> syn::Result<()> {
        let input = syn::parse_str::<DeriveInput>("enum Test {}")?;
        assert!(super::impl_block(&input).is_err());
        Ok(())
    }

    #[test]
    fn derive_struct_with_unnamed_fields() -> syn::Result<()> {
        let input = syn::parse_str::<DeriveInput>("struct Test(u32);")?;
        assert!(super::impl_block(&input).is_err());
        Ok(())
    }

    #[test]
    fn derive_with_incorrect_inner_type() -> syn::Result<()> {
        let input = syn::parse_str::<DeriveInput>(
            "struct Test { #[updater(inner_type, field)] field: usize }",
        )?;
        assert!(super::impl_block(&input).is_err());
        Ok(())
    }
}
