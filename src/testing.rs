//! Testing utilities.

use crate::storages::archetypes::{ArchetypeStorage, EntityLocationInArchetype};
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
        let location = self.0 .0.create_entity(ArchetypeStorage::DEFAULT_IDX);
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
        if self.location().is_none() {
            panic!(
                "assertion failed: `assert_entity({}).exists()`",
                usize::from(self.entity_idx)
            );
        }
        self
    }

    /// Asserts the entity does not exist.
    ///
    /// # Panics
    ///
    /// This will panic if the entity exists.
    pub fn does_not_exist(self) {
        if self.location().is_some() {
            panic!(
                "assertion failed: `assert_entity({}).does_not_exist()`",
                usize::from(self.entity_idx)
            );
        }
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
                "assertion failed: `assert_entity({}).has<{}, _>(...)` (entity does not exist)",
                usize::from(self.entity_idx),
                any::type_name::<C>()
            )
        });
        if self.test_component_exists::<C, F>(location, f).is_none() {
            panic!(
                "assertion failed: `assert_entity({}).has<{}, _>(...)` (missing component in entity)",
                usize::from(self.entity_idx),
                any::type_name::<C>()
            )
        }
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
                "assertion failed: `assert_entity({}).has_not<{}>()` (entity does not exist)",
                usize::from(self.entity_idx),
                any::type_name::<C>()
            )
        });
        if self
            .test_component_exists::<C, _>(location, |_| ())
            .is_some()
        {
            panic!(
                "assertion failed: `assert_entity({}).has_not<{}>()` (existing component in entity)",
                usize::from(self.entity_idx),
                any::type_name::<C>()
            )
        }
        self
    }

    fn location(&self) -> Option<EntityLocationInArchetype> {
        self.core.entities().location(self.entity_idx.into())
    }

    fn test_component_exists<C, F>(
        &self,
        location: EntityLocationInArchetype,
        mut f: F,
    ) -> Option<()>
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
    use super::*;
    use crate::Built;
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
    fn assert_entity() {
        let app = TestApp::new();

        let assertion = app.assert_entity(1);

        assert_eq!(assertion.entity_idx, 1.into());
    }

    #[test]
    fn create_entity() {
        let mut app = TestApp::new();

        let entity_idx = app.create_entity::<TestEntity>("string".into());

        assert_eq!(entity_idx, 0);
        let components = app.0 .0.components().read_components::<TestEntity>();
        let expected_components = ti_vec![ti_vec![], ti_vec![TestEntity("string".into())]];
        assert_eq!(&*components, &expected_components)
    }

    #[test]
    fn create_from_existing_app() {
        let existing_app = App::new().with_entity::<TestEntity>("string".into());

        let app = TestApp::from(existing_app);

        let components = app.0 .0.components().read_components::<TestEntity>();
        let expected_components = ti_vec![ti_vec![], ti_vec![TestEntity("string".into())]];
        assert_eq!(&*components, &expected_components)
    }

    #[test]
    fn deref() {
        let app = TestApp::new();

        let app_ref = &*app;

        assert!(ptr::eq(app_ref, &app.0))
    }

    #[test]
    fn deref_mut() {
        let mut app = TestApp::new();

        let app_ref = &mut *app as *const App;

        assert!(ptr::eq(app_ref, &app.0))
    }
}

#[cfg(test)]
mod entity_assertion_tests {
    use super::*;

    #[test]
    #[should_panic(expected = "assertion failed: `assert_entity(0).exists()")]
    fn assert_entity_exists_when_missing() {
        let core = CoreStorage::default();
        let assertion = EntityAssertion {
            core: &core,
            entity_idx: 0.into(),
        };

        assertion.exists();
    }

    #[test]
    fn assert_entity_exists_when_existing() {
        let mut core = CoreStorage::default();
        core.create_entity(ArchetypeStorage::DEFAULT_IDX);
        let assertion = EntityAssertion {
            core: &core,
            entity_idx: 0.into(),
        };

        let new_assertion = assertion.exists();

        assert_eq!(new_assertion.entity_idx, 0.into());
    }
    #[test]
    #[should_panic(expected = "assertion failed: `assert_entity(0).does_not_exist()")]
    fn assert_entity_does_not_exist_when_existing() {
        let mut core = CoreStorage::default();
        core.create_entity(ArchetypeStorage::DEFAULT_IDX);
        let assertion = EntityAssertion {
            core: &core,
            entity_idx: 0.into(),
        };

        assertion.does_not_exist();
    }

