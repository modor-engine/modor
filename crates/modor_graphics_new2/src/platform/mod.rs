pub(crate) mod testing;

#[cfg(target_arch = "wasm32")]
include!("wasm.rs");

#[cfg(not(target_arch = "wasm32"))]
include!("not_wasm.rs");
