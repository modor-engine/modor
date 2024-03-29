use proc_macro2::TokenStream;
use quote::quote;
use syn::ItemFn;

// coverage: off (associated derive macro cannot be tested)

pub(crate) struct MainEndpoint<'a> {
    function: &'a ItemFn,
}

impl<'a> MainEndpoint<'a> {
    pub(crate) fn new(function: &'a ItemFn) -> Self {
        Self { function }
    }

    pub(crate) fn main_function(&self) -> TokenStream {
        let function = self.function;
        let ident = &function.sig.ident;
        quote! {
            #[cfg(target_os = "android")]
            #[no_mangle]
            fn android_main(app: modor::AndroidApp) {
                let _ = modor::ANDROID_APP.get_or_init(move || app);
                #function
                #ident();
            }

            // Unused main method, defined only to avoid error with Clippy
            #[cfg(target_os = "android")]
            #[allow(dead_code)]
            fn main() {}

            #[cfg(not(target_os = "android"))]
            fn main() {
                #function
                #ident();
            }
        }
    }
}
