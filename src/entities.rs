use crate::entities::internal::{AddedComponents, ComponentAdd, StorageWrapper};
use crate::storages::archetypes::{ArchetypeIdx, ArchetypeStorage, EntityLocation};
use crate::storages::core::{CoreStorage, SystemCallerType};
use crate::storages::entities::EntityIdx;
use crate::SystemRunner;
use std::any::{Any, TypeId};
use std::marker::PhantomData;

/// A type that indicates the entity has been built.
pub struct Built(EntityLocation);

/// A trait for defining the main component of an entity type.
///
/// # Examples
///
/// ```rust
/// # use modor::*;
/// #
/// # struct Position(f32, f32);
/// # struct Velocity(f32, f32);
/// #
/// struct Object {
///     name: String,
/// }
///
/// impl EntityMainComponent for Object {
///     type Data = String;
///
///     fn build(builder: EntityBuilder<'_, Self>, name: Self::Data) -> Built {
///         builder
///             .with(Position(0., 0.))
///             .with(Velocity(1., 2.))
///             .with_self(Self { name })
///     }
///
///     fn on_update(runner: SystemRunner<'_>) -> SystemRunner<'_> {
///         runner
///             .run(system!(Self::update_position))
///             .run(system!(Self::update_state))
///     }
/// }
///
/// impl Object {
///     fn update_position(&self, position: &mut Position, velocity: &Velocity) {
///         position.0 += velocity.0;
///         position.1 += velocity.1;
///         println!("New position of '{}': ({}, {})", self.name, position.0, position.1);
///     }
///
///     fn update_state(&self, position: &Position, entity: Entity<'_>, mut world: World<'_>) {
///         if position.0 > 10. {
///             world.delete_entity(entity.id());
///             println!("'{}' has been deleted", self.name);
///         }
///     }
/// }
/// ```
pub trait EntityMainComponent: Sized + Any + Sync + Send {
    /// The type of the data used to build the entity.
    type Data: Any + Sync + Send;

