use modor::{Built, EntityBuilder};
use std::time::Duration;

/// The duration of the latest update.
///
/// The physics module does not update automatically this entity.<br>
/// Instead, the delta time can be manually set to simulate time, or be automatically updated
/// by another module.
///
/// # Modor
///
/// - **Type**: singleton entity
/// - **Lifetime**: same as [`PhysicsModule`](crate::PhysicsModule)
/// - **Default if missing**: `DeltaTime::build(Duration::ZERO)`
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
    /// Builds the entity with an initial `duration`.
    pub fn build(duration: Duration) -> impl Built<Self> {
        debug!("delta time initialized to `{duration:?}`");
        EntityBuilder::new(Self { duration })
    }

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
