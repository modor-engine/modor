//! Procedural macros of Modor.

use proc_macro::TokenStream;
use syn::{parse_macro_input, parse_quote, DeriveInput, ItemFn};

mod functions;
mod object;
mod utils;

// coverage: off (cannot be tested)
#[allow(missing_docs)] // doc available in `modor` crate
#[proc_macro_attribute]
#[proc_macro_error::proc_macro_error]
pub fn main(_args: TokenStream, item: TokenStream) -> TokenStream {
    let function = parse_macro_input!(item as ItemFn);
    functions::main_function(&function).into()
}
// coverage: on

#[allow(missing_docs)] // doc available in `modor` crate
#[proc_macro_attribute]
#[proc_macro_error::proc_macro_error]
pub fn test(args: TokenStream, item: TokenStream) -> TokenStream {
    let function = parse_macro_input!(item as ItemFn);
    let args = args.into();
    functions::test_function(&function, args)
        .unwrap_or_else(|error| error)
        .into()
}

#[allow(missing_docs)] // doc available in `modor` crate
#[proc_macro_derive(Object)]
#[proc_macro_error::proc_macro_error]
pub fn object_derive(item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as DeriveInput);
    object::impl_block(&input, &parse_quote!(Object)).into()
}

#[allow(missing_docs)] // doc available in `modor` crate
#[proc_macro_derive(SingletonObject)]
#[proc_macro_error::proc_macro_error]
pub fn singleton_object_derive(item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as DeriveInput);
    object::impl_block(&input, &parse_quote!(SingletonObject)).into()
}
