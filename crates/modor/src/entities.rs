use crate::entities::internal::{
    BuildEntity, BuildEntityPart, ChildPart, ChildrenPart, ComponentPart, DependencyPart,
    InheritedPart, SealedEntityType,
};
use crate::storages::archetypes::{ArchetypeIdx, ArchetypeStorage, EntityLocation};
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
/// impl Object {
///     fn build(name: String) -> impl Built<Self> {
///         EntityBuilder::new(Self{name})
///             .with(Position(0., 0.))
///             .with(Velocity(1., 2.))
///     }
///
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
///
/// impl EntityMainComponent for Object {
///     type Type = ();
///
///     fn on_update(runner: SystemRunner<'_>) -> SystemRunner<'_> {
///         runner
///             .run(system!(Self::update_position))
///             .run(system!(Self::update_state))
///     }
/// }
/// ```
pub trait EntityMainComponent: Sized + Any + Sync + Send {
    /// The entity type, can be either `()` (standard entity) or [`Singleton`](crate::Singleton).
    type Type: EntityType;

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
/// [`EntityBuilder::with_dependency`](crate::EntityBuilder::with_dependency) method.<br>
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

/// A trait implemented for all types able to build an entity.
///
/// This trait is particularly useful when defining a building method for an entity.
///
/// # Examples
///
/// See [`EntityMainComponent`](crate::EntityMainComponent).
pub trait Built<E>: BuildEntity
where
    E: EntityMainComponent,
{
}

/// A builder for defining the components and children of an entity.
///
/// # Examples
///
/// See [`EntityMainComponent`](crate::EntityMainComponent).
pub struct EntityBuilder<E, P, O> {
    part: P,
    other_parts: O,
    phantom: PhantomData<E>,
}

impl<E> EntityBuilder<E, ComponentPart<E>, ()>
where
    E: EntityMainComponent,
{
    /// Starts building an entity by providing its `main_component`.
    pub fn new(main_component: E) -> Self {
        Self {
            part: ComponentPart {
                component: Some(main_component),
                type_idx: None,
                is_singleton: TypeId::of::<E::Type>() == TypeId::of::<Singleton>(),
            },
            other_parts: (),
            phantom: PhantomData,
        }
    }
}

impl<E, P, O> EntityBuilder<E, P, O> {
    /// Adds a component of type `C`.
    ///
    /// If a component of type `C` already exists, it is overwritten.
    pub fn with<C>(self, component: C) -> EntityBuilder<E, ComponentPart<C>, Self>
    where
        C: Any + Sync + Send,
    {
        EntityBuilder {
            part: ComponentPart {
                component: Some(component),
                type_idx: None,
                is_singleton: false,
            },
            other_parts: self,
            phantom: PhantomData,
        }
    }

    /// Adds a component of type `C` only if `component` is not `None`.
    ///
    /// If `component` is not `None` and a component of type `C` already exists, it is overwritten.
    pub fn with_option<C>(self, component: Option<C>) -> EntityBuilder<E, ComponentPart<C>, Self>
    where
        C: Any + Sync + Send,
    {
        EntityBuilder {
            part: ComponentPart {
                component,
                type_idx: None,
                is_singleton: false,
            },
            other_parts: self,
            phantom: PhantomData,
        }
    }

    /// Inherits from an entity with main component type `I`.
    ///
    /// Components, children and systems of the parent entity are added to the built entity.
    ///
    /// If the parent has a component with a type already added to the built entity, the parent's
    /// component overwrites the existing one.<br>
    /// If after calling this method, a component with a type contained in the parent entity is
    /// added to the built entity, the new component overwrites the parent's.
    pub fn inherit_from<I>(
        self,
        inherited: impl Built<I>,
    ) -> EntityBuilder<E, InheritedPart<impl Built<I>>, Self>
    where
        I: EntityMainComponent,
    {
        EntityBuilder {
            part: InheritedPart { entity: inherited },
            other_parts: self,
            phantom: PhantomData,
        }
    }

    /// Creates a child entity with main component of type `C`.
    pub fn with_child<C>(
        self,
        child: impl Built<C>,
    ) -> EntityBuilder<E, ChildPart<impl Built<C>>, Self>
    where
        C: EntityMainComponent,
    {
        EntityBuilder {
            part: ChildPart { entity: child },
            other_parts: self,
            phantom: PhantomData,
        }
    }

