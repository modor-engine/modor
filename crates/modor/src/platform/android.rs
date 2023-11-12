use std::sync::OnceLock;

#[doc(hidden)]
pub use android_activity::AndroidApp;

#[doc(hidden)]
pub static ANDROID_APP: OnceLock<AndroidApp> = OnceLock::new();
