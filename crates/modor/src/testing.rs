//! Testing utilities.

use crate::storages::archetypes::EntityLocation;
use crate::storages::core::CoreStorage;
use crate::{App, Built, EntityMainComponent, Singleton};
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
/// # use modor::{EntityMainComponent, EntityBuilder, Built, entity};
/// #
/// let mut app = TestApp::new();
/// let entity_id = app.create_entity(Button::build("Play".into()));
/// app.assert_entity(entity_id)
///     .exists()
///     .has::<String, _>(|name| assert_eq!(name, "Play"))
///     .has::<Button, _>(|_| ())
///     .has_not::<usize>()
///     .has_children(|c| assert!(c.is_empty()));
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
pub struct TestApp {
    app: App,
}

impl TestApp {
    /// Creates a new empty `TestApp`.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a new entity and returns its ID.
    ///
    /// Entity IDs are unique and can be recycled in case the entity is deleted.
    pub fn create_entity<E, B>(&mut self, entity: B) -> usize
    where
        E: EntityMainComponent,
        B: Built<E>,
    {
        entity.build(&mut self.app.core, None).into()
    }

    /// Creates a new child entity and returns its ID.
    ///
    /// Entity IDs are unique and can be recycled in case the entity is deleted.
    ///
    /// # Panics
    ///
    /// This will panic if the parent entity does not exist.
    pub fn create_child<E, B>(&mut self, parent_id: usize, entity: B) -> usize
    where
        E: EntityMainComponent,
        B: Built<E>,
    {
        let core = &mut self.app.core;
        assert!(
            core.entities().location(parent_id.into()).is_some(),
            "parent entity with ID {} does not exist",
            parent_id
        );
        entity
            .build(&mut self.app.core, Some(parent_id.into()))
            .into()
    }

    /// Starts assertions on an entity.
    pub fn assert_entity(&self, entity_id: usize) -> EntityAssertion<'_> {
        EntityAssertion {
            core: &self.app.core,
            location: self.app.core.entities().location(entity_id.into()),
        }
    }

    /// Starts assertions on a singleton of type `C`.
    pub fn assert_singleton<C>(&self) -> EntityAssertion<'_>
    where
        C: EntityMainComponent<Type = Singleton>,
    {
        let core = &self.app.core;
        EntityAssertion {
            core,
            location: core
                .components()
                .type_idx(TypeId::of::<C>())
                .and_then(|c| core.components().singleton_location(c)),
        }
    }
}

impl From<App> for TestApp {
    fn from(app: App) -> Self {
        Self { app }
    }
}

impl From<TestApp> for App {
    fn from(app: TestApp) -> Self {
        app.app
    }
}

impl Deref for TestApp {
    type Target = App;

    fn deref(&self) -> &Self::Target {
        &self.app
    }
}

impl DerefMut for TestApp {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.app
    }
}

/// A utility for asserting on an entity.
///
/// # Examples
///
/// See [`TestApp`](crate::testing::TestApp).
pub struct EntityAssertion<'a> {
    core: &'a CoreStorage,
    location: Option<EntityLocation>,
}

#[allow(clippy::return_self_not_must_use)]
impl EntityAssertion<'_> {
    /// Asserts the entity exists.
    ///
    /// # Panics
    ///
    /// This will panic if the entity does not exist.
    #[allow(clippy::must_use_candidate)]
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
            "assertion failed: component expected to exist in entity",
        );
        self
    }

    /// Asserts the entity has not a component of type `C`.
    ///
    /// # Panics
    ///
    /// This will panic if the entity does not exist or has a component of type `C`.
    #[allow(clippy::must_use_candidate)]
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
            "assertion failed: component expected to be missing in entity",
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
