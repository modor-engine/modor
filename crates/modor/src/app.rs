use crate::storages::core::CoreStorage;
use crate::{Built, EntityMainComponent};

/// The entrypoint of the engine.
///
/// # Examples
///
/// ```rust
/// # #[macro_use]
/// # extern crate modor;
/// #
/// # use modor::*;
/// #
/// fn main() {
///     let mut app = App::new()
///         .with_thread_count(4)
///         .with_entity(Button::build("New game".into()))
///         .with_entity(Button::build("Settings".into()))
///         .with_entity(Button::build("Exit".into()));
///     app.update();
/// }
///
/// struct Button;
///
/// #[entity]
/// impl Button {
///     fn build(label: String) -> impl Built<Self> {
///         EntityBuilder::new(Self).with(label)
///     }
/// }
/// ```
#[derive(Default)]
pub struct App {
    pub(crate) core: CoreStorage,
}

impl App {
    /// Creates a new empty `App`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Changes the number of threads used by the `App` during update.
    ///
    /// Update is only done in one thread if `count` is `0` or `1`,
    /// which is the default behavior.
    pub fn with_thread_count(mut self, count: u32) -> Self {
        self.core.set_thread_count(count);
        self
    }

    /// Creates a new entity with main component of type `E`.
    pub fn with_entity<E, B>(mut self, entity: B) -> Self
    where
        E: EntityMainComponent,
        B: Built<E>,
    {
        entity.build(&mut self.core, None);
        self
    }

    /// Returns the number of threads used by the `App` during update.
    pub fn thread_count(&self) -> u32 {
        self.core.systems().thread_count()
    }

    /// Runs all systems registered in the `App`.
    pub fn update(&mut self) {
        self.core.update();
    }
}

#[cfg(test)]
mod app_tests {
    use crate::storages::systems::SystemProperties;
    use crate::{
        App, EntityBuilder, EntityMainComponent, NotSingleton, SystemBuilder, SystemRunner,
    };

    #[derive(Debug, PartialEq, Clone)]
    struct TestEntity(u32);

    impl EntityMainComponent for TestEntity {
        type Type = NotSingleton;

        fn on_update(runner: SystemRunner<'_>) -> SystemRunner<'_> {
            runner.run(SystemBuilder {
                properties_fn: |_| SystemProperties {
                    component_types: vec![],
                    can_update: false,
                    filtered_component_type_idxs: vec![],
                },
                wrapper: |d, _| d.updates.try_lock().unwrap().delete_entity(0.into()),
            })
        }
    }

    assert_impl_all!(App: Send, Unpin);

    #[test]
    fn configure_app() {
        let mut app = App::new()
            .with_thread_count(2)
            .with_entity(EntityBuilder::new(TestEntity(10)));
        assert_eq!(app.thread_count(), 2);
        let components = (&*app.core.components().read_components::<TestEntity>()).clone();
        let expected_components = ti_vec![ti_vec![], ti_vec![TestEntity(10)]];
        assert_eq!(components, expected_components);
        app.update();
        let components = (&*app.core.components().read_components::<TestEntity>()).clone();
        let expected_components = ti_vec![ti_vec![], ti_vec![]];
        assert_eq!(components, expected_components);
    }
}
