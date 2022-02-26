//! Testing utilities.

use crate::storages::archetypes::{ArchetypeStorage, EntityLocation};
use crate::storages::core::CoreStorage;
use crate::storages::entities::EntityIdx;
use crate::{App, EntityBuilder, EntityMainComponent, Global};
use std::any;
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

    /// Creates a new global of type `G`.
    ///
    /// If a global of type `G` already exists, it is overwritten.
    pub fn create_global<G>(&mut self, global: G)
    where
        G: Global,
    {
        App::create_global(&mut self.0 .0, global);
    }

    /// Starts assertions on an entity.
    pub fn assert_entity(&self, entity_id: usize) -> EntityAssertion<'_> {
        EntityAssertion {
            core: &self.0 .0,
            entity_idx: entity_id.into(),
        }
    }

    /// Asserts the global of type `G` exists and runs `f` on this global.
    ///
    /// # Panics
    ///
    /// This will panic if the global does not exist.
    pub fn assert_global_exists<G, F>(&self, mut f: F)
    where
        G: Global,
        F: FnMut(&G),
    {
        if let Some(global) = self.0 .0.globals().read::<G>() {
            f(&*global);
        } else {
            panic!(
                "assertion failed: assert_global_exists<{}, _>(...)",
                any::type_name::<G>()
            )
        }
    }

    /// Asserts the global of type `G` does not exist.
    ///
    /// # Panics
    ///
    /// This will panic if the global exists.
    pub fn assert_global_does_not_exist<G>(&self)
    where
        G: Global,
    {
        assert!(
            self.0 .0.globals().read::<G>().is_none(),
            "assertion failed: assert_global_does_not_exist<{}>()",
            any::type_name::<G>()
        );
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
    /// This will panic if the entity does not exist or has not a component of type `C`.
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
    /// This will panic if the entity does not exist or has a component of type `C`.
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

    /// Runs `f` that takes as parameter the list of child IDs of the entity.
    ///
    /// # Panics
    ///
    /// This will panic if the entity does not exist.
    pub fn has_children<F>(self, mut f: F) -> Self
    where
        F: FnMut(Vec<usize>),
    {
        assert!(
            self.location().is_some(),
            "assertion failed: assert_entity({}).has_children<_>() (entity does not exist)",
            usize::from(self.entity_idx)
        );
        let child_ids = self
            .core
            .entities()
            .child_idxs(self.entity_idx)
            .iter()
            .copied()
            .map(usize::from)
            .collect();
        f(child_ids);
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
    use crate::{App, Built, EntityBuilder, EntityMainComponent, Global};
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

    struct TestGlobal1(u32);

    impl Global for TestGlobal1 {}

    struct TestGlobal2(u32);

    impl Global for TestGlobal2 {}

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
    fn assert_on_globals_from_new_app() {
        let mut app = TestApp::new();
        app.create_global(TestGlobal1(10));
        app.assert_global_does_not_exist::<TestGlobal2>();
        app.assert_global_exists::<TestGlobal1, _>(|g| assert_eq!(g.0, 10));
        assert_panics!(app.assert_global_does_not_exist::<TestGlobal1>());
        assert_panics!(app.assert_global_exists::<TestGlobal1, _>(|_| panic!()));
        assert_panics!(app.assert_global_exists::<TestGlobal2, _>(|_| ()));
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