    /// Creates child entities.
    ///
    /// This method can be used instead of
    /// [`EntityBuilder::with_child`](crate::EntityBuilder::with_child) when children are
    /// created dynamically (e.g. with conditional creation or loops).
    pub fn with_children<F>(self, build_fn: F) -> EntityBuilder<E, ChildrenPart<F>, Self>
    where
        F: FnOnce(&mut ChildBuilder<'_>),
    {
        EntityBuilder {
            part: ChildrenPart { build_fn },
            other_parts: self,
            phantom: PhantomData,
        }
    }

    /// Creates a singleton entity with main component of type `D` if the singleton does
    /// not already exist.
    ///
    /// The created entity has no parent.
    pub fn with_dependency<D>(
        self,
        dependency: impl Built<D>,
    ) -> EntityBuilder<E, DependencyPart<D, impl Built<D>>, Self>
    where
        D: EntityMainComponent<Type = Singleton>,
    {
        EntityBuilder {
            part: DependencyPart {
                entity: dependency,
                phantom: PhantomData,
            },
            other_parts: self,
            phantom: PhantomData,
        }
    }
}

impl<E, P, O> BuildEntityPart for EntityBuilder<E, P, O>
where
    E: EntityMainComponent,
    P: BuildEntityPart,
    O: BuildEntityPart,
{
    fn create_archetype(
        &mut self,
        core: &mut CoreStorage,
        archetype_idx: ArchetypeIdx,
    ) -> ArchetypeIdx {
        let archetype_idx = self.other_parts.create_archetype(core, archetype_idx);
        self.part.create_archetype(core, archetype_idx)
    }

    fn add_components(&mut self, core: &mut CoreStorage, location: EntityLocation) {
        self.other_parts.add_components(core, location);
        self.part.add_components(core, location);
    }

    fn create_other_entities(self, core: &mut CoreStorage, parent_idx: Option<EntityIdx>) {
        self.other_parts.create_other_entities(core, parent_idx);
        self.part.create_other_entities(core, parent_idx);
    }
}

impl<E, P, O> BuildEntity for EntityBuilder<E, P, O>
where
    E: EntityMainComponent,
    P: BuildEntityPart,
    O: BuildEntityPart,
{
    fn build(mut self, core: &mut CoreStorage, parent_idx: Option<EntityIdx>) -> EntityIdx {
        if core.components().is_entity_type::<E>() {
            let type_idx = core
                .components()
                .type_idx(TypeId::of::<E>())
                .expect("internal error: entity type without index");
            if let Some(location) = core.components().singleton_locations(type_idx) {
                let entity_idx = core.archetypes().entity_idxs(location.idx)[location.pos];
                core.delete_entity(entity_idx);
            }
        } else {
            let entity_type_idx = core.add_entity_type::<E>();
            E::on_update(SystemRunner {
                core,
                entity_type_idx,
                latest_action_idx: None,
            });
        };
        let archetype_idx = self.create_archetype(core, ArchetypeStorage::DEFAULT_IDX);
        let (entity_idx, location) = core.create_entity(archetype_idx, parent_idx);
        self.add_components(core, location);
        self.create_other_entities(core, Some(entity_idx));
        entity_idx
    }
}

impl<E, P, O> Built<E> for EntityBuilder<E, P, O>
where
    E: EntityMainComponent,
    P: BuildEntityPart,
    O: BuildEntityPart,
{
}

/// A builder for dynamically defining children of an entity.
///
/// # Examples
///
/// ```rust
/// # use modor::{Built, EntityBuilder, EntityMainComponent};
/// #
/// struct Level1;
///
/// impl Level1 {
///     fn build(child_count: u32) -> impl Built<Self> {
///         EntityBuilder::new(Self)
///             .with_children(move |b| {
///                 for id in 0..child_count {
///                     b.add(Level2::build(id));
///                 }
///             })
///     }
/// }
///
/// impl EntityMainComponent for Level1 {
///     type Type = ();
/// }
///
/// struct Level2(u32);
///
/// impl Level2 {
///     fn build(id: u32) -> impl Built<Self> {
///         EntityBuilder::new(Self(id))
///     }
/// }
///
/// impl EntityMainComponent for Level2 {
///     type Type = ();
/// }
/// ```
pub struct ChildBuilder<'a> {
    core: &'a mut CoreStorage,
    parent_idx: Option<EntityIdx>,
}

impl ChildBuilder<'_> {
    /// Adds a child entity.
    pub fn add<C>(&mut self, child: impl Built<C>)
    where
        C: EntityMainComponent,
    {
        child.build(self.core, self.parent_idx);
    }
}

