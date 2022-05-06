use modor::{Action, Built, EntityBuilder};
use std::time::{Duration, Instant};

// TODO: compute it more accurately

/// The duration of the latest update.
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
    previous_instant: Instant,
    last_instant: Instant,
}

#[singleton]
impl DeltaTime {
    /// Returns the duration of the last update.
    pub fn get(&self) -> Duration {
        self.last_instant.duration_since(self.previous_instant)
    }

    pub(crate) fn build() -> impl Built<Self> {
        let now = Instant::now();
        EntityBuilder::new(Self {
            previous_instant: now,
            last_instant: now,
        })
    }

    #[run_as(UpdateDeltaTimeAction)]
    fn update(&mut self) {
        self.previous_instant = self.last_instant;
        self.last_instant = Instant::now();
    }
}

/// An action done when the delta time has been updated.
pub struct UpdateDeltaTimeAction;

impl Action for UpdateDeltaTimeAction {
    type Constraint = ();
}

#[cfg(test)]
mod updates_per_second_tests {
    use crate::DeltaTime;
    use modor::testing::TestApp;
    use modor::App;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn assert_correct_update() {
        modor_internal::retry!(10, {
            let mut app: TestApp = App::new().with_entity(DeltaTime::build()).into();
            thread::sleep(Duration::from_millis(100));
            app.update();
            app.assert_singleton::<DeltaTime>()
                .has::<DeltaTime, _>(|d| assert!(d.get() >= Duration::from_millis(100)))
                .has::<DeltaTime, _>(|d| assert!(d.get() <= Duration::from_millis(150)));
        });
    }
}
