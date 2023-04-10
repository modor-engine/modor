use log::Level;
use scoped_threadpool::Pool;

pub(crate) fn check_catch_unwind_availability() {
    // available
}

#[allow(clippy::unnecessary_wraps)]
pub(crate) fn create_pool(thread_count: u32) -> Option<Pool> {
    Some(Pool::new(thread_count))
}

pub(crate) fn init_logging(level: Level) {
    pretty_env_logger::formatted_builder()
        .filter_level(log::LevelFilter::Trace) // allow all levels at compile time
        .init();
    log::set_max_level(level.to_level_filter());
}
