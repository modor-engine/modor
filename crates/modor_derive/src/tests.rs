#![allow(clippy::option_if_let_else)]

use darling::ast::NestedMeta;
use darling::util::PathList;
use darling::FromMeta;
use proc_macro2::TokenStream;
use proc_macro_error::abort;
use quote::quote;
use std::collections::HashMap;
use syn::{parse_quote, ItemFn, Meta, Path};

pub(crate) struct TestFunction<'a> {
    function: &'a ItemFn,
    args: TestArgs,
    platform_conditions: HashMap<&'static str, Meta>,
    supported_platforms: Vec<&'static str>,
}

impl<'a> TestFunction<'a> {
    pub(crate) fn new(function: &'a ItemFn, args: TokenStream) -> Result<Self, TokenStream> {
        let args = NestedMeta::parse_meta_list(args)
            .map_err(|e| darling::Error::from(e).write_errors())?;
        let platform_conditions = Self::platform_conditions();
        let mut supported_platforms: Vec<_> = Self::platform_conditions().keys().copied().collect();
        supported_platforms.sort_unstable();
        TestArgs::from_list(&args)
            .map_err(darling::Error::write_errors)
            .map(|args| Self {
                function,
                args,
                platform_conditions,
                supported_platforms,
            })
    }

    pub(crate) fn annotated(&self) -> TokenStream {
        self.check_platform_paths(&self.args);
        let function = &self.function;
        let disabled_platform_conditions = self.disabled_platform_conditions();
        quote! {
            #[cfg_attr(any(#(#disabled_platform_conditions),*), allow(unused))]
            #[cfg_attr(not(any(#(#disabled_platform_conditions),*)), test)]
            #[cfg_attr(
                all(target_arch = "wasm32", not(any(#(#disabled_platform_conditions),*))),
                ::wasm_bindgen_test::wasm_bindgen_test)
            ]
            #function
        }
    }

    fn disabled_platform_conditions(&self) -> Vec<Meta> {
        return self
            .args
            .disabled
            .iter()
            .map(|p| self.platform_condition(p))
            .cloned()
            .collect();
    }

    fn platform_condition(&self, platform: &Path) -> &Meta {
        self.platform_conditions
            .get(platform.segments[0].ident.to_string().as_str())
            .unwrap_or_else(|| {
                abort!(
                    platform,
                    "allowed platforms are {:?}",
                    &self.supported_platforms
                )
            })
    }

    fn check_platform_paths(&self, args: &TestArgs) {
        for platform in args.disabled.iter() {
            if platform.segments.len() > 1 {
                abort!(
                    platform,
                    "allowed platforms are {:?}",
                    self.supported_platforms
                );
            }
        }
    }

    fn platform_conditions() -> HashMap<&'static str, Meta> {
        [
            ("windows", parse_quote! { target_os = "windows" }),
            ("macos", parse_quote! { target_os = "macos" }),
            ("android", parse_quote! { target_os = "android" }),
            ("wasm", parse_quote! { target_arch = "wasm32" }),
            (
                "linux",
                parse_quote! { any(
                    target_os = "linux",
                    target_os = "dragonfly",
                    target_os = "freebsd",
                    target_os = "netbsd",
                    target_os = "openbsd"
                ) },
            ),
        ]
        .into_iter()
        .collect()
    }
}

#[derive(FromMeta)]
struct TestArgs {
    #[darling(default)]
    disabled: PathList,
}
