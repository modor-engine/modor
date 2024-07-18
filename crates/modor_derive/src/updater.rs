#![allow(clippy::manual_unwrap_or_default)] // caused by #[darling(default)]

use crate::utils;
use darling::ast::Data;
use darling::util::SpannedValue;
use darling::{FromDeriveInput, FromField};
use proc_macro2::Span;
use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote, quote_spanned};
use syn::spanned::Spanned;
use syn::{
    parse_quote, parse_quote_spanned, Attribute, DeriveInput, Expr, GenericParam, Generics, LitStr,
    Type,
};

// TODO: test all generics cases

pub(crate) fn impl_block(input: &DeriveInput) -> Result<TokenStream, TokenStream> {
    let crate_ident = utils::crate_ident();
    let ident = &input.ident;
    let vis = &input.vis;
    let (impl_generics, type_generics, where_clause) = input.generics.split_for_impl();
    let updater_generics = updater_generics(input);
    let (updater_impl_generics, updater_type_generics, updater_where_clause) =
        updater_generics.split_for_impl();
    let generic_type_idents = utils::generic_type_idents(&updater_generics);
    let parsed: ParsedUpdaterStruct = UpdaterStruct::from_derive_input(input)
        .map_err(darling::Error::write_errors)?
        .try_into()?;
    let field_idents = field_idents(&parsed.fields);
    let field_types = field_types(&parsed.fields);
    let updater_fns = all_field_fns(input, &crate_ident, &parsed.fields);
    let updater_ident = format_ident!("{}Updater", ident);
    Ok(quote! {
        #[automatically_derived]
        impl #impl_generics ::modor::Updater for #ident #type_generics #where_clause {
            type Updater<'glob> = #updater_ident #updater_type_generics;

            fn updater(glob: &::modor::Glob<Self>) -> Self::Updater<'_> {
                #updater_ident {
                    glob,
                    #(#field_idents: None,)*
                    phantom: ::std::marker::PhantomData,
                }
            }
        }

        #[must_use]
        #vis struct #updater_ident #updater_generics {
            glob: &'glob ::modor::Glob<#ident #type_generics>,
            #(#field_idents: Option<#field_types>,)*
            phantom: PhantomData<(#(#generic_type_idents,)*)>,
        }

        #[automatically_derived]
        #[allow(dead_code, clippy::clone_on_copy)]
        impl #updater_impl_generics #updater_ident #updater_type_generics #updater_where_clause {
            #(#updater_fns)*
        }
    })
}

fn updater_generics(input: &DeriveInput) -> Generics {
    let mut updater_generics = input.generics.clone();
    updater_generics
        .params
        .insert(0, GenericParam::Lifetime(parse_quote! {'glob}));
    updater_generics
}

fn field_idents(fields: &[ParsedUpdaterField]) -> Vec<&Ident> {
    fields
        .iter()
        .filter(|field| field.is_field_method_generated || field.for_field_method_getter.is_some())
        .map(|field| &field.ident)
        .collect()
}

fn field_types(fields: &[ParsedUpdaterField]) -> Vec<&Type> {
    fields
        .iter()
        .filter(|field| field.is_field_method_generated || field.for_field_method_getter.is_some())
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
    let type_ident = &input.ident;
    let (_, type_generics, _) = input.generics.split_for_impl();
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
                self.#ident = Some(#ident.into());
                self
            }
        });
    }
    if let Some(getter) = &field.for_field_method_getter {
        let fn_ident = format_ident!("for_{}", ident);
        let doc = format!("Runs `f` on the current value of `{ident}`.",);
        fns.push(quote_spanned! {
            ident.span() =>
            #[doc=#doc]
            #[doc=""]
            #(#doc_attrs)*
            #vis fn #fn_ident<O>(
                mut self,
                app: &::#crate_ident::App,
                f: impl ::std::ops::FnOnce(&mut #type_) -> O
            ) -> Self {
                let getter: fn(&#type_ident #type_generics, &::modor::App) -> _ = #getter;
                f(self
                    .#ident
                    .get_or_insert_with(|| getter(self.glob.get(app), app)));
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
    for_field: Option<SpannedValue<String>>,
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
    for_field_method_getter: Option<Expr>,
}

impl TryFrom<UpdaterField> for ParsedUpdaterField {
    type Error = TokenStream;

    fn try_from(field: UpdaterField) -> Result<Self, Self::Error> {
        let ident = Self::parse_ident(field.ident)?;
        Ok(Self {
            for_field_method_getter: Self::parse_for_field_method_getter(&ident, field.for_field)?,
            ident,
            type_: Self::parse_type(field.inner_type, field.ty)?,
            doc_attrs: Self::parse_doc_attributes(field.attrs),
            is_field_method_generated: field.field,
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

    fn parse_for_field_method_getter(
        ident: &Ident,
        getter: Option<SpannedValue<String>>,
    ) -> Result<Option<Expr>, TokenStream> {
        Ok(if let Some(getter) = getter {
            Some(if &**getter == "default" {
                parse_quote_spanned! {ident.span() => |value, _| value.#ident.clone()}
            } else {
                LitStr::new(&getter, getter.span())
                    .parse()
                    .map_err(|err| utils::error(getter.span(), &format!("{err}")))?
            })
        } else {
            None
        })
    }
}
