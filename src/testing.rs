//! Testing utilities.

use crate::storages::archetypes::{ArchetypeStorage, EntityLocation};
use crate::storages::core::CoreStorage;
use crate::{App, EntityBuilder, EntityMainComponent, Singleton};
use std::any::{Any, TypeId};
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
///     .has_not::<usize>()
///     .has_children(|c| assert!(c.is_empty()));
///
/// struct Button;
///
/// impl EntityMainComponent for Button {
///     type Type = ();
///     type Data = String;
///
///     fn build(builder: EntityBuilder<'_, Self>, name: Self::Data) -> Built<'_> {
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

    /// Creates a new entity and returns its ID.
    ///
    /// Entity IDs are unique and can be recycled in case the entity is deleted.
    pub fn create_entity<E>(&mut self, data: E::Data) -> usize
    where
        E: EntityMainComponent,
    {
        let core = &mut self.0 .0;
        let location = core.create_entity(ArchetypeStorage::DEFAULT_IDX, None).1;
        let entity_idx = core.archetypes().entity_idxs(location.idx)[location.pos];
        E::build(
            EntityBuilder::<_, ()>::from_existing(core, entity_idx),
            data,
        );
        entity_idx.into()
    }

    /// Starts assertions on an entity.
    pub fn assert_entity(&self, entity_id: usize) -> EntityAssertion<'_> {
        // TODO: avoid `.0`
        EntityAssertion {
            core: &self.0 .0,
            location: self.0 .0.entities().location(entity_id.into()),
        }
    }

    /// Starts assertions on a singleton of type `C`.
    pub fn assert_singleton<C>(&self) -> EntityAssertion<'_>
    where
        C: EntityMainComponent<Type = Singleton>,
    {
        EntityAssertion {
            core: &self.0 .0,
            location: self
                .0
                 .0
                .components()
                .type_idx(TypeId::of::<C>())
                .and_then(|c| self.0 .0.components().singleton_locations(c)),
        }
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
    location: Option<EntityLocation>,
}

impl EntityAssertion<'_> {
    /// Asserts the entity exists.
    ///
    /// # Panics
    ///
    /// This will panic if the entity does not exist.
    pub fn exists(self) -> Self {
        assert!(
            self.location.is_some(),
            "assertion failed: entity expected to exist",
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
            self.location.is_none(),
            "assertion failed: entity expected to not exist",
        );
    }

    /// Asserts the entity has a component of type `C` and runs `f` on this component.
    ///
    /// # Panics
    ///
    /// This will panic if the entity does not exist or has not a component of type `C`.
    pub fn has<C, F>(self, f: F) -> Self
    where
        C: Any,
        F: FnMut(&C),
    {
        let location = self
            .location
            .unwrap_or_else(|| panic!("assertion failed: entity expected to exist",));
        assert!(
            self.test_component_exists::<C, F>(location, f).is_some(),
            "assertion failed: component expected to exist in entity)",
        );
        self
    }

    /// Asserts the entity has not a component of type `C`.
    ///
    /// # Panics
    ///
    /// This will panic if the entity does not exist or has a component of type `C`.
    pub fn has_not<C>(self) -> Self
    where
        C: Any,
    {
        let location = self
            .location
            .unwrap_or_else(|| panic!("assertion failed: entity expected to exist",));
        assert!(
            self.test_component_exists::<C, _>(location, |_| ())
                .is_none(),
            "assertion failed: component expected to exist in entity",
        );
        self
    }

    /// Runs `f` that takes as parameter the list of child IDs of the entity.
    ///
    /// # Panics
    ///
    /// This will panic if the entity does not exist.
    pub fn has_children<F>(self, mut f: F) -> Self
    where
        F: FnMut(Vec<usize>),
    {
        let location = self
            .location
            .expect("assertion failed: entity expected to exist");
        let child_ids = self
            .core
            .entities()
            .child_idxs(self.core.archetypes().entity_idxs(location.idx)[location.pos])
            .iter()
            .copied()
            .map(usize::from)
            .collect();
        f(child_ids);
        self
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
    use crate::{App, Built, EntityBuilder, EntityMainComponent, Singleton};
    use std::ptr;

    #[derive(Debug, PartialEq)]
    struct TestEntity(String);

    impl EntityMainComponent for TestEntity {
        type Type = ();
        type Data = String;

        fn build(builder: EntityBuilder<'_, Self>, data: Self::Data) -> Built<'_> {
            builder
                .with_child::<ChildEntity>(10)
                .with_child::<ChildEntity>(20)
                .with_self(Self(data))
        }
    }

    #[derive(Debug, PartialEq)]
    struct ChildEntity(u32);

    impl EntityMainComponent for ChildEntity {
        type Type = ();
        type Data = u32;

        fn build(builder: EntityBuilder<'_, Self>, data: Self::Data) -> Built<'_> {
            builder.with_self(Self(data))
        }
    }

    #[derive(Debug, PartialEq)]
    struct SingletonEntity1(u32);

    impl EntityMainComponent for SingletonEntity1 {
        type Type = Singleton;
        type Data = u32;

        fn build(builder: EntityBuilder<'_, Self>, data: Self::Data) -> Built<'_> {
            builder.with_self(Self(data))
        }
    }

    // TODO: use macro to generate test entities
    #[derive(Debug, PartialEq)]
    struct SingletonEntity2(u32);

    impl EntityMainComponent for SingletonEntity2 {
        type Type = Singleton;
        type Data = u32;

        fn build(builder: EntityBuilder<'_, Self>, data: Self::Data) -> Built<'_> {
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
    fn assert_on_entities_from_new_app() {
        let mut app = TestApp::new();
        let entity_id = app.create_entity::<TestEntity>("string".into());
        app.assert_entity(entity_id)
            .exists()
            .has::<TestEntity, _>(|c| assert_eq!(c.0, "string"))
            .has_not::<String>()
            .has_children(|c| assert_eq!(c, [1, 2]));
        assert_panics!(app.assert_entity(entity_id).does_not_exist());
        assert_panics!(app.assert_entity(entity_id).has::<String, _>(|_| ()));
        assert_panics!(app
            .assert_entity(entity_id)
            .has::<TestEntity, _>(|_| panic!()));
        assert_panics!(app.assert_entity(entity_id).has_children(|_| panic!()));
        let missing_id = 10;
        app.assert_entity(missing_id).does_not_exist();
        assert_panics!(app.assert_entity(entity_id).has_not::<TestEntity>());
        assert_panics!(app.assert_entity(missing_id).exists());
        assert_panics!(app.assert_entity(missing_id).has::<String, _>(|_| ()));
        assert_panics!(app.assert_entity(missing_id).has_not::<String>());
        assert_panics!(app.assert_entity(missing_id).has_children(|_| ()));
    }

    #[test]
    fn assert_on_singleton_from_new_app() {
        let mut app = TestApp::new();
        app.create_entity::<SingletonEntity1>(10);
        app.assert_singleton::<SingletonEntity2>().does_not_exist();
        app.assert_singleton::<SingletonEntity1>().exists();
        assert_panics!(app.assert_singleton::<SingletonEntity1>().does_not_exist());
        assert_panics!(app.assert_singleton::<SingletonEntity2>().exists());
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
