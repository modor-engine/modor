//! Testing utilities.

use fxhash::FxHashSet;
use modor::{App, FromApp, State};
use std::thread;
use std::time::Duration;

/// Wait until all resources are loaded.
///
/// A resource is considered as loaded if its state is
/// [`ResourceState::Loaded`](crate::ResourceState::Loaded) or
/// [`ResourceState::Error`](crate::ResourceState::Error).
///
/// # Platform-specific
///
/// - Web: sleep is not supported, so the function panics.
pub fn wait_resources(app: &mut App) {
    app.update();
    while !app.take::<ResourceStates, _>(|states, app| states.are_all_loaded(app)) {
        app.update();
        thread::sleep(Duration::from_micros(10));
    }
}

#[derive(FromApp, State)]
pub(crate) struct ResourceStates {
    pub(crate) are_all_loaded_fns: FxHashSet<fn(app: &mut App) -> bool>,
}

impl ResourceStates {
    fn are_all_loaded(&self, app: &mut App) -> bool {
        self.are_all_loaded_fns.iter().all(|f| f(app))
    }
}
