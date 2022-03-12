use modor::{Built, EntityBuilder, EntityMainComponent, Singleton};

/// An entity main component used to limit the number of updates per second.
///
/// # Examples
///
/// ```rust
/// # use modor::App;
/// # use modor_physics::{PhysicsModule, UpdatesPerSecond};
/// #
/// let mut app = App::new()
///     .with_entity(PhysicsModule::build())
///     .with_entity(UpdatesPerSecond::build(60));
/// loop {
///     app.update(); // Limited to 60 updates per second
///     # break;
/// }
/// ```
pub struct UpdatesPerSecond(u16);

impl UpdatesPerSecond {
    /// Builds the entity.
    pub fn build(updates_per_second: u16) -> impl Built<Self> {
        EntityBuilder::new(Self(updates_per_second))
    }

    /// Returns the number of updates per second.
    pub fn get(&self) -> u16 {
        self.0
    }

    /// Set the number of updates per second.
    pub fn set(&mut self, updates_per_second: u16) {
        self.0 = updates_per_second;
    }
}

impl EntityMainComponent for UpdatesPerSecond {
    type Type = Singleton;
}

#[cfg(test)]
mod updates_per_second_tests {
    use super::UpdatesPerSecond;
    use approx::assert_abs_diff_eq;
    use modor::testing::TestApp;
    use modor::App;

    #[test]
    fn build() {
        let app: TestApp = App::new().with_entity(UpdatesPerSecond::build(60)).into();
        app.assert_singleton::<UpdatesPerSecond>()
            .has::<UpdatesPerSecond, _>(|u| assert_abs_diff_eq!(u.get(), 60));
    }

    #[test]
    fn use_() {
        let mut entity = UpdatesPerSecond(60);
        assert_eq!(entity.get(), 60);
        entity.set(30);
        assert_eq!(entity.get(), 30);
    }
}
