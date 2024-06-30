#![allow(clippy::manual_unwrap_or_default)] // caused by #[darling(default)]

use darling::ast::Data;
use darling::{FromDeriveInput, FromField, FromMeta};
use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote, quote_spanned};
use syn::__private::Span;
use syn::{DeriveInput, Type, Visibility};

use crate::utils;

pub(crate) fn impl_block(input: &DeriveInput) -> Result<TokenStream, TokenStream> {
    let ident = &input.ident;
    let (impl_generics, type_generics, where_clause) = input.generics.split_for_impl();
    let parsed = BuilderStruct::from_derive_input(input).map_err(darling::Error::write_errors)?;
    let builder_fns = builder_fns(&parsed).ok_or_else(|| {
        utils::error(
            Span::call_site(),
            "only structs with named fields are supported",
        )
    })?;
    Ok(quote! {
        #[automatically_derived]
        impl #impl_generics #ident #type_generics #where_clause {
            #(#builder_fns)*
        }
    })
}

fn builder_fns(parsed: &BuilderStruct) -> Option<Vec<Option<TokenStream>>> {
    match &parsed.data {
        Data::Enum(_) => None,
        Data::Struct(data) => data
            .fields
            .iter()
            .map(|field| field.ident.as_ref().map(|ident| builder_fn(field, ident)))
            .collect(),
    }
}

fn builder_fn(field: &BuilderField, field_ident: &Ident) -> Option<TokenStream> {
    let vis = &field.vis;
    let type_ = &field.ty;
    let fn_ident = format_ident!("with_{}", field_ident);
    let documentation =
        format!("Returns `self` with a different [`{field_ident}`](#structfield.{field_ident}).",);
    match &field.form {
        None => None,
        Some(BuilderForm::Value) => Some(quote_spanned! {
            field_ident.span() =>
            #[doc=#documentation]
            #[allow(dead_code)]
            #vis fn #fn_ident(mut self, #field_ident: #type_) -> Self {
                self.#field_ident = #field_ident;
                self
            }
        }),
        Some(BuilderForm::Closure) => Some(quote_spanned! {
            field_ident.span() =>
            #[doc=#documentation]
            #[allow(dead_code)]
            #vis fn #fn_ident(mut self, f: impl FnOnce(&mut #type_)) -> Self {
                f(&mut self.#field_ident);
                self
            }
        }),
    }
}

#[derive(Debug, FromDeriveInput)]
struct BuilderStruct {
    data: Data<(), BuilderField>,
}

#[derive(Debug, FromField)]
#[darling(attributes(builder))]
struct BuilderField {
    ident: Option<Ident>,
    ty: Type,
    vis: Visibility,
    #[darling(default)]
    form: Option<BuilderForm>,
}

#[derive(Debug, PartialEq, Eq, FromMeta)]
enum BuilderForm {
    Value,
    Closure,
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
}
