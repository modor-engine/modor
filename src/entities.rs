use crate::entities::internal::{
    AddedComponents, ComponentAdd, ComponentInfo, SealedEntityType, StorageWrapper,
};
use crate::storages::archetypes::{ArchetypeIdx, ArchetypeStorage};
use crate::storages::core::CoreStorage;
use crate::storages::entities::EntityIdx;
use crate::SystemRunner;
use std::any::{Any, TypeId};
use std::marker::PhantomData;

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
///     type Type = ();
///     type Data = String;
///
///     fn build(builder: EntityBuilder<'_, Self>, name: Self::Data) -> Built<'_> {
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
    /// The entity type, can be either `()` (standard entity) or [`Singleton`](crate::Singleton).
    type Type: EntityType;

    /// The type of the data used to build the entity.
    type Data: Any + Sync + Send;

    /// Builds the entity.
    fn build(builder: EntityBuilder<'_, Self>, data: Self::Data) -> Built<'_>;

    /// Defines systems to run during update.
    ///
    /// The systems are only run when a component of type `Self` exists in at least one entity.
    fn on_update(runner: SystemRunner<'_>) -> SystemRunner<'_> {
        runner
    }
}

/// A trait implemented for all entity types.
pub trait EntityType: Any + SealedEntityType {}

impl SealedEntityType for () {}

impl EntityType for () {}

/// The singleton entity type.
///
/// When you create a singleton entity, any existing instance is deleted first, except with the
/// [`Built::with_dependency`](crate::Built::with_dependency) method.<br>
/// The instance can be directly accessed in systems using [`Single`](crate::Single) and
/// [`SingleMut`](crate::SingleMut) parameter types.
///
/// It has to be noted that an entity main component defined as singleton can be added to entities
/// as a simple component (e.g. using [`EntityBuilder::with`](crate::EntityBuilder::with)).<br>
/// In this case, the entity will not be tracked as a singleton by the engine, and so the component
/// will not be accessible in systems using [`Single`](crate::Single) and
/// [`SingleMut`](crate::SingleMut).
pub struct Singleton;

impl SealedEntityType for Singleton {}

impl EntityType for Singleton {}

/// A builder for defining the components of an entity.
///
/// # Examples
///
/// See [`EntityMainComponent`](crate::EntityMainComponent).
pub struct EntityBuilder<'a, E, A = ()> {
    core: &'a mut CoreStorage,
    entity_idx: Option<EntityIdx>,
    dst_archetype_idx: ArchetypeIdx,
    parent_idx: Option<EntityIdx>,
    added_components: A,
    phantom: PhantomData<E>,
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
        let (core, entity_idx) = self.build();
        C::build(EntityBuilder::<_, ()>::new(core, Some(entity_idx)), data);
        EntityBuilder::<_, ()>::from_existing(core, entity_idx)
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
        let (core, entity_idx) = self.build();
        let built = P::build(
            EntityBuilder::<_, ()>::from_existing(core, entity_idx),
            data,
        );
        EntityBuilder::<_, ()>::from_existing(built.core, entity_idx)
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
            dst_archetype_idx: archetype_idx,
            parent_idx: self.parent_idx,
            added_components: AddedComponents {
                component: Some(ComponentInfo {
                    component,
                    type_idx,
                    is_singleton: TypeId::of::<C>() == TypeId::of::<E>()
                        && TypeId::of::<E::Type>() == TypeId::of::<Singleton>(),
                }),
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
            EntityBuilder {
                core: self.core,
                entity_idx: self.entity_idx,
                dst_archetype_idx: self.dst_archetype_idx,
                parent_idx: self.parent_idx,
                added_components: AddedComponents {
                    component: None,
                    other_components: self.added_components,
                },
                phantom: PhantomData,
            }
        }
    }

    /// Add the main component of the entity.
    pub fn with_self(self, entity: E) -> Built<'a> {
        let builder = self.with(entity);
        let components = builder.core.components();
        if !components.is_entity_type::<E>() {
            let entity_type_idx = builder.core.add_entity_type::<E>();
            E::on_update(SystemRunner {
                core: builder.core,
                entity_type_idx,
                latest_action_idx: None,
            });
        }
        let (core, _) = builder.build();
        Built { core }
    }

    pub(crate) fn new(
        core: &mut CoreStorage,
        parent_idx: Option<EntityIdx>,
    ) -> EntityBuilder<'_, E, ()> {
        EntityBuilder {
            core,
            dst_archetype_idx: ArchetypeStorage::DEFAULT_IDX,
            entity_idx: None,
            parent_idx,
            added_components: (),
            phantom: PhantomData,
        }
    }

    pub(crate) fn from_existing(
        core: &mut CoreStorage,
        entity_idx: EntityIdx,
    ) -> EntityBuilder<'_, E, ()> {
        EntityBuilder {
            dst_archetype_idx: core
                .entities()
                .location(entity_idx)
                .expect("internal error: missing entity location")
                .idx,
            parent_idx: core.entities().parent_idx(entity_idx),
            core,
            entity_idx: Some(entity_idx),
            added_components: (),
            phantom: PhantomData,
        }
    }

    fn build(self) -> (&'a mut CoreStorage, EntityIdx) {
        let type_idx = self.core.register_component_type::<E>();
        if let Some(location) = self.core.components().singleton_locations(type_idx) {
            let entity_idx = self.core.archetypes().entity_idxs(location.idx)[location.pos];
            self.core.delete_entity(entity_idx);
        }
        let (entity_idx, location) = if let Some(entity_idx) = self.entity_idx {
            let src_location = self
                .core
                .entities()
                .location(entity_idx)
                .expect("internal error: missing entity when adding components");
            let location = self.core.move_entity(src_location, self.dst_archetype_idx);
            (entity_idx, location)
        } else {
            self.core
                .create_entity(self.dst_archetype_idx, self.parent_idx)
        };
        let mut storage = StorageWrapper { core: self.core };
        self.added_components.add(&mut storage, location);
        (self.core, entity_idx)
    }
}