    /// Builds the entity.
    fn build(builder: EntityBuilder<'_, Self>, data: Self::Data) -> Built;

    /// Defines systems to run during update.
    ///
    /// The systems are only run when a component of type `Self` exists in at least one entity.
    fn on_update(runner: SystemRunner<'_>) -> SystemRunner<'_> {
        runner
    }
}

/// A builder for defining the components of an entity.
///
/// # Examples
///
/// See [`EntityMainComponent`](crate::EntityMainComponent).
pub struct EntityBuilder<'a, E, A = ()> {
    pub(crate) core: &'a mut CoreStorage,
    pub(crate) entity_idx: Option<EntityIdx>,
    pub(crate) src_location: Option<EntityLocation>,
    pub(crate) dst_archetype_idx: ArchetypeIdx,
    pub(crate) parent_idx: Option<EntityIdx>,
    pub(crate) added_components: A,
    pub(crate) phantom: PhantomData<E>,
}

impl<'a, E, A> EntityBuilder<'a, E, A>
where
    E: EntityMainComponent,
    A: ComponentAdd,
{
    /// Creates a child entity with main component of type `C` and building data `data`.
    pub fn with_child<C>(self, data: C::Data) -> EntityBuilder<'a, E, ()>
    where
        C: EntityMainComponent,
    {
        let parent_idx = self.parent_idx;
        let (core, entity_idx, location) = self.build();
        C::build(
            EntityBuilder {
                core,
                entity_idx: None,
                src_location: None,
                dst_archetype_idx: ArchetypeStorage::DEFAULT_IDX,
                parent_idx: Some(entity_idx),
                added_components: (),
                phantom: PhantomData,
            },
            data,
        );
        EntityBuilder {
            core,
            entity_idx: Some(entity_idx),
            src_location: Some(location),
            dst_archetype_idx: location.idx,
            parent_idx,
            added_components: (),
            phantom: PhantomData,
        }
    }

    /// Inherits from an entity with main component type `P` and building data `data`.
    ///
    /// Components and systems of the parent entity are added to the built entity.
    ///
    /// If the parent has a component with a type already added to the built entity, the parent's
    /// component overwrites the existing one.<br>
    /// If after calling this method, a component with a type contained in the parent entity is
    /// added to the built entity, the new component overwrites the parent's.
    pub fn inherit_from<P>(self, data: P::Data) -> EntityBuilder<'a, E, ()>
    where
        P: EntityMainComponent,
    {
        let parent_idx = self.parent_idx;
        let (core, entity_idx, location) = self.build();
        let built = P::build(
            EntityBuilder {
                core,
                entity_idx: Some(entity_idx),
                src_location: Some(location),
                dst_archetype_idx: location.idx,
                parent_idx,
                added_components: (),
                phantom: PhantomData,
            },
            data,
        );
        EntityBuilder {
            core,
            entity_idx: Some(entity_idx),
            src_location: Some(built.0),
            dst_archetype_idx: built.0.idx,
            parent_idx,
            added_components: (),
            phantom: PhantomData,
        }
    }

    /// Adds a component of type `C`.
    ///
    /// If a component of type `C` already exists, it is overwritten.
    pub fn with<C>(self, component: C) -> EntityBuilder<'a, E, AddedComponents<C, A>>
    where
        C: Any + Sync + Send,
    {
        let (type_idx, archetype_idx) = self.core.add_component_type::<C>(self.dst_archetype_idx);
        EntityBuilder {
            core: self.core,
            entity_idx: self.entity_idx,
            src_location: self.src_location,
            dst_archetype_idx: archetype_idx,
            parent_idx: self.parent_idx,
            added_components: AddedComponents {
                component: Some(component),
                type_idx,
                other_components: self.added_components,
            },
            phantom: PhantomData,
        }
    }

    /// Adds a component of type `C` only if `component` is not `None`.
    ///
    /// If `component` is not `None` and a component of type `C` already exists, it is overwritten.
    pub fn with_option<C>(self, component: Option<C>) -> EntityBuilder<'a, E, AddedComponents<C, A>>
    where
        C: Any + Sync + Send,
    {
        if let Some(component) = component {
            self.with(component)
        } else {
            let (type_idx, _) = self.core.add_component_type::<C>(self.dst_archetype_idx);
            EntityBuilder {
                core: self.core,
                entity_idx: self.entity_idx,
                src_location: self.src_location,
                dst_archetype_idx: self.dst_archetype_idx,
                parent_idx: self.parent_idx,
                added_components: AddedComponents {
                    component: None,
                    type_idx,
                    other_components: self.added_components,
                },
                phantom: PhantomData,
            }
        }
    }

    /// Add the main component of the entity.
    pub fn with_self(self, entity: E) -> Built {
        let builder = self.with(entity);
        let components = builder.core.components();
        if !components.is_entity_type::<E>() {
            builder.core.add_entity_type::<E>();
            E::on_update(SystemRunner {
                core: builder.core,
                caller_type: SystemCallerType::Entity(TypeId::of::<E>()),
                latest_action_idx: None,
            });
        }
        let (_, _, location) = builder.build();
        Built(location)
    }

    fn build(self) -> (&'a mut CoreStorage, EntityIdx, EntityLocation) {
        let (entity_idx, location) = if let Some(src_location) = self.src_location {
            let entity_idx = self
                .entity_idx
                .expect("internal error: missing index of modified entity");
            let location = self.core.move_entity(src_location, self.dst_archetype_idx);
            (entity_idx, location)
        } else {
            self.core
                .create_entity(self.dst_archetype_idx, self.parent_idx)
        };
        let mut storage = StorageWrapper(self.core);
        self.added_components.add(&mut storage, location);
        (self.core, entity_idx, location)
    }
}

mod internal {
    use crate::storages::archetypes::EntityLocation;
    use crate::storages::components::ComponentTypeIdx;
    use crate::storages::core::CoreStorage;
    use std::any::Any;

    pub struct StorageWrapper<'a>(pub(super) &'a mut CoreStorage);

    pub trait ComponentAdd {
        fn add(self, storage: &mut StorageWrapper<'_>, location: EntityLocation);
    }

    pub struct AddedComponents<C, A> {
        pub(super) component: Option<C>,
        pub(super) type_idx: ComponentTypeIdx,
        pub(super) other_components: A,
    }

