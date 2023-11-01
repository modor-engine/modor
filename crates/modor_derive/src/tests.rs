#![allow(clippy::option_if_let_else)]

use darling::ast::NestedMeta;
use darling::util::{PathList, SpannedValue};
use darling::FromMeta;
use proc_macro2::{Ident, TokenStream};
use proc_macro_error::{abort, OptionExt};
use quote::{quote, quote_spanned};
use std::collections::HashMap;
use syn::spanned::Spanned;
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
        if self.args.cases.0.is_empty() {
            self.annotated_without_cases()
        } else {
            self.annotated_with_cases()
        }
    }

    pub(crate) fn annotated_without_cases(&self) -> TokenStream {
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

    pub(crate) fn annotated_with_cases(&self) -> TokenStream {
        let function = &self.function;
        let main_function_ident = &function.sig.ident;
        let disabled_platform_conditions = self.disabled_platform_conditions();
        let test_functions = self.args.cases.0.iter().map(|(suffix, params)| {
            let span = params.span();
            let function_ident =
                Ident::new(&format!("{main_function_ident}_{suffix}"), span.span());
            let params = params
                .parse::<TokenStream>()
                .ok()
                .expect_or_abort("cannot parse test case args")
                .into_iter()
                .map(|mut token| {
                    token.set_span(span);
                    token
                })
                .collect::<TokenStream>();
            let params = quote_spanned! {span => #params};
            quote_spanned! {
                span =>
                #[cfg_attr(any(#(#disabled_platform_conditions),*), allow(unused))]
                #[cfg_attr(not(any(#(#disabled_platform_conditions),*)), test)]
                #[cfg_attr(
                    all(target_arch = "wasm32", not(any(#(#disabled_platform_conditions),*))),
                    ::wasm_bindgen_test::wasm_bindgen_test)
                ]
                fn #function_ident() {
                    #main_function_ident(#params);
                }
            }
        });
        quote! {
            #function

            #(#test_functions)*
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
    #[darling(default)]
    cases: TestCases,
}

#[derive(Default, FromMeta)]
struct TestCases(HashMap<String, SpannedValue<String>>);
