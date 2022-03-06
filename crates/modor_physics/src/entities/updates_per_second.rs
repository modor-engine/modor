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
///     .with_entity::<PhysicsModule>(())
///     .with_entity::<UpdatesPerSecond>(60.);
/// loop {
///     app.update(); // Limited to 60 updates per second
///     # break;
/// }
/// ```
pub struct UpdatesPerSecond(f32);

impl EntityMainComponent for UpdatesPerSecond {
    type Type = Singleton;
    type Data = f32;

    fn build(builder: EntityBuilder<'_, Self>, updates_per_second: Self::Data) -> Built<'_> {
        builder.with_self(Self(updates_per_second))
    }
}

impl UpdatesPerSecond {
    /// Returns the number of updates per second.
    pub fn get(&self) -> f32 {
        self.0
    }

    /// Set the number of updates per second.
    pub fn set(&mut self, updates_per_second: f32) {
        self.0 = updates_per_second;
    }
}

#[cfg(test)]
mod updates_per_second_tests {
    use super::UpdatesPerSecond;
    use approx::assert_abs_diff_eq;
    use modor::testing::TestApp;
    use modor::App;

    #[test]
    fn build() {
        let app: TestApp = App::new().with_entity::<UpdatesPerSecond>(60.).into();
        app.assert_singleton::<UpdatesPerSecond>()
            .has::<UpdatesPerSecond, _>(|u| assert_abs_diff_eq!(u.get(), 60.));
    }

    #[test]
    fn use_() {
        let mut entity = UpdatesPerSecond(60.);
        assert_abs_diff_eq!(entity.get(), 60.);
        entity.set(30.);
        assert_abs_diff_eq!(entity.get(), 30.);
    }
}
