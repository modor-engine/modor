use crate::tests::parsing::TestArgs;
use proc_macro2::TokenStream;
use proc_macro_error::abort;
use quote::quote;
use std::collections::HashMap;

pub(super) fn platform_conditions(args: &TestArgs) -> Vec<TokenStream> {
    let all_conditions = all_conditions();
    return args
        .disabled
        .iter()
        .map(|p| {
            all_conditions
                .get(p.segments[0].ident.to_string().as_str())
                .unwrap_or_else(|| abort!(p, "allowed platforms are {:?}", supported_platforms()))
        })
        .cloned()
        .collect();
}

pub(super) fn supported_platforms() -> Vec<&'static str> {
    let mut platforms: Vec<_> = all_conditions().keys().copied().collect();
    platforms.sort_unstable();
    platforms
}

fn all_conditions() -> HashMap<&'static str, TokenStream> {
    [
        ("windows", quote!(target_os = "windows")),
        ("macos", quote!(target_os = "macos")),
        ("android", quote!(target_os = "android")),
        ("wasm", quote!(target_arch = "wasm32")),
        (
            "linux",
            quote!(any(
                target_os = "linux",
                target_os = "dragonfly",
                target_os = "freebsd",
                target_os = "netbsd",
                target_os = "openbsd"
            )),
        ),
    ]
    .into_iter()
    .collect()
}
