//! Procedural macros of Modor.

use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput, ItemFn};

mod builder;
mod from_app;
mod functions;
mod global;
mod state;
mod updater;
mod utils;

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
#[proc_macro_derive(State)]
pub fn state_derive(item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as DeriveInput);
    state::impl_block(&input).into()
}

#[allow(missing_docs)] // doc available in `modor` crate
#[proc_macro_derive(Global)]
pub fn global_derive(item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as DeriveInput);
    global::impl_block(&input).into()
}

#[allow(missing_docs)] // doc available in `modor` crate
#[proc_macro_derive(Builder, attributes(builder))]
pub fn builder_derive(item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as DeriveInput);
    builder::impl_block(&input)
        .unwrap_or_else(Into::into)
        .into()
}

#[allow(missing_docs)] // doc available in `modor` crate
#[proc_macro_derive(FromApp)]
pub fn from_app_derive(item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as DeriveInput);
    from_app::impl_block(&input)
        .unwrap_or_else(Into::into)
        .into()
}

#[allow(missing_docs)] // doc available in `modor` crate
#[proc_macro_derive(Updater, attributes(updater))]
pub fn updater_derive(item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as DeriveInput);
    updater::impl_block(&input)
        .unwrap_or_else(Into::into)
        .into()
}