/// A type that indicates the entity has been built.
///
/// This type is also used to perform operations once an entity is created.
///
/// # Examples
///
/// ```rust
/// # use modor::{Built, EntityBuilder, EntityMainComponent, Singleton};
/// #
/// struct PhysicsModule;
///
/// impl EntityMainComponent for PhysicsModule {
///     type Type = Singleton;
///     type Data = ();
///
///     fn build(builder: EntityBuilder<'_, Self>, _: Self::Data) -> Built<'_> {
///         builder.with_self(Self)
///     }
/// }
///
/// struct GraphicsModule;
///
/// impl EntityMainComponent for GraphicsModule {
///     type Type = Singleton;
///     type Data = ();
///
///     fn build(builder: EntityBuilder<'_, Self>, _: Self::Data) -> Built<'_> {
///         builder.with_self(Self).with_dependency::<PhysicsModule>(())
///     }
/// }
/// ```
pub struct Built<'a> {
    core: &'a mut CoreStorage,
}

impl Built<'_> {
    /// Creates a singleton entity with main component of type `E` and building data `data` if
    /// the singleton does not already exist.
    ///
    /// The created entity has no parent.
    pub fn with_dependency<E>(self, data: E::Data) -> Self
    where
        E: EntityMainComponent<Type = Singleton>,
    {
        // Method of `Build` and not of `EntityBuilder` to avoid stack overflow
        let singleton_exists = self
            .core
            .components()
            .type_idx(TypeId::of::<E>())
            .and_then(|c| self.core.components().singleton_locations(c))
            .is_none();
        if singleton_exists {
            E::build(EntityBuilder::<_, ()>::new(self.core, None), data);
        }
        self
    }
}

