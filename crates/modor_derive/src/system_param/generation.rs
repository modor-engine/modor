use crate::idents;
use crate::system_param::parsing::{SystemParamStruct, SystemParamStructFields};
use crate::system_param::utils;
use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
use syn::spanned::Spanned;
use syn::{Lifetime, Type};

const GENERIC_LIFETIME: &str = "'__modor";

pub(super) fn system_param_impl(parsed: &SystemParamStruct) -> TokenStream {
    let crate_ident = idents::find_crate_ident(parsed.input.span());
    let ident = &parsed.input.ident;
    let (impl_generics, type_generics, where_clause) = parsed.input.generics.split_for_impl();
    let generic_lifetime = Lifetime::new(GENERIC_LIFETIME, Span::call_site());
    let renamed_generics = utils::replace_first_lifetime(&parsed.input.generics, &generic_lifetime);
    let (_, renamed_type_generics, _) = renamed_generics.split_for_impl();
    let tuple = tuple(parsed);
    let constructor_from_tuple = constructor_from_tuple(parsed, quote!(tuple));
    quote! {
        impl #impl_generics #crate_ident::CustomSystemParam for #ident #type_generics #where_clause {
            type ConstParam<#generic_lifetime> = #ident #renamed_type_generics;
            type Param<#generic_lifetime> = #ident #renamed_type_generics;
            type Tuple = #tuple;

            fn from_tuple_const_param_mut_param<#generic_lifetime>(
                _tuple: <
                    <Self::Tuple
                    as #crate_ident::QuerySystemParamWithLifetime<#generic_lifetime>>::ConstParam
                    as #crate_ident::SystemParamWithLifetime<#generic_lifetime>>::Param,
            ) -> <#crate_ident::Custom<Self::ConstParam<#generic_lifetime>>
                as #crate_ident::SystemParamWithLifetime<#generic_lifetime>>::Param
            where
                Self::Tuple: #crate_ident::QuerySystemParam,
            {
                unreachable!()
            }

            fn from_tuple_const_param(
                _tuple: <Self::Tuple as #crate_ident::QuerySystemParamWithLifetime<'_>>::ConstParam,
            ) -> #crate_ident::Custom<Self::ConstParam<'_>>
            where
                Self::Tuple: #crate_ident::QuerySystemParam,
            {
                unreachable!()
            }

            fn from_tuple_mut_param(
                tuple: <Self::Tuple as #crate_ident::SystemParamWithLifetime<'_>>::Param,
            ) -> #crate_ident::Custom<Self::Param<'_>> {
                #crate_ident::Custom::new(#ident #constructor_from_tuple)
            }
        }
    }
}

pub(super) fn query_system_param_impl(parsed: &SystemParamStruct) -> TokenStream {
    let crate_ident = idents::find_crate_ident(parsed.input.span());
    let ident = &parsed.input.ident;
    let const_ident = Ident::new(&format!("Const{}", parsed.input.ident), ident.span());
    let (impl_generics, type_generics, where_clause) = parsed.input.generics.split_for_impl();
    let generic_lifetime = Lifetime::new(GENERIC_LIFETIME, Span::call_site());
    let renamed_generics = utils::replace_first_lifetime(&parsed.input.generics, &generic_lifetime);
    let (_, renamed_type_generics, _) = renamed_generics.split_for_impl();
    let tuple = tuple(parsed);
    let const_tuple = const_tuple(parsed);
    let constructor_from_tuple = constructor_from_tuple(parsed, quote!(tuple));
    let const_struct = const_struct(parsed, &const_ident);
    quote! {
        impl #impl_generics #crate_ident::CustomSystemParam for #ident #type_generics
            #where_clause
        {
            type ConstParam<#generic_lifetime> = #const_ident #renamed_type_generics;
            type Param<#generic_lifetime> = #ident #renamed_type_generics;
            type Tuple = #tuple;

            fn from_tuple_const_param_mut_param<#generic_lifetime>(
                tuple: <
                    <Self::Tuple
                    as #crate_ident::QuerySystemParamWithLifetime<#generic_lifetime>>::ConstParam
                    as #crate_ident::SystemParamWithLifetime<#generic_lifetime>>::Param,
            ) -> <#crate_ident::Custom<Self::ConstParam<#generic_lifetime>>
                as #crate_ident::SystemParamWithLifetime<#generic_lifetime>>::Param
            {
                #crate_ident::Custom::new(#const_ident #constructor_from_tuple)
            }

            fn from_tuple_const_param(
                tuple: <Self::Tuple as #crate_ident::QuerySystemParamWithLifetime<'_>>::ConstParam,
            ) -> #crate_ident::Custom<Self::ConstParam<'_>>
            {
                #crate_ident::Custom::new(#const_ident #constructor_from_tuple)
            }

            fn from_tuple_mut_param(
                tuple: <Self::Tuple as #crate_ident::SystemParamWithLifetime<'_>>::Param,
            ) -> #crate_ident::Custom<Self::Param<'_>> {
                #crate_ident::Custom::new(#ident #constructor_from_tuple)
            }
        }

        #const_struct

        impl #impl_generics #crate_ident::CustomSystemParam for #const_ident #type_generics
            #where_clause
        {
            type ConstParam<#generic_lifetime> = #const_ident #renamed_type_generics;
            type Param<#generic_lifetime> = #const_ident #renamed_type_generics;
            type Tuple = #const_tuple;

            fn from_tuple_const_param_mut_param<#generic_lifetime>(
                tuple: <
                    <Self::Tuple
                    as #crate_ident::QuerySystemParamWithLifetime<#generic_lifetime>>::ConstParam
                    as #crate_ident::SystemParamWithLifetime<#generic_lifetime>>::Param,
            ) -> <#crate_ident::Custom<Self::ConstParam<#generic_lifetime>>
                as #crate_ident::SystemParamWithLifetime<#generic_lifetime>>::Param
            {
                #crate_ident::Custom::new(#const_ident #constructor_from_tuple)
            }

            fn from_tuple_const_param(
                tuple: <Self::Tuple as #crate_ident::QuerySystemParamWithLifetime<'_>>::ConstParam,
            ) -> #crate_ident::Custom<Self::ConstParam<'_>>
            {
                #crate_ident::Custom::new(#const_ident #constructor_from_tuple)
            }

            fn from_tuple_mut_param(
                tuple: <Self::Tuple as #crate_ident::SystemParamWithLifetime<'_>>::Param,
            ) -> #crate_ident::Custom<Self::Param<'_>> {
                #crate_ident::Custom::new(#const_ident #constructor_from_tuple)
            }
        }
    }
}

