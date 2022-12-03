use log::Level;
use std::sync::Once;

const DEFAULT_LEVEL: Level = Level::Warn;

static INIT: Once = Once::new();

pub(crate) fn init_logging() {
    INIT.call_once(|| {
        #[cfg(not(target_arch = "wasm32"))]
        {
            pretty_env_logger::formatted_builder()
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

pub(crate) fn merge<T, const N: usize>(vectors: [Vec<T>; N]) -> Vec<T> {
    let mut merged_vec = Vec::new();
    for vec in vectors {
        merged_vec.extend(vec);
    }
    merged_vec
}

macro_rules! run_for_tuples_with_idxs {
    ($macro:ident) => {
        run_for_tuples_with_idxs!(
            @internal
            $macro,
            ((A, 0)),
            ((B, 1), (C, 2), (D, 3), (E, 4), (F, 5), (G, 6), (H, 7), (I, 8), (J, 9))
        );
    };
    (
        @internal
        $macro:ident,
        ($(($generics:ident, $indexes:tt)),+),
        (($next_generic:ident, $next_index:tt) $(,($last_generics:ident, $last_index:tt))*)
    ) => {
        $macro!($(($generics, $indexes)),+);
        run_for_tuples_with_idxs!(
            @internal
            $macro,
            ($(($generics, $indexes)),+, ($next_generic, $next_index)),
            ($(($last_generics, $last_index)),*)
        );
    };
    (@internal $macro:ident, ($(($generics:ident, $indexes:tt)),+), ()) => {
        $macro!($(($generics, $indexes)),+);
    };
}
