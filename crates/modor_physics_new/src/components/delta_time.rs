use std::time::Duration;

/// The duration of the latest update.
///
/// The physics module does not update automatically this entity.<br>
/// Instead, the delta time can be manually set to simulate time, or be automatically updated
/// by another module (e.g. graphics module).
///
/// Default value is `Duration::ZERO`.
///
/// # Requirements
///
/// The component is created only if:
/// - physics [`module`](crate::module()) is initialized
///
/// # Examples
///
/// ```rust
/// # use modor::*;
/// # use modor_physics_new::*;
/// #
/// fn print_delta_time(delta_time: &SingleRef<'_, '_, DeltaTime>) {
///     println!("Duration of the last update: {:?}", delta_time.get().get());
/// }
/// ```
#[derive(SingletonComponent, NoSystem)]
pub struct DeltaTime {
    pub(crate) duration: Duration,
}

impl DeltaTime {
    /// Returns the duration of the last update.
    pub fn get(&self) -> Duration {
        self.duration
    }

    /// Set the duration of the last update.
    pub fn set(&mut self, duration: Duration) {
        self.duration = duration;
        trace!("delta time set to `{duration:?}`");
    }
}
