//! Procedural macros of Modor.

use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput, ItemFn};

mod functions;
mod node;
mod root_node;
mod utils;
mod visit;

// coverage: off (cannot be tested)
#[allow(missing_docs)] // doc available in `modor` crate
#[proc_macro_attribute]
pub fn main(_args: TokenStream, item: TokenStream) -> TokenStream {
    let function = parse_macro_input!(item as ItemFn);
    functions::main_function(&function).into()
}
// coverage: on

#[allow(missing_docs)] // doc available in `modor` crate
#[proc_macro_attribute]
pub fn test(args: TokenStream, item: TokenStream) -> TokenStream {
    let function = parse_macro_input!(item as ItemFn);
    let args = args.into();
    functions::test_function(&function, args)
        .unwrap_or_else(|error| error)
        .into()
}

#[allow(missing_docs)] // doc available in `modor` crate
#[proc_macro_derive(RootNode)]
pub fn root_node_derive(item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as DeriveInput);
    root_node::impl_block(&input).into()
}

#[allow(missing_docs)] // doc available in `modor` crate
#[proc_macro_derive(Node)]
pub fn node_derive(item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as DeriveInput);
    node::impl_block(&input).into()
}

#[allow(missing_docs)] // doc available in `modor` crate
#[proc_macro_derive(Visit, attributes(modor))]
pub fn visit_derive(item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as DeriveInput);
    visit::impl_block_with_visit(&input)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}
