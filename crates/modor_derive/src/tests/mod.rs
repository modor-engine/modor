use crate::tests::parsing::TestArgs;
use darling::FromMeta;
use proc_macro2::TokenStream;
use quote::quote;
use syn::{AttributeArgs, ItemFn};

mod generation;
mod parsing;
mod validation;

pub(crate) fn define_test_method(function: ItemFn, args: AttributeArgs) -> TokenStream {
    match TestArgs::from_list(&args) {
        Ok(args) => {
            validation::check_platform_paths(&args);
            let conditions = generation::platform_conditions(&args);
            quote! {
                #[cfg_attr(any(#(#conditions),*), allow(unused))]
                #[cfg_attr(not(any(#(#conditions),*)), test)]
                #[cfg_attr(
                    all(target_arch = "wasm32", not(any(#(#conditions),*))),
                    ::wasm_bindgen_test::wasm_bindgen_test)
                ]
                #function
            }
        }
        Err(error) => error.write_errors(),
    }
}
