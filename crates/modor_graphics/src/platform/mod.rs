#[cfg(target_os = "android")]
include!("android.rs");

#[cfg(not(target_os = "android"))]
include!("not_android.rs");

#[cfg(target_arch = "wasm32")]
include!("wasm.rs");

#[cfg(not(target_arch = "wasm32"))]
include!("not_wasm.rs");
