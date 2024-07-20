#[cfg(target_os = "android")]
include!("android.rs");

#[cfg(target_arch = "wasm32")]
include!("wasm.rs");

#[cfg(not(any(target_os = "android", target_arch = "wasm32")))]
include!("not_android_wasm.rs");
