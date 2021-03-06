use modor::{Built, EntityBuilder};
use std::time::Duration;

/// The duration of the latest update.
///
/// Default value is zero.
///
/// The physics module does not update automatically this entity.<br>
/// Instead, the delta time can be manually set to simulate time, or be automatically updated
/// by another module.
///
/// # Modor
///
/// - **Type**: singleton entity
/// - **Lifetime**: same as [`PhysicsModule`](crate::PhysicsModule)
///
/// # Examples
///
/// ```rust
/// # use modor_physics::DeltaTime;
/// #
/// fn print_delta_time(delta_time: &DeltaTime) {
///     println!("Duration of the last update: {:?}", delta_time.get());
/// }
/// ```
pub struct DeltaTime {
    duration: Duration,
}

#[singleton]
impl DeltaTime {
    /// Returns the duration of the last update.
    #[must_use]
    pub fn get(&self) -> Duration {
        self.duration
    }

    /// Set the duration of the last update.
    pub fn set(&mut self, duration: Duration) {
        self.duration = duration;
    }

    pub(crate) fn build() -> impl Built<Self> {
        EntityBuilder::new(Self {
            duration: Duration::ZERO,
        })
    }
}
