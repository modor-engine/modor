use crate::common::{generation, idents, lifetimes, tuples};
use proc_macro2::{Ident, Span, TokenStream};
use proc_macro_error::abort;
use quote::quote;
use syn::spanned::Spanned;
use syn::{
    parse_quote, parse_quote_spanned, Data, DataStruct, DeriveInput, Expr, Fields, FieldsNamed,
    FieldsUnnamed, Lifetime, Type,
};

pub(crate) struct SystemParamStruct<'a> {
    crate_name: String,
    input: &'a DeriveInput,
    data: &'a DataStruct,
    lt: Option<&'a Lifetime>,
    internal_lt: Lifetime,
    tuple_arg: Ident,
    const_ident: Ident,
}

impl<'a> SystemParamStruct<'a> {
    pub(crate) fn new(input: &'a DeriveInput) -> Self {
        Self {
            crate_name: idents::crate_name(),
            input,
            data: match &input.data {
                Data::Struct(fields) => fields,
                Data::Enum(data) => {
                    abort!(data.enum_token, "custom system param cannot be an enum")
                }
                Data::Union(data) => {
                    abort!(data.union_token, "custom system param cannot be a union")
                }
            },
            lt: lifetimes::nth(&input.generics, 0),
            internal_lt: parse_quote! { '__modor },
            tuple_arg: parse_quote! { tuple },
            const_ident: idents::add_prefix(&input.ident, "Const"),
        }
    }

    pub(crate) fn custom_system_param_impl(&self) -> TokenStream {
        self.check_lifetime_count();
        self.impl_block(
            &self.input.ident,
            &self.param(&self.input.ident),
            &self.tuple(),
            &self.constructor(&self.input.ident),
        )
    }

    pub(crate) fn custom_query_system_param_impl(&self) -> TokenStream {
        self.check_lifetime_count();
        let impl_block = self.custom_system_param_impl();
        let query_impl_block = self.query_impl_block(&self.input.ident);
        let const_struct = self.const_struct();
        let const_impl_block = self.impl_block(
            &self.const_ident,
            &self.param(&self.const_ident),
            &self.const_tuple(),
            &self.constructor(&self.const_ident),
        );
        let const_query_impl_block = self.query_impl_block(&self.const_ident);
        quote! {
            #[allow(unused, unused_qualifications)]
            #impl_block
            #[allow(unused_qualifications)]
            #query_impl_block

            #[allow(unused, unused_qualifications)]
            #const_struct

            #[allow(unused_qualifications)]
            #const_impl_block
            #[allow(unused_qualifications)]
            #const_query_impl_block
        }
    }

    fn check_lifetime_count(&self) {
        if lifetimes::count(self.input) != 1 {
            abort!(
                self.input.generics,
                "custom system param should have exactly one generic lifetime",
            );
        }
    }

    fn impl_block(
        &self,
        ident: &Ident,
        param: &Type,
        tuple: &Type,
        constructor: &Expr,
    ) -> TokenStream {
        let crate_ = Ident::new(&self.crate_name, Span::call_site());
        let lt = &self.internal_lt;
        let tuple_arg = &self.tuple_arg;
        let impl_header = self.impl_header(ident, &parse_quote! { CustomSystemParam });
        quote! {
            #[allow(unused_variables)]
            #impl_header {
                type Param<#lt> = #param;
                type Tuple = #tuple;

                fn from_tuple_mut_param(
                    #tuple_arg: <Self::Tuple as #crate_::SystemParamWithLifetime<'_>>::Param,
                ) -> #crate_::Custom<Self::Param<'_>> {
                    #crate_::Custom::new(#constructor)
                }
            }
        }
    }

    fn query_impl_block(&self, ident: &Ident) -> TokenStream {
        let crate_ = Ident::new(&self.crate_name, Span::call_site());
        let lt = &self.internal_lt;
        let tuple_arg = &self.tuple_arg;
        let const_param = self.param(&self.const_ident);
        let const_constructor = self.constructor(&self.const_ident);
        let impl_header = self.impl_header(ident, &parse_quote! { CustomQuerySystemParam });
        quote! {
            #[allow(unused_variables)]
            #impl_header {
                type ConstParam<#lt> = #const_param;

                fn from_tuple_const_param_mut_param<#lt>(
                    #tuple_arg: <
                        <
                            Self::Tuple as #crate_::QuerySystemParamWithLifetime<#lt>
                        >::ConstParam as #crate_::SystemParamWithLifetime<#lt>
                    >::Param,
                ) -> <
                    #crate_::Custom<Self::ConstParam<#lt>
                > as #crate_::SystemParamWithLifetime<#lt>>::Param {
                    #crate_::Custom::new(#const_constructor)
                }

                fn from_tuple_const_param(
                    #tuple_arg: <
                        Self::Tuple as #crate_::QuerySystemParamWithLifetime<'_>
                    >::ConstParam,
                ) -> #crate_::Custom<Self::ConstParam<'_>> {
                    #crate_::Custom::new(#const_constructor)
                }
            }
        }
    }

    fn impl_header(&self, type_ident: &Ident, trait_ident: &Ident) -> TokenStream {
        let crate_ = Ident::new(&self.crate_name, Span::call_site());
        generation::trait_impl_header(
            &self.input.generics,
            type_ident,
            &parse_quote! { #crate_::#trait_ident },
        )
    }

    fn const_struct(&self) -> DeriveInput {
        let mut data = self.data.clone();
        for field in &mut data.fields {
            field.ty = self.const_field_type(&field.ty);
        }
        let mut input = self.input.clone();
        input.ident = self.const_ident.clone();
        input.data = Data::Struct(data);
        input
    }

    fn param(&self, ident: &Ident) -> Type {
        let input = self.input;
        let generics = lifetimes::rename_nth(&input.generics, 0, &self.internal_lt);
        let (_, type_generics, _) = generics.split_for_impl();
        parse_quote! { #ident #type_generics }
    }

    fn tuple(&self) -> Type {
        let types = self.data.fields.iter().map(|f| f.ty.clone());
        tuples::recursive(types)
    }

    fn const_tuple(&self) -> Type {
        let types = self
            .data
            .fields
            .iter()
            .map(|f| self.const_field_type(&f.ty));
        tuples::recursive(types)
    }

    fn const_field_type(&self, type_: &Type) -> Type {
        let span = type_.span();
        let crate_ = Ident::new(&self.crate_name, span);
        let lt = &self.lt;
        parse_quote_spanned! {
            span =>
            <#type_ as #crate_::QuerySystemParamWithLifetime<#lt>>::ConstParam
        }
    }

    fn constructor(&self, ident: &Ident) -> Expr {
        match &self.data.fields {
            Fields::Named(fields) => self.named_constructor(fields, ident),
            Fields::Unnamed(fields) => self.unnamed_constructor(fields, ident),
            Fields::Unit => unreachable!("internal error: unit unsupported (1 lifetime required)"),
        }
    }

    fn named_constructor(&self, fields: &FieldsNamed, ident: &Ident) -> Expr {
        let field_names = fields.named.iter().map(|f| &f.ident);
        let values = (0..fields.named.len())
            .map(|i| tuples::recursive_access(&self.tuple_arg, i, fields.named.len()));
        parse_quote! { #ident { #(#field_names: #values),* } }
    }

    fn unnamed_constructor(&self, fields: &FieldsUnnamed, ident: &Ident) -> Expr {
        let values = (0..fields.unnamed.len())
            .map(|i| tuples::recursive_access(&self.tuple_arg, i, fields.unnamed.len()));
        parse_quote! { #ident ( #(#values),* ) }
    }
}
