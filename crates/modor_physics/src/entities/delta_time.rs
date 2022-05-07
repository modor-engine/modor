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

#[cfg(test)]
mod updates_per_second_tests {
    use crate::DeltaTime;
    use std::time::Duration;

    #[test]
    fn use_delta_time() {
        let mut delta_time = DeltaTime {
            duration: Duration::ZERO,
        };
        assert_eq!(delta_time.get(), Duration::ZERO);
        delta_time.set(Duration::from_millis(10));
        assert_eq!(delta_time.get(), Duration::from_millis(10));
    }
}
