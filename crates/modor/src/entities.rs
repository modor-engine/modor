use crate::entities::internal::{
    BuiltEntityPart, ChildPart, ChildrenPart, ComponentPart, DependencyPart, InheritedPart,
};
use crate::storages::archetypes::{ArchetypeIdx, EntityLocation};
use crate::storages::core::CoreStorage;
use crate::storages::entities::EntityIdx;
use crate::{Component, ComponentSystems, True};
use std::any::Any;
use std::marker::PhantomData;

/// A trait implemented for an entity builder.
pub trait BuiltEntity: BuildableEntity + Any + Sync + Send {
    #[doc(hidden)]
    type Parts: BuiltEntityPart;

    /// Adds a component of type `C`.
    ///
    /// If a component of type `C` already exists, it is overwritten.
    fn component<C>(self, component: C) -> EntityBuilder<(Self::Parts, ComponentPart<C>)>
    where
        C: Component + ComponentSystems;

    /// Adds a component of type `C` only if `component` is not `None`.
    ///
    /// If `component` is not `None` and a component of type `C` already exists, it is overwritten.
    fn component_option<C>(
        self,
        component: Option<C>,
    ) -> EntityBuilder<(Self::Parts, ComponentPart<C>)>
    where
        C: Component + ComponentSystems;

    /// Inherits from another built entity.
    ///
    /// Components, children and systems of the parent entity are added to the built entity.
    ///
    /// If the parent has a component with a type already added to the built entity, the parent's
    /// component overwrites the existing one.<br>
    /// If after calling this method, a component with a type contained in the parent entity is
    /// added to the built entity, the new component overwrites the parent's.
    fn inherited<E>(self, entity: E) -> EntityBuilder<(Self::Parts, InheritedPart<E>)>
    where
        E: BuildableEntity;

    /// Creates a child entity.
    fn child_entity<E>(self, child: E) -> EntityBuilder<(Self::Parts, ChildPart<E>)>
    where
        E: BuildableEntity;

