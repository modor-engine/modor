#![allow(missing_docs, clippy::missing_errors_doc, unsafe_code)] // TODO: remove
#![allow(
    clippy::non_canonical_clone_impl,
    clippy::non_canonical_partial_ord_impl
)] // Warnings coming from Derivative macro

// TODO: add doc

#[cfg(target_os = "android")]
pub use android_activity;
pub use log;
#[cfg(target_arch = "wasm32")]
pub use wasm_bindgen_test;

mod app;
mod node;
mod platform;

pub use app::*;
pub use node::*;
#[allow(unused_imports, unreachable_pub)]
pub use platform::*;

pub use modor_derive::main;
pub use modor_derive::test;
pub use modor_derive::Node;
pub use modor_derive::RootNode;
pub use modor_derive::Visit;
