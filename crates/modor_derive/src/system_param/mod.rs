use crate::system_param::parsing::SystemParamStruct;
use proc_macro2::TokenStream;
use syn::DeriveInput;

pub(crate) mod generation;
pub(crate) mod parsing;
pub(crate) mod utils;
pub(crate) mod validation;

pub(super) fn implement_system_param(input: DeriveInput) -> TokenStream {
    let parsed = SystemParamStruct::from_input(&input);
    validation::check_lifetime_uniqueness(&parsed);
    generation::system_param_impl(&parsed)
}
