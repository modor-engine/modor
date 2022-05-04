use crate::UpdatesPerSecond;
use modor::{Action, Built, EntityBuilder, Single};
use std::thread;
use std::time::{Duration, Instant};

// TODO: scale has to be mandatory as it stores the absolute scale

/// The duration of the latest update.
///
/// # Modor
///
/// - **Type**: singleton entity
/// - **Lifetime**: same as [`PhysicsModule`](crate::PhysicsModule)
/// - **Updated by**: [`PhysicsModule`](crate::PhysicsModule)
/// - **Updated during**: [`UpdateDeltaTimeAction`](crate::UpdateDeltaTimeAction)
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
    fn update(&mut self, updates_per_second: Option<Single<'_, UpdatesPerSecond>>) {
        if let Some(updates_per_second) = updates_per_second {
            if updates_per_second.get() > 0 {
                let update_time = Duration::from_secs_f32(1. / f32::from(updates_per_second.get()));
                let current_update_time = Instant::now().duration_since(self.last_instant);
                if let Some(remaining_time) = update_time.checked_sub(current_update_time) {
                    thread::sleep(remaining_time);
                }
            }
        }
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
    use crate::{DeltaTime, UpdatesPerSecond};
    use modor::testing::TestApp;
    use modor::App;
    use std::thread;
    use std::time::{Duration, Instant};

    macro_rules! retry {
        ($count:literal, $block:block) => {
            for i in 0..$count {
                println!("Try #{}...", i);
                let r = std::panic::catch_unwind(|| $block);
                if r.is_ok() {
                    return;
                }
                if i == $count - 1 {
                    std::panic::resume_unwind(r.unwrap_err());
                } else {
                    std::thread::sleep(std::time::Duration::from_secs(1));
                }
            }
        };
    }

    #[test]
    fn update_without_rate_limit() {
        retry!(10, {
            let mut app: TestApp = App::new().with_entity(DeltaTime::build()).into();
            assert_correct_update(&mut app, 100, 100, 150);
        });
    }

    #[test]
    fn update_with_rate_limit_equal_to_zero() {
        retry!(10, {
            let mut app: TestApp = App::new()
                .with_entity(DeltaTime::build())
                .with_entity(UpdatesPerSecond::build(0))
                .into();
            assert_correct_update(&mut app, 100, 100, 150);
        });
    }

    #[test]
    fn update_with_rate_limit_equal_to_one() {
        retry!(10, {
            let mut app: TestApp = App::new()
                .with_entity(DeltaTime::build())
                .with_entity(UpdatesPerSecond::build(1))
                .into();
            assert_correct_update(&mut app, 500, 1000, 1200);
        });
    }

    #[test]
    fn update_with_rate_limit_greater_than_one() {
        retry!(10, {
            let mut app: TestApp = App::new()
                .with_entity(DeltaTime::build())
                .with_entity(UpdatesPerSecond::build(5))
                .into();
            assert_correct_update(&mut app, 100, 200, 300);
        });
    }

    fn assert_correct_update(
        app: &mut TestApp,
        external_sleep_millis: u64,
        min_millis: u64,
        max_millis: u64,
    ) {
        let update_start = Instant::now();
        thread::sleep(Duration::from_millis(external_sleep_millis));
        app.update();
        let update_end = Instant::now();
        app.assert_singleton::<DeltaTime>()
            .has::<DeltaTime, _>(|d| assert!(d.get() >= Duration::from_millis(min_millis)))
            .has::<DeltaTime, _>(|d| assert!(d.get() <= Duration::from_millis(max_millis)));
        assert!(update_end.duration_since(update_start) >= Duration::from_millis(min_millis));
        assert!(update_end.duration_since(update_start) <= Duration::from_millis(max_millis));
    }
}