fn tuple(parsed: &SystemParamStruct) -> TokenStream {
    field_types(parsed)
        .into_iter()
        .fold(quote! { () }, |o, t| quote! { (#o, #t) })
}

fn const_tuple(parsed: &SystemParamStruct) -> TokenStream {
    let crate_ident = idents::find_crate_ident(parsed.input.span());
    field_types(parsed).into_iter().fold(
        quote!(()),
        |o, t| quote!((<#o as #crate_ident::QuerySystemParamWithLifetime<'a>>::ConstParam, #t)),
    )
}

fn field_types(parsed: &SystemParamStruct) -> Vec<&Type> {
    match &parsed.fields {
        SystemParamStructFields::Named(fields) => fields.iter().map(|f| &f.type_).collect(),
        SystemParamStructFields::Unnamed(fields) => fields.iter().map(|f| &f.type_).collect(),
        SystemParamStructFields::Unit => vec![],
    }
}

fn constructor_from_tuple(parsed: &SystemParamStruct, tuple: TokenStream) -> TokenStream {
    match &parsed.fields {
        SystemParamStructFields::Named(fields) => {
            let field_names = fields.iter().map(|f| &f.ident);
            let field_accessors = (0..fields.len()).map(|i| tuple_accessor(fields.len() - i - 1));
            quote! { { #(#field_names: #tuple #field_accessors,)* } }
        }
        SystemParamStructFields::Unnamed(fields) => {
            let field_accessors = (0..fields.len()).map(|i| tuple_accessor(fields.len() - i - 1));
            quote! { (#(#tuple #field_accessors,)*) }
        }
        SystemParamStructFields::Unit => quote! {},
    }
}

fn tuple_accessor(zero_count: usize) -> TokenStream {
    (0..zero_count).fold(quote! { .1 }, |o, _| quote! { .0 #o })
}

fn const_struct(parsed: &SystemParamStruct, const_ident: &Ident) -> TokenStream {
    let crate_ident = idents::find_crate_ident(parsed.input.span());
    let visibility = &parsed.input.vis;
    let generics = &parsed.input.generics;
    let where_clause = &generics.where_clause;
    match &parsed.fields {
        SystemParamStructFields::Named(fields) => {
            let const_fields: Vec<_> = fields
                .iter()
                .map(|f| {
                    let visibility = &f.visibility;
                    let ident = &f.ident;
                    let type_ = &f.type_;
                    quote! {
                        #visibility #ident:
                        <#type_ as #crate_ident::QuerySystemParamWithLifetime<'a>>::ConstParam,
                    }
                })
                .collect();
            quote! { #visibility struct #const_ident #generics #where_clause { #(#const_fields)* } }
        }
        SystemParamStructFields::Unnamed(fields) => {
            let const_fields: Vec<_> = fields
                .iter()
                .map(|f| {
                    let visibility = &f.visibility;
                    let type_ = &f.type_;
                    quote! {
                        #visibility
                        <#type_ as #crate_ident::QuerySystemParamWithLifetime<'a>>::ConstParam,
                    }
                })
                .collect();
            quote! { #visibility struct #const_ident #generics (#(#const_fields)*) #where_clause; }
        }
        SystemParamStructFields::Unit => {
            quote! { #visibility struct #const_ident #generics #where_clause; }
        }
    }
}
