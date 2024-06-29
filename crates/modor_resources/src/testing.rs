//! Testing utilities.

use crate::resource::ResGlob;
use crate::ResourceState;
use modor::{App, Globals};
use std::thread;
use std::time::Duration;

/// Wait until all resources are loaded.
///
/// A resource is considered as loaded if its state is [`ResourceState::Loaded`] or
/// [`ResourceState::Error`].
///
/// # Platform-specific
///
/// - Web: sleep is not supported, so the function panics.
pub fn wait_resources(app: &mut App) {
    app.update();
    while app
        .get_mut::<Globals<ResGlob>>()
        .iter()
        .any(|res| res.state == ResourceState::Loading)
    {
        app.update();
        thread::sleep(Duration::from_micros(10));
    }
}
