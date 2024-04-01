use crate::utils;
use darling::ast::{Data, Fields};
use darling::{FromDeriveInput, FromField, FromVariant};
use proc_macro2::{Ident, TokenStream};
use quote::{quote, quote_spanned, ToTokens};
use syn::spanned::Spanned;
use syn::{DeriveInput, Index, Type};

pub(crate) fn impl_block_without_visit(input: &DeriveInput) -> TokenStream {
    let crate_ident = utils::crate_ident();
    let ident = &input.ident;
    let (impl_generics, type_generics, where_clause) = input.generics.split_for_impl();
    quote! {
        #[automatically_derived]
        impl #impl_generics ::#crate_ident::Visit for #ident #type_generics #where_clause {
            #[inline]
            fn visit(&mut self, ctx: &mut ::#crate_ident::Context<'_>) {}
        }
    }
}

pub(crate) fn impl_block_with_visit(input: &DeriveInput) -> syn::Result<TokenStream> {
    let crate_ident = utils::crate_ident();
    let ident = &input.ident;
    let (impl_generics, type_generics, where_clause) = input.generics.split_for_impl();
    let visit_body: TokenStream = match VisitStruct::from_derive_input(input)?.data {
        Data::Enum(variants) => enum_visit_body(&crate_ident, ident, variants),
        Data::Struct(fields) => struct_visit_body(&crate_ident, fields),
    };
    Ok(quote! {
        #[automatically_derived]
        impl #impl_generics ::#crate_ident::Visit for #ident #type_generics #where_clause {
            #[inline]
            fn visit(&mut self, ctx: &mut ::#crate_ident::Context<'_>) {
                #visit_body
            }
        }
    })
}

fn enum_visit_body(crate_ident: &Ident, ident: &Ident, variants: Vec<VisitVariant>) -> TokenStream {
    let match_branches: TokenStream = variants
        .into_iter()
        .filter(|variant| !variant.skip)
        .map(|variant| {
            let variant_ident = variant.ident;
            quote_spanned! {
            variant_ident.span() =>
            #ident::#variant_ident(node) => ::#crate_ident::Node::update(node, ctx),
            }
        })
        .collect();
    quote! {
        match self {
            #match_branches
            _ => (),
        }
    }
}

fn struct_visit_body(crate_ident: &Ident, fields: Fields<VisitField>) -> TokenStream {
    fields
        .into_iter()
        .enumerate()
        .filter(|(_, field)| !field.skip)
        .map(|(index, field)| {
            (
                field.ident.map_or_else(
                    || Index::from(index).into_token_stream(),
                    ToTokens::into_token_stream,
                ),
                field.ty,
            )
        })
        .map(|(field_ident, field_type)| {
            quote_spanned! {
                field_type.span() =>
                ::#crate_ident::Node::update(&mut self.#field_ident, ctx);
            }
        })
        .collect()
}

#[derive(FromDeriveInput)]
struct VisitStruct {
    data: Data<VisitVariant, VisitField>,
}

#[derive(FromField)]
#[darling(attributes(modor))]
struct VisitField {
    ident: Option<Ident>,
    ty: Type,
    #[darling(default)]
    // coverage: off (incorrect coverage result)
    skip: bool,
    // coverage: on
}

#[derive(FromVariant)]
#[darling(attributes(modor))]
struct VisitVariant {
    ident: Ident,
    #[darling(default)]
    // coverage: off (incorrect coverage result)
    skip: bool,
    // coverage: on
}