mod internal {
    use crate::storages::archetypes::{ArchetypeIdx, EntityLocation};
    use crate::storages::components::ComponentTypeIdx;
    use crate::storages::core::CoreStorage;
    use crate::storages::entities::EntityIdx;
    use crate::{ChildBuilder, EntityMainComponent};
    use std::any::{Any, TypeId};
    use std::marker::PhantomData;

    pub trait SealedEntityType {}

    #[allow(unused_variables)]
    pub trait BuildEntityPart: Sized + Any + Sync + Send {
        fn create_archetype(
            &mut self,
            core: &mut CoreStorage,
            archetype_idx: ArchetypeIdx,
        ) -> ArchetypeIdx {
            archetype_idx
        }

        fn add_components(&mut self, core: &mut CoreStorage, location: EntityLocation) {}

        fn create_other_entities(self, core: &mut CoreStorage, parent_idx: Option<EntityIdx>) {}
    }

    impl BuildEntityPart for () {}

    pub struct ComponentPart<C> {
        pub(super) component: Option<C>,
        pub(super) type_idx: Option<ComponentTypeIdx>,
        pub(super) is_singleton: bool,
    }

    impl<C> BuildEntityPart for ComponentPart<C>
    where
        C: Any + Sync + Send,
    {
        fn create_archetype(
            &mut self,
            core: &mut CoreStorage,
            archetype_idx: ArchetypeIdx,
        ) -> ArchetypeIdx {
            if self.component.is_some() {
                let (new_type_idx, archetype_idx) = core.add_component_type::<C>(archetype_idx);
                self.type_idx.replace(new_type_idx);
                archetype_idx
            } else {
                archetype_idx
            }
        }

        fn add_components(&mut self, core: &mut CoreStorage, location: EntityLocation) {
            if let (Some(component), Some(type_idx)) = (self.component.take(), self.type_idx) {
                core.add_component(component, type_idx, location, self.is_singleton);
            }
        }
    }

    pub struct InheritedPart<E> {
        pub(super) entity: E,
    }

    impl<E> BuildEntityPart for InheritedPart<E>
    where
        E: BuildEntityPart,
    {
        fn create_archetype(
            &mut self,
            core: &mut CoreStorage,
            archetype_idx: ArchetypeIdx,
        ) -> ArchetypeIdx {
            self.entity.create_archetype(core, archetype_idx)
        }

        fn add_components(&mut self, core: &mut CoreStorage, location: EntityLocation) {
            self.entity.add_components(core, location);
        }

        fn create_other_entities(self, core: &mut CoreStorage, parent_idx: Option<EntityIdx>) {
            self.entity.create_other_entities(core, parent_idx);
        }
    }

    pub struct ChildPart<E> {
        pub(super) entity: E,
    }

    impl<E> BuildEntityPart for ChildPart<E>
    where
        E: BuildEntity,
    {
        fn create_other_entities(self, core: &mut CoreStorage, parent_idx: Option<EntityIdx>) {
            self.entity.build(core, parent_idx);
        }
    }

    pub struct ChildrenPart<F> {
        pub(super) build_fn: F,
    }

    impl<F> BuildEntityPart for ChildrenPart<F>
    where
        F: FnOnce(&mut ChildBuilder<'_>) + Sync + Send + 'static,
    {
        fn create_other_entities(self, core: &mut CoreStorage, parent_idx: Option<EntityIdx>) {
            let mut builder = ChildBuilder { core, parent_idx };
            (self.build_fn)(&mut builder);
        }
    }

    pub struct DependencyPart<E, B> {
        pub(super) entity: B,
        pub(super) phantom: PhantomData<E>,
    }

    impl<E, B> BuildEntityPart for DependencyPart<E, B>
    where
        E: EntityMainComponent,
        B: BuildEntity,
    {
        fn create_other_entities(self, core: &mut CoreStorage, _parent_idx: Option<EntityIdx>) {
            let singleton_exists = core
                .components()
                .type_idx(TypeId::of::<E>())
                .and_then(|c| core.components().singleton_locations(c))
                .is_some();
            if !singleton_exists {
                self.entity.build(core, None);
            }
        }
    }

    pub trait BuildEntity: BuildEntityPart {
        fn build(self, core: &mut CoreStorage, parent: Option<EntityIdx>) -> EntityIdx;
    }
}

#[cfg(test)]
mod entity_builder_tests {
    use crate::entities::internal::BuildEntity;
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

