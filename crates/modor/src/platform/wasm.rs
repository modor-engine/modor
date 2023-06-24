use log::Level;
use scoped_threadpool::Pool;
use std::panic;

pub(crate) fn check_catch_unwind_availability() {
    panic!("`panic::catch_unwind` unsupported on this platform");
}

pub(crate) fn create_pool(_thread_count: u32) -> Option<Pool> {
    None
}

pub(crate) fn init_logging(level: Level) {
    panic::set_hook(Box::new(console_error_panic_hook::hook));
    console_log::init_with_level(level).expect("cannot initialize logger");
}
