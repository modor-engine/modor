use darling::ast::GenericParamExt;
use proc_macro2::{Ident, TokenStream};
use quote::quote;
use std::iter;
use syn::{
    parse_quote, DeriveInput, ExprField, GenericParam, Generics, Index, ItemStruct, Lifetime, Type,
    Visibility,
};

pub(crate) fn count_lifetimes(input: &DeriveInput) -> usize {
    input
        .generics
        .params
        .iter()
        .filter(|p| p.as_lifetime_def().is_some())
        .count()
}

pub(crate) fn with_renamed_lifetime(
    generics: &Generics,
    position: usize,
    new_lifetime: &Lifetime,
) -> Generics {
    let mut generics = generics.clone();
    if let Some(GenericParam::Lifetime(lifetime)) = generics.params.iter_mut().nth(position) {
        lifetime.lifetime = new_lifetime.clone();
    }
    generics
}

pub(crate) fn nth_lifetime(generics: &Generics, position: usize) -> Option<&Lifetime> {
    generics.params.iter().nth(position).and_then(|p| {
        if let GenericParam::Lifetime(lifetime) = p {
            Some(&lifetime.lifetime)
        } else {
            None
        }
    })
}

pub(crate) fn recursive_tuple(types: impl Iterator<Item = Type>) -> Type {
    types.fold(parse_quote! { () }, |o, t| parse_quote! { (#o, #t) })
}

pub(crate) fn recursive_tuple_access(
    tuple_var: &Ident,
    item_pos: usize,
    item_count: usize,
) -> ExprField {
    let zero_index: Index = parse_quote! { 0 };
    let one_index: Index = parse_quote! { 1 };
    let zero_count = item_count - item_pos - 1;
    let indices = iter::repeat(zero_index)
        .take(zero_count)
        .chain(iter::once(one_index));
    parse_quote! { #tuple_var #(.#indices)* }
}

pub(crate) fn impl_header(generics: &Generics, struct_: &Ident, trait_: &Type) -> TokenStream {
    let (impl_generics, type_generics, where_clause) = generics.split_for_impl();
    quote!(
        impl #impl_generics #trait_ for #struct_ #type_generics #where_clause
    )
}

pub(crate) fn ident_with_prefix(prefix: &str, ident: &Ident) -> Ident {
    Ident::new(&format!("{prefix}{ident}"), ident.span())
}

pub(crate) fn ident_with_suffix(ident: &Ident, suffix: &str) -> Ident {
    Ident::new(&format!("{ident}{suffix}"), ident.span())
}

pub(crate) fn tuple_struct(
    visibility: &Visibility,
    type_: &Ident,
    generics: &Generics,
    field_types: &[Type],
) -> ItemStruct {
    let (impl_generics, _type_generics, where_clause) = generics.split_for_impl();
    parse_quote! {
        #visibility struct #type_ #impl_generics(#(#field_types),*) #where_clause;
    }
}
