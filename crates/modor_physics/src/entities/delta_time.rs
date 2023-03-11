use std::time::Duration;

/// The duration of the latest update.
///
/// The physics module does not update automatically this entity.<br>
/// Instead, the delta time can be manually set to simulate time, or be automatically updated
/// by another module.
///
/// If the delta time is not defined or set, then its default value is `Duration::ZERO`.
///
/// # Examples
///
/// ```rust
/// # use modor_physics::*;
/// #
/// fn print_delta_time(delta_time: &DeltaTime) {
///     println!("Duration of the last update: {:?}", delta_time.get());
/// }
/// ```
#[derive(SingletonComponent, NoSystem)]
pub struct DeltaTime {
    duration: Duration,
}

impl From<Duration> for DeltaTime {
    fn from(duration: Duration) -> Self {
        debug!("delta time initialized to `{duration:?}`");
        Self { duration }
    }
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
