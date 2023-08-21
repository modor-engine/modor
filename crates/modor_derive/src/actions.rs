use crate::{common, idents};
use proc_macro2::{Ident, Span, TokenStream};
use proc_macro_error::abort;
use quote::quote;
use syn::spanned::Spanned;
use syn::{parse_quote, parse_quote_spanned, Data, DataStruct, DeriveInput, Expr, Field};

pub(crate) struct ActionStruct<'a> {
    crate_name: String,
    input: &'a DeriveInput,
    data: &'a DataStruct,
}

impl<'a> ActionStruct<'a> {
    pub(crate) fn new(input: &'a DeriveInput) -> Self {
        Self {
            crate_name: idents::crate_name(),
            input,
            data: match &input.data {
                Data::Struct(fields) => fields,
                Data::Enum(data) => {
                    abort!(data.enum_token, "action cannot be an enum")
                }
                Data::Union(data) => {
                    abort!(data.union_token, "action cannot be a union")
                }
            },
        }
    }

    pub(crate) fn action_impl(&self) -> TokenStream {
        let crate_ = Ident::new(&self.crate_name, Span::call_site());
        let impl_header = common::impl_header(
            &self.input.generics,
            &self.input.ident,
            &parse_quote! { #crate_::Action },
        );
        let type_ids = self.dependency_type_ids();
        let dependency_types = self.dependencies_inner_types();
        quote! {
            #impl_header {
                fn dependency_types() -> ::std::vec::Vec<::std::any::TypeId> {
                    let mut types = vec![#(#type_ids),*];
                    #(types.extend(#dependency_types);)*
                    types
                }
            }
        }
    }

    #[allow(clippy::redundant_closure)]
    fn dependency_type_ids(&self) -> impl Iterator<Item = Expr> + '_ {
        self.data.fields.iter().map(|f| Self::dependency_type_id(f))
    }

    fn dependency_type_id(field: &Field) -> Expr {
        let span = field.span();
        let type_ = &field.ty;
        parse_quote_spanned! { span => ::std::any::TypeId::of::<#type_>() }
    }

    fn dependencies_inner_types(&self) -> impl Iterator<Item = Expr> + '_ {
        self.data
            .fields
            .iter()
            .map(|f| self.dependency_inner_types(f))
    }

    fn dependency_inner_types(&self, field: &Field) -> Expr {
        let span = field.span();
        let crate_ = Ident::new(&self.crate_name, span);
        let type_ = &field.ty;
        parse_quote_spanned! { span => <#type_ as #crate_::Action>::dependency_types() }
    }
}
