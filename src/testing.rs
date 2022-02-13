//! Testing utilities.

use crate::storages::archetypes::{ArchetypeStorage, EntityLocation};
use crate::storages::core::CoreStorage;
use crate::storages::entities::EntityIdx;
use crate::{App, EntityBuilder, EntityMainComponent};
use std::any;
use std::any::{Any, TypeId};
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};

/// A utility to facilitate entity testing.
///
/// This is an augmented [`App`](crate::App) type.
///
/// # Examples
///
/// ```rust
/// # use modor::testing::TestApp;
/// # use modor::{EntityMainComponent, EntityBuilder, Built};
/// #
/// let mut app = TestApp::new();
/// let entity_id = app.create_entity::<Button>("name".into());
/// app.assert_entity(entity_id)
///     .exists()
///     .has::<String, _>(|name| assert_eq!(name, "name"))
///     .has::<Button, _>(|_| ())
///     .has_not::<usize>();
///
/// struct Button;
///
/// impl EntityMainComponent for Button {
///     type Data = String;
///
///     fn build(builder: EntityBuilder<'_, Self>, name: Self::Data) -> Built {
///         builder
///             .with(name)
///             .with_self(Self)
///     }
/// }
/// ```
#[derive(Default)]
pub struct TestApp(App);

impl TestApp {
    /// Creates a new empty `TestApp`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Starts assertions on an entity.
    pub fn assert_entity(&self, entity_id: usize) -> EntityAssertion<'_> {
        EntityAssertion {
            core: &self.0 .0,
            entity_idx: entity_id.into(),
        }
    }

    /// Creates an entity and returns its ID.
    ///
    /// Entity IDs are unique and can be recycled in case the entity is deleted.
    pub fn create_entity<E>(&mut self, data: E::Data) -> usize
    where
        E: EntityMainComponent,
    {
        let location = self.0 .0.create_entity(ArchetypeStorage::DEFAULT_IDX).1;
        let entity_idx = self.0 .0.archetypes().entity_idxs(location.idx)[location.pos];
        let entity_builder = EntityBuilder {
            core: &mut self.0 .0,
            src_location: Some(location),
            dst_archetype_idx: ArchetypeStorage::DEFAULT_IDX,
            added_components: (),
            phantom: PhantomData,
        };
        E::build(entity_builder, data);
        entity_idx.into()
    }
}

impl From<App> for TestApp {
    fn from(app: App) -> Self {
        Self(app)
    }
}

impl Deref for TestApp {
    type Target = App;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for TestApp {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

/// A utility to assert on an entity.
///
/// # Examples
///
/// See [`TestApp`](crate::testing::TestApp).
pub struct EntityAssertion<'a> {
    core: &'a CoreStorage,
    entity_idx: EntityIdx,
}

impl EntityAssertion<'_> {
    /// Asserts the entity exists.
    ///
    /// # Panics
    ///
    /// This will panic if the entity does not exist.
    pub fn exists(self) -> Self {
        assert!(
            self.location().is_some(),
            "assertion failed: assert_entity({}).exists()",
            usize::from(self.entity_idx)
        );
        self
    }

    /// Asserts the entity does not exist.
    ///
    /// # Panics
    ///
    /// This will panic if the entity exists.
    pub fn does_not_exist(self) {
        assert!(
            self.location().is_none(),
            "assertion failed: assert_entity({}).does_not_exist()",
            usize::from(self.entity_idx)
        );
    }

    /// Asserts the entity has a component of type `C` and runs `f` on this component.
    ///
    /// # Panics
    ///
    /// This will panic if the entity does not exists or has not a component of type `C`.
    pub fn has<C, F>(self, f: F) -> Self
    where
        C: Any,
        F: FnMut(&C),
    {
        let location = self.location().unwrap_or_else(|| {
            panic!(
                "assertion failed: assert_entity({}).has<{}, _>(...) (entity does not exist)",
                usize::from(self.entity_idx),
                any::type_name::<C>()
            )
        });
        assert!(
            self.test_component_exists::<C, F>(location, f).is_some(),
            "assertion failed: assert_entity({}).has<{}, _>(...) (missing component in entity)",
            usize::from(self.entity_idx),
            any::type_name::<C>()
        );
        self
    }

    /// Asserts the entity has not a component of type `C`.
    ///
    /// # Panics
    ///
    /// This will panic if the entity does not exists or has a component of type `C`.
    pub fn has_not<C>(self) -> Self
    where
        C: Any,
    {
        let location = self.location().unwrap_or_else(|| {
            panic!(
                "assertion failed: assert_entity({}).has_not<{}>() (entity does not exist)",
                usize::from(self.entity_idx),
                any::type_name::<C>()
            )
        });
        assert!(
            self.test_component_exists::<C, _>(location, |_| ())
                .is_none(),
            "assertion failed: assert_entity({}).has_not<{}>() (existing component in entity)",
            usize::from(self.entity_idx),
            any::type_name::<C>()
        );
        self
    }

    fn location(&self) -> Option<EntityLocation> {
        self.core.entities().location(self.entity_idx)
    }

    fn test_component_exists<C, F>(&self, location: EntityLocation, mut f: F) -> Option<()>
    where
        C: Any,
        F: FnMut(&C),
    {
        self.core.components().type_idx(TypeId::of::<C>())?;
        let components = self.core.components().read_components::<C>();
        let component = components.get(location.idx)?.get(location.pos)?;
        f(component);
        Some(())
    }
}

#[cfg(test)]
mod test_app_tests {
    use crate::testing::TestApp;
    use crate::{App, Built, EntityBuilder, EntityMainComponent};
    use std::ptr;

    #[derive(Debug, PartialEq)]
    struct TestEntity(String);

    impl EntityMainComponent for TestEntity {
        type Data = String;

        fn build(builder: EntityBuilder<'_, Self>, data: Self::Data) -> Built {
            builder.with_self(Self(data))
        }
    }

    #[test]
    fn deref() {
        let mut app = TestApp::from(App::new());
        assert!(ptr::eq(&*app, &app.0));
        assert!(ptr::eq(&mut *app as *const App, &app.0));
    }

    #[test]
    fn assert_from_new_app() {
        let mut app = TestApp::new();
        let entity_id = app.create_entity::<TestEntity>("string".into());
        app.assert_entity(entity_id)
            .exists()
            .has::<TestEntity, _>(|c| assert_eq!(c.0, "string"))
            .has_not::<String>();
        app.assert_entity(1).does_not_exist();
        assert_panics!(app.assert_entity(entity_id).does_not_exist());
        assert_panics!(app.assert_entity(entity_id).has::<String, _>(|_| ()));
        assert_panics!(app
            .assert_entity(entity_id)
            .has::<TestEntity, _>(|_| panic!()));
        assert_panics!(app.assert_entity(entity_id).has_not::<TestEntity>());
        assert_panics!(app.assert_entity(1).exists());
        assert_panics!(app.assert_entity(1).has::<String, _>(|_| ()));
        assert_panics!(app.assert_entity(1).has_not::<String>());
    }

    #[test]
    fn assert_from_existing_app() {
        let existing_app = App::new().with_entity::<TestEntity>("string".into());
        let mut app = TestApp::from(existing_app);
        app.assert_entity(0).exists();
        assert!(ptr::eq(&*app, &app.0));
        assert!(ptr::eq(&mut *app as *const App, &app.0));
    }
}
