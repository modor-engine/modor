//! Testing utilities.

use crate::{Res, Resource, ResourceState};
use modor::{App, RootNode};
use std::thread;
use std::time::Duration;

/// Wait until the resource returned by `f` is loaded.
///
/// The resource is considered as loaded if the state is [`ResourceState::Loaded`] or
/// [`ResourceState::Error`].
///
/// # Platform-specific
///
/// - Web: sleep is not supported, so the function panics.
pub fn wait_resource<T, R>(app: &mut App, f: fn(&T) -> &Res<R>)
where
    T: RootNode,
    R: Resource,
{
    while f(app.get_mut::<T>()).state() == &ResourceState::Loading {
        app.update();
        thread::sleep(Duration::from_micros(10));
    }
}