    impl<C, A> ComponentAdd for AddedComponents<C, A>
    where
        C: Any + Sync + Send,
        A: ComponentAdd,
    {
        fn add(self, storage: &mut StorageWrapper<'_>, location: EntityLocation) {
            self.other_components.add(storage, location);
            if let Some(component) = self.component {
                storage.0.add_component(component, self.type_idx, location);
            }
        }
    }

    impl ComponentAdd for () {
        fn add(self, _storage: &mut StorageWrapper<'_>, _location: EntityLocation) {}
    }
}

#[cfg(test)]
mod built_tests {
    use crate::Built;
    use std::panic::{RefUnwindSafe, UnwindSafe};

    assert_impl_all!(Built: Sync, Send, Unpin, UnwindSafe, RefUnwindSafe);
}

#[cfg(test)]
mod entity_builder_tests {
    use crate::storages::archetypes::{ArchetypeEntityPos, ArchetypeIdx, ArchetypeStorage};
    use crate::storages::core::CoreStorage;
    use crate::{Built, EntityBuilder, EntityMainComponent};
    use std::any::Any;
    use std::fmt::Debug;
    use std::marker::PhantomData;

    assert_impl_all!(EntityBuilder<'_, ParentEntity>: Send, Unpin);

    #[derive(Debug, PartialEq)]
    struct ParentEntity(u32);

    impl EntityMainComponent for ParentEntity {
        type Data = u32;

        fn build(builder: EntityBuilder<'_, Self>, data: Self::Data) -> Built {
            builder.with_self(Self(data))
        }
    }

    #[derive(Debug, PartialEq)]
    struct TestEntity(u32);

    impl EntityMainComponent for TestEntity {
        type Data = u32;

        fn build(builder: EntityBuilder<'_, Self>, data: Self::Data) -> Built {
            builder.with_self(Self(data))
        }
    }

    #[derive(Debug, PartialEq)]
    struct ChildEntity(u32);

    impl EntityMainComponent for ChildEntity {
        type Data = u32;

        fn build(builder: EntityBuilder<'_, Self>, data: Self::Data) -> Built {
            builder.with_self(Self(data))
        }
    }

    #[test]
    fn build_entity() {
        let mut core = CoreStorage::default();
        let builder = EntityBuilder::<TestEntity> {
            core: &mut core,
            src_location: None,
            dst_archetype_idx: ArchetypeStorage::DEFAULT_IDX,
            entity_idx: None,
            parent_idx: None,
            added_components: (),
            phantom: PhantomData,
        };
        builder
            .with(10_u32)
            .with_option(Some(0_i64))
            .with_option(Some(20_i64))
            .with_child::<ChildEntity>(60)
            .with_option::<i8>(None)
            .inherit_from::<ParentEntity>(40)
            .with_self(TestEntity(50));
        let archetype_idx = ArchetypeIdx::from(6);
        let archetype_pos = ArchetypeEntityPos::from(0);
        assert_component_eq(&core, archetype_idx, archetype_pos, Some(&10_u32));
        assert_component_eq(&core, archetype_idx, archetype_pos, Some(&20_i64));
        assert_component_eq::<i8>(&core, archetype_idx, archetype_pos, None);
        assert_component_eq(&core, archetype_idx, archetype_pos, Some(&ParentEntity(40)));
        assert_component_eq(&core, archetype_idx, archetype_pos, Some(&TestEntity(50)));
        assert_component_eq(&core, 3.into(), 0.into(), Some(&ChildEntity(60)));
        assert!(core.components().is_entity_type::<TestEntity>());
        assert_eq!(core.entities().parent_idx(1.into()), Some(0.into()));
    }

    fn assert_component_eq<C>(
        core: &CoreStorage,
        archetype_idx: ArchetypeIdx,
        archetype_pos: ArchetypeEntityPos,
        expected_component: Option<&C>,
    ) where
        C: Any + PartialEq + Debug,
    {
        assert_eq!(
            core.components()
                .read_components::<C>()
                .get(archetype_idx)
                .and_then(|c| c.get(archetype_pos)),
            expected_component
        );
    }
}
