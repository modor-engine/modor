#[cfg(target_os = "android")]
include!("android.rs");

#[cfg(not(target_os = "android"))]
include!("not_android.rs");