    #[test]
    fn build_entity() {
        let mut core = CoreStorage::default();
        EntityBuilder::new(Singleton1(10)).build(&mut core, None);
        EntityBuilder::new(TestEntity(20))
            .with(30_u32)
            .with_option(Some(0_i64))
            .with_dependency(EntityBuilder::new(Singleton1(40)))
            .inherit_from(
                EntityBuilder::new(ParentEntity(50))
                    .with("A".to_string())
                    .with_child(EntityBuilder::new(ChildEntity(140))),
            )
            .with_option(Some(60_i64))
            .with_child(
                EntityBuilder::new(ChildEntity(70))
                    .with("B".to_string())
                    .with_child(EntityBuilder::new(ChildEntity(90))),
            )
            .with_children(|b| {
                b.add(EntityBuilder::new(ChildEntity(110)));
                b.add(EntityBuilder::new(ChildEntity(120)));
            })
            .with_children(|b| b.add(EntityBuilder::new(ChildEntity(130))))
            .with_option::<i8>(None)
            .with_dependency(
                EntityBuilder::new(Singleton2(80))
                    .with("C".to_string())
                    .with_child(EntityBuilder::new(ChildEntity(100))),
            )
            .build(&mut core, None);
        core.register_component_type::<i8>();
        let location = core.entities().location(0.into()).unwrap();
        assert_component_eq(&core, location.idx, location.pos, Some(&Singleton1(10)));
        let location = core.entities().location(1.into()).unwrap();
        assert_component_eq(&core, location.idx, location.pos, Some(&TestEntity(20)));
        assert_component_eq(&core, location.idx, location.pos, Some(&30_u32));
        assert_component_eq(&core, location.idx, location.pos, Some(&ParentEntity(50)));
        assert_component_eq(&core, location.idx, location.pos, Some(&60_i64));
        assert_component_eq::<i8>(&core, location.idx, location.pos, None);
        assert_component_eq(&core, location.idx, location.pos, Some(&"A".to_string()));
        let location = core.entities().location(2.into()).unwrap();
        assert_component_eq(&core, location.idx, location.pos, Some(&ChildEntity(140)));
        let location = core.entities().location(3.into()).unwrap();
        assert_component_eq(&core, location.idx, location.pos, Some(&ChildEntity(70)));
        assert_component_eq(&core, location.idx, location.pos, Some(&"B".to_string()));
        let location = core.entities().location(4.into()).unwrap();
        assert_component_eq(&core, location.idx, location.pos, Some(&ChildEntity(90)));
        let location = core.entities().location(5.into()).unwrap();
        assert_component_eq(&core, location.idx, location.pos, Some(&ChildEntity(110)));
        let location = core.entities().location(6.into()).unwrap();
        assert_component_eq(&core, location.idx, location.pos, Some(&ChildEntity(120)));
        let location = core.entities().location(7.into()).unwrap();
        assert_component_eq(&core, location.idx, location.pos, Some(&ChildEntity(130)));
        let location = core.entities().location(8.into()).unwrap();
        assert_component_eq(&core, location.idx, location.pos, Some(&Singleton2(80)));
        assert_component_eq(&core, location.idx, location.pos, Some(&"C".to_string()));
        let location = core.entities().location(9.into()).unwrap();
        assert_component_eq(&core, location.idx, location.pos, Some(&ChildEntity(100)));
        assert_eq!(
            core.entities().child_idxs(1.into()),
            [2.into(), 3.into(), 5.into(), 6.into(), 7.into()]
        );
        assert_eq!(core.entities().child_idxs(3.into()), [4.into()]);
        assert_eq!(core.entities().child_idxs(8.into()), [9.into()]);
    }

    #[test]
    fn build_existing_singleton() {
        let mut core = CoreStorage::default();
        EntityBuilder::new(Singleton1(10)).build(&mut core, None);
        EntityBuilder::new(Singleton1(20)).build(&mut core, None);
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
