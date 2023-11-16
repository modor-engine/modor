use log::{Level, LevelFilter};
use std::sync::OnceLock;

#[doc(hidden)]
pub use android_activity::AndroidApp;
use android_logger::Config;

#[doc(hidden)]
pub static ANDROID_APP: OnceLock<AndroidApp> = OnceLock::new();

pub(crate) fn check_catch_unwind_availability() {
    // available
}

pub(crate) fn init_logging(level: Level) {
    let config = Config::default().with_max_level(LevelFilter::Trace); // allow all levels at compile time
    android_logger::init_once(config);
    log::set_max_level(level.to_level_filter());
}