    #[test]
    fn assert_entity_does_not_exist_when_missing() {
        let core = CoreStorage::default();
        let assertion = EntityAssertion {
            core: &core,
            entity_idx: 0.into(),
        };

        assertion.does_not_exist();
    }

    #[test]
    #[should_panic(expected = "assertion failed: `assert_entity(0).has<u32, _>(...)` \
    (entity does not exist)")]
    fn assert_entity_has_component_when_entity_missing() {
        let core = CoreStorage::default();
        let assertion = EntityAssertion {
            core: &core,
            entity_idx: 0.into(),
        };

        assertion.has::<u32, _>(|a| assert_eq!(a, &10));
    }

    #[test]
    #[should_panic(expected = "assertion failed: `assert_entity(0).has<u32, _>(...)` \
    (missing component in entity)")]
    fn assert_entity_has_component_when_component_missing() {
        let mut core = CoreStorage::default();
        core.create_entity(ArchetypeStorage::DEFAULT_IDX);
        let assertion = EntityAssertion {
            core: &core,
            entity_idx: 0.into(),
        };

        assertion.has::<u32, _>(|a| assert_eq!(a, &10));
    }

    #[test]
    #[should_panic]
    fn assert_entity_has_component_when_component_existing_and_false_assertion() {
        let mut core = CoreStorage::default();
        let archetype1_idx = ArchetypeStorage::DEFAULT_IDX;
        let (type_idx, archetype2_idx) = core.add_component_type::<u32>(archetype1_idx);
        let location = core.create_entity(archetype2_idx);
        core.add_component(10_u32, type_idx, location);
        let assertion = EntityAssertion {
            core: &core,
            entity_idx: 0.into(),
        };

        assertion.has::<u32, _>(|a| assert_eq!(a, &20));
    }

    #[test]
    fn assert_entity_has_component_when_component_existing_and_true_assertion() {
        let mut core = CoreStorage::default();
        let archetype1_idx = ArchetypeStorage::DEFAULT_IDX;
        let (type_idx, archetype2_idx) = core.add_component_type::<u32>(archetype1_idx);
        let location = core.create_entity(archetype2_idx);
        core.add_component(10_u32, type_idx, location);
        let assertion = EntityAssertion {
            core: &core,
            entity_idx: 0.into(),
        };

        let new_assertion = assertion.has::<u32, _>(|a| assert_eq!(a, &10));

        assert_eq!(new_assertion.entity_idx, 0.into());
    }

    #[test]
    #[should_panic(expected = "assertion failed: `assert_entity(0).has_not<u32>()` \
    (entity does not exist)")]
    fn assert_entity_has_not_component_when_entity_missing() {
        let core = CoreStorage::default();
        let assertion = EntityAssertion {
            core: &core,
            entity_idx: 0.into(),
        };

        assertion.has_not::<u32>();
    }

    #[test]
    #[should_panic(expected = "assertion failed: `assert_entity(0).has_not<u32>()` \
    (existing component in entity)")]
    fn assert_entity_has_not_component_when_component_existing() {
        let mut core = CoreStorage::default();
        let archetype1_idx = ArchetypeStorage::DEFAULT_IDX;
        let (type_idx, archetype2_idx) = core.add_component_type::<u32>(archetype1_idx);
        let location = core.create_entity(archetype2_idx);
        core.add_component(10_u32, type_idx, location);
        let assertion = EntityAssertion {
            core: &core,
            entity_idx: 0.into(),
        };

        assertion.has_not::<u32>();
    }

    #[test]
    fn assert_entity_has_not_component_when_component_missing() {
        let mut core = CoreStorage::default();
        core.create_entity(ArchetypeStorage::DEFAULT_IDX);
        let assertion = EntityAssertion {
            core: &core,
            entity_idx: 0.into(),
        };

        let new_assertion = assertion.has_not::<u32>();

        assert_eq!(new_assertion.entity_idx, 0.into());
    }
}
