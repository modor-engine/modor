use crate::system_param::parsing::SystemParamStruct;
use proc_macro2::TokenStream;
use syn::DeriveInput;

mod generation;
mod parsing;
mod validation;

pub(crate) fn implement_simple(input: DeriveInput) -> TokenStream {
    let parsed = SystemParamStruct::from_input(&input);
    validation::check_lifetime_uniqueness(&parsed);
    generation::system_param_impl(&parsed)
}

pub(crate) fn implement_query(input: DeriveInput) -> TokenStream {
    let parsed = SystemParamStruct::from_input(&input);
    validation::check_lifetime_uniqueness(&parsed);
    generation::query_system_param_impl(&parsed)
}