mod internal {
    use crate::storages::archetypes::EntityLocation;
    use crate::storages::components::ComponentTypeIdx;
    use crate::storages::core::CoreStorage;
    use std::any::Any;

    pub trait SealedEntityType {}

    pub struct StorageWrapper<'a> {
        pub(super) core: &'a mut CoreStorage,
    }

    pub trait ComponentAdd {
        fn add(self, storage: &mut StorageWrapper<'_>, location: EntityLocation);
    }

    pub struct AddedComponents<C, A> {
        pub(super) component: Option<ComponentInfo<C>>,
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
                storage.core.add_component(
                    component.component,
                    component.type_idx,
                    location,
                    component.is_singleton,
                );
            }
        }
    }

    impl ComponentAdd for () {
        fn add(self, _storage: &mut StorageWrapper<'_>, _location: EntityLocation) {}
    }

    pub(super) struct ComponentInfo<C> {
        pub(super) component: C,
        pub(super) type_idx: ComponentTypeIdx,
        pub(super) is_singleton: bool,
    }
}

#[cfg(test)]
mod built_tests {
    use crate::Built;

    assert_impl_all!(Built<'_>:  Send, Unpin);
}

#[cfg(test)]
mod entity_builder_tests {
    use crate::storages::archetypes::{ArchetypeEntityPos, ArchetypeIdx};
    use crate::storages::core::CoreStorage;
    use crate::{EntityBuilder, Singleton};
    use std::any::Any;
    use std::fmt::Debug;

    create_entity_type!(ParentEntity);
    create_entity_type!(TestEntity);
    create_entity_type!(ChildEntity);
    create_entity_type!(Singleton1, Singleton);
    create_entity_type!(Singleton2, Singleton);

    assert_impl_all!(EntityBuilder<'_, ParentEntity>: Send, Unpin);

    #[test]
    fn build_entity() {
        let mut core = CoreStorage::default();
        EntityBuilder::<_, ()>::new(&mut core, None).with_self(Singleton1(70));
        EntityBuilder::<_, ()>::new(&mut core, None)
            .with(10_u32)
            .with_option(Some(0_i64))
            .with_option(Some(20_i64))
            .with_child::<ChildEntity>(60)
            .with_option::<i8>(None)
            .inherit_from::<ParentEntity>(40)
            .with_self(TestEntity(50))
            .with_dependency::<Singleton1>(80)
            .with_dependency::<Singleton2>(90);
        core.register_component_type::<i8>();
        let archetype_idx = ArchetypeIdx::from(6);
        let archetype_pos = ArchetypeEntityPos::from(0);
        assert_component_eq(&core, archetype_idx, archetype_pos, Some(&10_u32));
        assert_component_eq(&core, archetype_idx, archetype_pos, Some(&20_i64));
        assert_component_eq::<i8>(&core, archetype_idx, archetype_pos, None);
        assert_component_eq(&core, archetype_idx, archetype_pos, Some(&ParentEntity(40)));
        assert_component_eq(&core, archetype_idx, archetype_pos, Some(&TestEntity(50)));
        assert_component_eq(&core, 4.into(), 0.into(), Some(&ChildEntity(60)));
        assert!(core.components().is_entity_type::<TestEntity>());
        assert_eq!(core.entities().parent_idx(2.into()), Some(1.into()));
        assert_component_eq(&core, 1.into(), 0.into(), Some(&Singleton1(70)));
        assert_component_eq(&core, 7.into(), 0.into(), Some(&Singleton2(90)));
    }

    #[test]
    fn build_existing_singleton() {
        let mut core = CoreStorage::default();
        EntityBuilder::<_, ()>::new(&mut core, None).with_self(Singleton1(10));
        EntityBuilder::<_, ()>::new(&mut core, None).with_self(Singleton1(20));
        assert_component_eq(&core, 1.into(), 0.into(), Some(&Singleton1(20)));
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
