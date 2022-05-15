use crate::entities::internal::{
    BuildEntity, BuildEntityPart, ChildPart, ChildrenPart, ComponentPart, DependencyPart,
    InheritedPart, MainComponentPart, SealedEntityType,
};
use crate::storages::archetypes::{ArchetypeIdx, ArchetypeStorage, EntityLocation};
use crate::storages::core::CoreStorage;
use crate::storages::entities::EntityIdx;
use crate::SystemRunner;
use std::any::Any;
use std::marker::PhantomData;

/// A trait for defining the main component of an entity type.
///
/// This trait shouldn't be directly implemented.<br>
/// Instead, you can use [`entity`](macro@crate::entity) and [`singleton`](macro@crate::singleton)
/// proc macros.
pub trait EntityMainComponent: Sized + Any + Sync + Send {
    #[doc(hidden)]
    type Type: EntityType;

    #[doc(hidden)]
    fn on_update(runner: SystemRunner<'_>) -> SystemRunner<'_>;
}

#[doc(hidden)]
pub trait EntityType: Any + SealedEntityType {}

#[doc(hidden)]
pub struct NotSingleton;

impl SealedEntityType for NotSingleton {}

impl EntityType for NotSingleton {}

#[doc(hidden)]
pub struct Singleton;

impl SealedEntityType for Singleton {}

impl EntityType for Singleton {}

/// A trait implemented for all types able to build an entity.
///
/// This trait is particularly useful when defining a building method for an entity.
///
/// # Examples
///
/// See [`EntityBuilder`](crate::EntityBuilder).
pub trait Built<E>: BuildEntity
where
    E: EntityMainComponent,
{
}

/// A builder for defining the components and children of an entity.
///
/// # Examples
///
///
/// ```rust
/// # use modor::*;
/// #
/// # struct Position(f32, f32);
/// # struct Velocity(f32, f32);
/// # struct Acceleration(f32, f32);
/// #
/// struct Object {
///     name: String,
/// }
///
/// #[entity]
/// impl Object {
///     fn build(name: String, is_accelerating: bool) -> impl Built<Self> {
///         EntityBuilder::new(Self{name})
///             .with(Position(0., 0.))
///             .with(Velocity(1., 2.))
///             .with_option(is_accelerating.then(|| Acceleration(0.01, 0.08)))
///     }
/// }
/// ```
pub struct EntityBuilder<E, P, O> {
    part: P,
    other_parts: O,
    phantom: PhantomData<E>,
}

impl<E> EntityBuilder<E, MainComponentPart<E>, ()>
where
    E: EntityMainComponent,
{
    /// Starts building an entity by providing its `main_component`.
    pub fn new(main_component: E) -> Self {
        Self {
            part: MainComponentPart {
                component_part: ComponentPart {
                    component: Some(main_component),
                    type_idx: None,
                    phantom: PhantomData,
                },
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
    pub fn with<C>(self, component: C) -> EntityBuilder<E, ComponentPart<E, C>, Self>
    where
        C: Any + Sync + Send,
    {
        EntityBuilder {
            part: ComponentPart {
                component: Some(component),
                type_idx: None,
                phantom: PhantomData,
            },
            other_parts: self,
            phantom: PhantomData,
        }
    }

    /// Adds a component of type `C` only if `component` is not `None`.
    ///
    /// If `component` is not `None` and a component of type `C` already exists, it is overwritten.
    pub fn with_option<C>(self, component: Option<C>) -> EntityBuilder<E, ComponentPart<E, C>, Self>
    where
        C: Any + Sync + Send,
    {
        EntityBuilder {
            part: ComponentPart {
                component,
                type_idx: None,
                phantom: PhantomData,
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
/// # use modor::{Built, EntityBuilder, entity};
/// #
/// struct Level1;
///
/// #[entity]
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
/// struct Level2(u32);
///
/// #[entity]
/// impl Level2 {
///     fn build(id: u32) -> impl Built<Self> {
///         EntityBuilder::new(Self(id))
///     }
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
    use crate::{ChildBuilder, EntityMainComponent, Singleton, SystemRunner};
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

    pub struct MainComponentPart<E> {
        pub(super) component_part: ComponentPart<E, E>,
    }

    impl<E> BuildEntityPart for MainComponentPart<E>
    where
        E: EntityMainComponent,
    {
        fn create_archetype(
            &mut self,
            core: &mut CoreStorage,
            archetype_idx: ArchetypeIdx,
        ) -> ArchetypeIdx {
            if core.components().is_entity_type::<E>() {
                let type_idx = core
                    .components()
                    .type_idx(TypeId::of::<E>())
                    .expect("internal error: entity type without index");
                if let Some(location) = core.components().singleton_location(type_idx) {
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
            self.component_part.create_archetype(core, archetype_idx)
        }

        fn add_components(&mut self, core: &mut CoreStorage, location: EntityLocation) {
            self.component_part.add_components(core, location);
        }
    }

    pub struct ComponentPart<E, C> {
        pub(super) component: Option<C>,
        pub(super) type_idx: Option<ComponentTypeIdx>,
        pub(super) phantom: PhantomData<E>,
    }

    impl<E, C> BuildEntityPart for ComponentPart<E, C>
    where
        E: EntityMainComponent,
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
                core.add_component(
                    component,
                    type_idx,
                    location,
                    TypeId::of::<C>() == TypeId::of::<E>()
                        && TypeId::of::<E::Type>() == TypeId::of::<Singleton>(),
                );
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
                .and_then(|c| core.components().singleton_location(c))
                .is_some();
            if !singleton_exists {
                self.entity.build(core, None);
            }
        }
    }

    pub trait BuildEntity: BuildEntityPart {
        fn build(self, core: &mut CoreStorage, parent_idx: Option<EntityIdx>) -> EntityIdx;
    }
}
