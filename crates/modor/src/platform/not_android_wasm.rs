use log::Level;

pub(crate) fn init_logging(level: Level) {
    let _ = pretty_env_logger::formatted_builder()
        .filter_level(log::LevelFilter::Trace) // allow all levels at compile time
        .try_init();
    log::set_max_level(level.to_level_filter());
}