    /// Creates child entities.
    ///
    /// This method can be used instead of
    /// [`EntityBuilder::with_child`](EntityBuilder::child_entity) when children are
    /// created dynamically (e.g. with conditional creation or loops).
    fn children<F>(self, builder: F) -> EntityBuilder<(Self::Parts, ChildrenPart<F>)>
    where
        F: FnOnce(&mut EntityChildBuilder<'_>) + Any + Sync + Send;

    /// Creates an entity if the singleton of type `C` does not already exist.
    ///
    /// The created entity has no parent.
    fn dependency<C, E, F>(self, f: F) -> EntityBuilder<(Self::Parts, DependencyPart<C, E, F>)>
    where
        C: Component<IsSingleton = True>,
        E: BuildableEntity,
        F: FnOnce() -> E + Any + Sync + Send;
}

/// A trait implemented for a type that can be used to create an entity.
pub trait BuildableEntity: BuiltEntityPart {}

impl<T> BuildableEntity for T where T: Component + ComponentSystems {}

impl<T> BuiltEntityPart for T
where
    T: Component + ComponentSystems,
{
    fn build(self, core: &mut CoreStorage, parent_idx: Option<EntityIdx>) -> EntityIdx {
        ComponentPart {
            component: Some(self),
            type_idx: None,
        }
        .build(core, parent_idx)
    }
}

/// A builder for defining the components and children of an entity.
///
/// # Examples
///
/// ```rust
/// # use modor::*;
/// #
/// #[derive(Component, NoSystem)]
/// struct Position(f32, f32);
///
/// #[derive(Component, NoSystem)]
/// struct Velocity(f32, f32);
///
/// #[derive(Component, NoSystem)]
/// struct Acceleration(f32, f32);
///
/// fn movable_entity(is_accelerating: bool) -> impl BuiltEntity {
///     EntityBuilder::new()
///         .component(Position(0., 0.))
///         .component(Velocity(1., 2.))
///         .component_option(is_accelerating.then(|| Acceleration(0.01, 0.08)))
/// }
///
/// App::new()
///     .with_entity(movable_entity(true))
///     .with_entity(movable_entity(false));
/// ```
///
/// If the entity only contains one component, you can also create the entity without using
/// [`EntityBuilder`]:
///
/// ```rust
/// # use modor::*;
/// #
/// #[derive(SingletonComponent, NoSystem)]
/// struct Score(u32);
///
/// App::new().with_entity(Score(0));
/// ```
pub struct EntityBuilder<P> {
    parts: P,
}

impl EntityBuilder<()> {
    /// Creates a new entity builder.
    pub const fn new() -> Self {
        Self { parts: () }
    }
}

impl Default for EntityBuilder<()> {
    fn default() -> Self {
        Self::new()
    }
}

impl<P> BuiltEntity for EntityBuilder<P>
where
    P: BuiltEntityPart,
{
    type Parts = P;

    fn component<C>(self, component: C) -> EntityBuilder<(Self::Parts, ComponentPart<C>)>
    where
        C: Component + ComponentSystems,
    {
        EntityBuilder {
            parts: (
                self.parts,
                ComponentPart {
                    component: Some(component),
                    type_idx: None,
                },
            ),
        }
    }

    fn component_option<C>(
        self,
        component: Option<C>,
    ) -> EntityBuilder<(Self::Parts, ComponentPart<C>)>
    where
        C: Component + ComponentSystems,
    {
        EntityBuilder {
            parts: (
                self.parts,
                ComponentPart {
                    component,
                    type_idx: None,
                },
            ),
        }
    }

    fn inherited<E>(self, entity: E) -> EntityBuilder<(Self::Parts, InheritedPart<E>)>
    where
        E: BuildableEntity,
    {
        EntityBuilder {
            parts: (self.parts, InheritedPart { entity }),
        }
    }

    fn child_entity<E>(self, child: E) -> EntityBuilder<(Self::Parts, ChildPart<E>)>
    where
        E: BuildableEntity,
    {
        EntityBuilder {
            parts: (self.parts, ChildPart { child }),
        }
    }

    fn children<F>(self, builder: F) -> EntityBuilder<(Self::Parts, ChildrenPart<F>)>
    where
        F: FnOnce(&mut EntityChildBuilder<'_>) + Any + Sync + Send,
    {
        EntityBuilder {
            parts: (self.parts, ChildrenPart { builder }),
        }
    }

    fn dependency<C, E, F>(self, f: F) -> EntityBuilder<(Self::Parts, DependencyPart<C, E, F>)>
    where
        C: Component<IsSingleton = True>,
        E: BuildableEntity,
        F: FnOnce() -> E + Any + Sync + Send,
    {
        EntityBuilder {
            parts: (
                self.parts,
                DependencyPart {
                    dependency_creation_fn: f,
                    phantom: PhantomData,
                },
            ),
        }
    }
}

impl<P> BuildableEntity for EntityBuilder<P> where P: BuiltEntityPart {}

impl<P> BuiltEntityPart for EntityBuilder<P>
where
    P: BuiltEntityPart,
{
    fn create_archetype(
        &mut self,
        core: &mut CoreStorage,
        archetype_idx: ArchetypeIdx,
    ) -> ArchetypeIdx {
        self.parts.create_archetype(core, archetype_idx)
    }

    fn add_components(&mut self, core: &mut CoreStorage, location: EntityLocation) {
        self.parts.add_components(core, location);
    }

    fn create_other_entities(self, core: &mut CoreStorage, parent_idx: Option<EntityIdx>) {
        self.parts.create_other_entities(core, parent_idx);
    }
}

/// A builder for defining children of an entity.
///
/// # Examples
///
/// ```rust
/// # use modor::*;
/// #
/// #[derive(Component, NoSystem)]
/// struct Value(u32);
///
/// fn build_root() -> impl BuiltEntity {
///     EntityBuilder::new()
///         .component(Value(0))
///         .children(|b| {
///             for i in 1..=10 {
///                 b.add(build_child(i));
///             }
///         })
/// }
///
/// fn build_child(value: u32) -> impl BuiltEntity {
///     EntityBuilder::new().component(Value(value))
/// }
/// ```
pub struct EntityChildBuilder<'a> {
    core: &'a mut CoreStorage,
    parent_idx: Option<EntityIdx>,
}

impl EntityChildBuilder<'_> {
    /// Adds a child entity.
    pub fn add(&mut self, child: impl BuildableEntity) {
        child.build(self.core, self.parent_idx);
    }
}

mod internal {
    use crate::storages::archetypes::{ArchetypeIdx, ArchetypeStorage, EntityLocation};
    use crate::storages::components::ComponentTypeIdx;
    use crate::storages::core::CoreStorage;
    use crate::storages::entities::EntityIdx;
    use crate::{BuildableEntity, ComponentSystems, EntityChildBuilder};
    use crate::{Component, SystemRunner, True};
    use std::any;
    use std::any::{Any, TypeId};
    use std::marker::PhantomData;

    #[allow(unused_variables)]
    pub trait BuiltEntityPart: Sized + Any + Sync + Send {
        fn create_archetype(
            &mut self,
            core: &mut CoreStorage,
            archetype_idx: ArchetypeIdx,
        ) -> ArchetypeIdx {
            archetype_idx
        }

        fn add_components(&mut self, core: &mut CoreStorage, location: EntityLocation) {}

        fn create_other_entities(self, core: &mut CoreStorage, parent_idx: Option<EntityIdx>) {}

