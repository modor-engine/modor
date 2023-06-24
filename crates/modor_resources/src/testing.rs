//! Testing utilities.

use crate::{Resource, ResourceState};
use std::thread;
use std::time::Duration;

/// Returns whether a resource is loaded, and sleeps 10ms if not yet loaded.
///
/// The resource is considered as loaded if the state is [`ResourceState::Loaded`] or
/// [`ResourceState::Error`].
///
/// # Platform-specific
///
/// - Web: sleep is not supported, so the function panics.
pub fn wait_resource_loading<R: Resource>(resource: &R) -> bool {
    if matches!(
        resource.state(),
        ResourceState::Loaded | ResourceState::Error(_)
    ) {
        true
    } else {
        thread::sleep(Duration::from_micros(10));
        false
    }
}
