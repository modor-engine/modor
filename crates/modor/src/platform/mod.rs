#[cfg(target_arch = "wasm32")]
include!("sync_send_wasm.rs");

#[cfg(not(target_arch = "wasm32"))]
include!("sync_send_not_wasm.rs");

#[cfg(target_os = "android")]
include!("other_android.rs");

#[cfg(target_arch = "wasm32")]
include!("other_wasm.rs");

#[cfg(not(any(target_os = "android", target_arch = "wasm32")))]
include!("other_not_android_wasm.rs");