        fn build(mut self, core: &mut CoreStorage, parent_idx: Option<EntityIdx>) -> EntityIdx {
            let archetype_idx = self.create_archetype(core, ArchetypeStorage::DEFAULT_IDX);
            let (entity_idx, location) = core.create_entity(archetype_idx, parent_idx);
            self.add_components(core, location);
            self.create_other_entities(core, Some(entity_idx));
            trace!("entity created with ID {}", entity_idx.0);
            entity_idx
        }
    }

    impl BuiltEntityPart for () {}

    impl<T, U> BuiltEntityPart for (T, U)
    where
        T: BuiltEntityPart,
        U: BuiltEntityPart,
    {
        fn create_archetype(
            &mut self,
            core: &mut CoreStorage,
            archetype_idx: ArchetypeIdx,
        ) -> ArchetypeIdx {
            let archetype_idx = self.0.create_archetype(core, archetype_idx);
            self.1.create_archetype(core, archetype_idx)
        }

        fn add_components(&mut self, core: &mut CoreStorage, location: EntityLocation) {
            self.0.add_components(core, location);
            self.1.add_components(core, location);
        }

        fn create_other_entities(self, core: &mut CoreStorage, parent_idx: Option<EntityIdx>) {
            self.0.create_other_entities(core, parent_idx);
            self.1.create_other_entities(core, parent_idx);
        }
    }

    pub struct ComponentPart<C> {
        pub(crate) component: Option<C>,
        pub(crate) type_idx: Option<ComponentTypeIdx>,
    }

    impl<C> BuiltEntityPart for ComponentPart<C>
    where
        C: Component + ComponentSystems,
    {
        fn create_archetype(
            &mut self,
            core: &mut CoreStorage,
            archetype_idx: ArchetypeIdx,
        ) -> ArchetypeIdx {
            if !core.components().has_systems_loaded::<C>() {
                let component_type_idx = core.set_systems_as_loaded::<C>();
                C::on_update(SystemRunner {
                    core,
                    component_action_type: TypeId::of::<C::Action>(),
                    component_type_idx,
                    action_idxs: vec![],
                });
            };
            if self.component.is_some() {
                let (type_idx, archetype_idx) = core.add_component_type::<C>(archetype_idx);
                self.type_idx = Some(type_idx);
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
                    TypeId::of::<C::IsSingleton>() == TypeId::of::<True>(),
                );
                trace!(
                    "component `{}` added to entity with ID {}",
                    any::type_name::<C>(),
                    core.archetypes().entity_idxs(location.idx)[location.pos].0
                );
            } else {
                trace!(
                    "component `{}` not added to entity with ID {} as condition is false",
                    any::type_name::<C>(),
                    core.archetypes().entity_idxs(location.idx)[location.pos].0
                );
            }
        }
    }

    pub struct InheritedPart<E> {
        pub(crate) entity: E,
    }

    impl<E> BuiltEntityPart for InheritedPart<E>
    where
        E: BuildableEntity,
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
        pub(crate) child: E,
    }

    impl<E> BuiltEntityPart for ChildPart<E>
    where
        E: BuildableEntity,
    {
        fn create_other_entities(self, core: &mut CoreStorage, parent_idx: Option<EntityIdx>) {
            self.child.build(core, parent_idx);
        }
    }

    pub struct ChildrenPart<F> {
        pub(crate) builder: F,
    }

    impl<F> BuiltEntityPart for ChildrenPart<F>
    where
        F: FnOnce(&mut EntityChildBuilder<'_>) + Any + Sync + Send,
    {
        fn create_other_entities(self, core: &mut CoreStorage, parent_idx: Option<EntityIdx>) {
            let mut builder = EntityChildBuilder { core, parent_idx };
            (self.builder)(&mut builder);
        }
    }

    pub struct DependencyPart<C, E, F> {
        pub(crate) dependency_creation_fn: F,
        pub(crate) phantom: PhantomData<(C, fn() -> E)>,
    }

    impl<C, E, F> BuiltEntityPart for DependencyPart<C, E, F>
    where
        C: Component<IsSingleton = True>,
        E: BuildableEntity,
        F: FnOnce() -> E + Any + Sync + Send,
    {
        fn create_other_entities(self, core: &mut CoreStorage, _parent_idx: Option<EntityIdx>) {
            let singleton_exists = core
                .components()
                .type_idx(TypeId::of::<C>())
                .and_then(|c| core.components().singleton_location(c))
                .is_some();
            if singleton_exists {
                trace!(
                    "dependency entity for singleton of type `{}` not created as already existing",
                    any::type_name::<F>(),
                );
            } else {
                (self.dependency_creation_fn)().build(core, None);
                trace!(
                    "dependency entity for singleton of type `{}` created",
                    any::type_name::<F>(),
                );
            }
        }
    }
}
