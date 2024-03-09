use log::Level;
use std::panic;

pub(crate) fn init_logging(level: Level) {
    panic::set_hook(Box::new(console_error_panic_hook::hook));
    let _ = console_log::init_with_level(level);
}
