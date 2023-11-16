use log::Level;
use std::panic;

pub(crate) fn check_catch_unwind_availability() {
    panic!("`panic::catch_unwind` unsupported on this platform");
}

pub(crate) fn init_logging(level: Level) {
    panic::set_hook(Box::new(console_error_panic_hook::hook));
    console_log::init_with_level(level).expect("cannot initialize logger");
}
