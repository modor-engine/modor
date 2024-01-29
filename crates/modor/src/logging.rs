use crate::platform;
use log::Level;
use std::sync::Once;

pub(crate) const DEFAULT_LEVEL: Level = Level::Warn;

pub(crate) fn init() {
    static INIT: Once = Once::new();
    INIT.call_once(|| platform::init_logging(DEFAULT_LEVEL));
}
