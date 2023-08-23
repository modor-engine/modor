use proc_macro2::Ident;
use std::iter;
use syn::{parse_quote, ExprField, Index, Type};

pub(crate) fn recursive(types: impl Iterator<Item = Type>) -> Type {
    types.fold(parse_quote! { () }, |o, t| parse_quote! { (#o, #t) })
}

pub(crate) fn recursive_access(tuple_var: &Ident, item_pos: usize, item_count: usize) -> ExprField {
    let zero_index: Index = parse_quote! { 0 };
    let one_index: Index = parse_quote! { 1 };
    let zero_count = item_count - item_pos - 1;
    let indices = iter::repeat(zero_index)
        .take(zero_count)
        .chain(iter::once(one_index));
    parse_quote! { #tuple_var #(.#indices)* }
}
