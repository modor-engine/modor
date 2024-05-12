use crate::utils;
use darling::ast::{Data, Fields, Style};
use darling::{FromDeriveInput, FromField, FromVariant};
use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote, quote_spanned, ToTokens};
use syn::spanned::Spanned;
use syn::{DeriveInput, Index, Type};

pub(crate) fn impl_block_with_visit(input: &DeriveInput) -> syn::Result<TokenStream> {
    let crate_ident = utils::crate_ident();
    let ident = &input.ident;
    let (impl_generics, type_generics, where_clause) = input.generics.split_for_impl();
    let visit_body: TokenStream = match VisitStruct::from_derive_input(input)?.data {
        Data::Enum(variants) => enum_visit_body(&crate_ident, variants),
        Data::Struct(fields) => struct_visit_body(&crate_ident, fields),
    };
    Ok(quote! {
        #[automatically_derived]
        impl #impl_generics ::#crate_ident::Visit for #ident #type_generics #where_clause {
            #[inline]
            fn visit(&mut self, ctx: &mut ::#crate_ident::Context<'_>) {
                use ::#crate_ident::macro_utils::NotNode;
                #visit_body
            }
        }
    })
}

fn enum_visit_body(crate_ident: &Ident, variants: Vec<VisitVariant>) -> TokenStream {
    let match_branches: TokenStream = variants
        .into_iter()
        .map(|variant| match variant.fields.style {
            Style::Tuple => enum_tuple_branch_body(crate_ident, &variant),
            Style::Struct => enum_struct_branch_body(crate_ident, &variant),
            Style::Unit => enum_unit_branch_body(&variant),
        })
        .collect();
    quote! {
        match self {
            #match_branches
            _ => (),
        }
    }
}

fn enum_tuple_branch_body(crate_ident: &Ident, variant: &VisitVariant) -> TokenStream {
    let variant_ident = &variant.ident;
    let field_names: Vec<_> = (0..variant.fields.len())
        .map(|i| format_ident!("field{}", i))
        .collect();
    quote_spanned! {
        variant_ident.span() =>
        Self::#variant_ident(#(#field_names),*) => {
            #(::#crate_ident::macro_utils::MaybeNode(#field_names).update(ctx));*
        }
    }
}

fn enum_struct_branch_body(crate_ident: &Ident, variant: &VisitVariant) -> TokenStream {
    let variant_ident = &variant.ident;
    let field_names: Vec<_> = variant
        .fields
        .iter()
        .map(|field| field.ident.as_ref().expect("internal error: missing type"))
        .collect();
    quote_spanned! {
        variant_ident.span() =>
        Self::#variant_ident{#(#field_names),*} => {
            #(::#crate_ident::macro_utils::MaybeNode(#field_names).update(ctx));*
        }
    }
}

fn enum_unit_branch_body(variant: &VisitVariant) -> TokenStream {
    let variant_ident = &variant.ident;
    quote_spanned! {
        variant_ident.span() =>
        Self::#variant_ident => (),
    }
}

fn struct_visit_body(crate_ident: &Ident, fields: Fields<VisitField>) -> TokenStream {
    fields
        .into_iter()
        .enumerate()
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
                ::#crate_ident::macro_utils::MaybeNode(&mut self.#field_ident).update(ctx);
            }
        })
        .collect()
}

#[derive(FromDeriveInput)]
struct VisitStruct {
    data: Data<VisitVariant, VisitField>,
}

#[derive(FromVariant)]
#[darling(attributes(modor))]
struct VisitVariant {
    ident: Ident,
    fields: Fields<VisitField>,
}

#[derive(FromField)]
#[darling(attributes(modor))]
struct VisitField {
    ident: Option<Ident>,
    ty: Type,
}
