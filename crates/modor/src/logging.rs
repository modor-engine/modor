use log::Level;
use std::sync::Once;

const DEFAULT_LEVEL: Level = Level::Warn;

static INIT: Once = Once::new();

pub(crate) fn init() {
    INIT.call_once(|| {
        #[cfg(not(target_arch = "wasm32"))]
        {
            pretty_env_logger::formatted_timed_builder()
                .filter_level(log::LevelFilter::Trace) // allow all levels at compile time
                .init();
            log::set_max_level(DEFAULT_LEVEL.to_level_filter());
        }
        #[cfg(target_arch = "wasm32")]
        {
            std::panic::set_hook(Box::new(console_error_panic_hook::hook));
            console_log::init_with_level(DEFAULT_LEVEL).expect("cannot initialize logger");
        }
    });
}
