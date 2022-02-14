use crate::storages::archetypes::ArchetypeStorage;
use crate::storages::core::CoreStorage;
use crate::{EntityBuilder, EntityMainComponent};
use std::marker::PhantomData;

/// The entrypoint of the engine.
///
/// # Examples
///
/// ```rust
/// # use modor::*;
/// #
/// fn main() {
///     let mut app = App::new()
///         .with_thread_count(4)
///         .with_entity::<Button>("New game".into())
///         .with_entity::<Button>("Settings".into())
///         .with_entity::<Button>("Exit".into());
///     app.update();
/// }
///
/// struct Button;
///
/// impl EntityMainComponent for Button {
///     type Data = String;
///
///     fn build(builder: EntityBuilder<'_, Self>, label: Self::Data) -> Built {
///         builder
///             .with(label)
///             .with_self(Self)
///     }
/// }
/// ```
#[derive(Default)]
pub struct App(pub(crate) CoreStorage);

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
        self.0.set_thread_count(count);
        self
    }

    /// Creates a new entity with main component of type `E` and building data `data`.
    pub fn with_entity<E>(mut self, data: E::Data) -> Self
    where
        E: EntityMainComponent,
    {
        let entity_builder = EntityBuilder {
            core: &mut self.0,
            entity_idx: None,
            src_location: None,
            dst_archetype_idx: ArchetypeStorage::DEFAULT_IDX,
            parent_idx: None,
            added_components: (),
            phantom: PhantomData,
        };
        E::build(entity_builder, data);
        self
    }

    /// Returns the number of threads used by the `App` during update.
    pub fn thread_count(&self) -> u32 {
        self.0.systems().thread_count()
    }

    /// Runs all systems registered in the `App`.
    pub fn update(&mut self) {
        self.0.update();
    }
}

#[cfg(test)]
mod app_tests {
    use crate::storages::archetypes::ArchetypeFilter;
    use crate::storages::systems::SystemProperties;
    use crate::{App, Built, EntityBuilder, EntityMainComponent, EntityRunner, SystemBuilder};

    assert_impl_all!(App: Send, Unpin);

    #[derive(Debug, PartialEq, Clone)]
    struct TestEntity(String);

    impl EntityMainComponent for TestEntity {
        type Data = String;

        fn build(builder: EntityBuilder<'_, Self>, data: Self::Data) -> Built {
            builder.with_self(Self(data))
        }

        fn on_update(runner: EntityRunner<'_, Self>) {
            runner.run(SystemBuilder {
                properties_fn: |_| SystemProperties {
                    component_types: vec![],
                    can_update: false,
                    archetype_filter: ArchetypeFilter::None,
                },
                wrapper: |d, _| d.updates.try_lock().unwrap().delete_entity(0.into()),
            });
        }
    }

    #[test]
    fn configure_app() {
        let mut app = App::new()
            .with_thread_count(2)
            .with_entity::<TestEntity>("string".into());
        assert_eq!(app.thread_count(), 2);
        let components = (&*app.0.components().read_components::<TestEntity>()).clone();
        let expected_components = ti_vec![ti_vec![], ti_vec![TestEntity("string".into())]];
        assert_eq!(components, expected_components);
        app.update();
        let components = (&*app.0.components().read_components::<TestEntity>()).clone();
        let expected_components = ti_vec![ti_vec![], ti_vec![]];
        assert_eq!(components, expected_components);
    }
}
