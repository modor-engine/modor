//! Procedural macros of Modor.

use crate::actions::ActionStruct;
use crate::components::ComponentType;
use crate::system_impl::SystemImpl;
use crate::system_params::SystemParamStruct;
use crate::tests::TestFunction;
use proc_macro::TokenStream;
use syn::{parse_macro_input, AttributeArgs, DeriveInput, ItemFn, ItemImpl};

mod actions;
mod common;
mod components;
mod system_impl;
mod system_params;
mod tests;

#[allow(missing_docs)] // doc available in `modor` crate
#[proc_macro_attribute]
#[proc_macro_error::proc_macro_error]
pub fn modor_test(attr: TokenStream, item: TokenStream) -> TokenStream {
    let function = parse_macro_input!(item as ItemFn);
    let args = parse_macro_input!(attr as AttributeArgs);
    let (Ok(output) | Err(output)) = TestFunction::new(&function, &args).map(|f| f.annotated());
    output.into()
}

#[allow(missing_docs)] // doc available in `modor` crate
#[proc_macro_derive(Action)]
#[proc_macro_error::proc_macro_error]
pub fn action_derive(item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as DeriveInput);
    ActionStruct::new(&input).action_impl().into()
}

#[allow(missing_docs)] // doc available in `modor` crate
#[proc_macro_derive(Component)]
#[proc_macro_error::proc_macro_error]
pub fn component_derive(item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as DeriveInput);
    ComponentType::new(&input).component_impl().into()
}

#[allow(missing_docs)] // doc available in `modor` crate
#[proc_macro_derive(SingletonComponent)]
#[proc_macro_error::proc_macro_error]
pub fn singleton_component_derive(item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as DeriveInput);
    ComponentType::new(&input).singleton_component_impl().into()
}

#[allow(missing_docs)] // doc available in `modor` crate
#[proc_macro_derive(NoSystem)]
#[proc_macro_error::proc_macro_error]
pub fn no_system_derive(item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as DeriveInput);
    ComponentType::new(&input).no_system_impl().into()
}

#[allow(missing_docs)] // doc available in `modor` crate
#[proc_macro_attribute]
#[proc_macro_error::proc_macro_error]
pub fn systems(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let item = parse_macro_input!(item as ItemImpl);
    let (Ok(output) | Err(output)) = SystemImpl::new(&item).map(|f| f.component_systems_impl());
    output.into()
}

#[allow(missing_docs)] // doc available in `modor` crate
#[proc_macro_derive(SystemParam)]
#[proc_macro_error::proc_macro_error]
pub fn system_param_derive(item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as DeriveInput);
    SystemParamStruct::new(&input)
        .custom_system_param_impl()
        .into()
}

#[allow(missing_docs)] // doc available in `modor` crate
#[proc_macro_derive(QuerySystemParam)]
#[proc_macro_error::proc_macro_error]
pub fn query_system_param_derive(item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as DeriveInput);
    SystemParamStruct::new(&input)
        .custom_query_system_param_impl()
        .into()
}
