use crate::entity_builders::child::EntityChildBuilder;
use crate::entity_builders::children::{EntityChildrenBuilder, EntityGenerator};
use crate::entity_builders::component::EntityComponentBuilder;
use crate::entity_builders::dependency::EntityDependencyBuilder;
use crate::entity_builders::inherited::EntityInheritedBuilder;
use crate::entity_builders::internal::BuiltEntityPart;
use crate::storages::archetypes::{ArchetypeIdx, EntityLocation};
use crate::storages::core::CoreStorage;
use crate::storages::entities::EntityIdx;
use crate::{Component, ComponentSystems, True};
use std::marker::PhantomData;

/// A trait implemented for an entity builder.
pub trait BuiltEntity: Sized + BuiltEntityPart {
    /// Adds a component of type `C`.
    ///
    /// If a component of type `C` already exists, it is overwritten.
    fn component<C>(self, component: C) -> EntityComponentBuilder<C, Self>
    where
        C: ComponentSystems,
    {
        EntityComponentBuilder {
            component: Some(component),
            type_idx: None,
            previous: self,
        }
    }

    /// Adds a component of type `C` only if `component` is not `None`.
    ///
    /// If `component` is not `None` and a component of type `C` already exists, it is overwritten.
    fn component_option<C>(self, component: Option<C>) -> EntityComponentBuilder<C, Self>
    where
        C: ComponentSystems,
    {
        EntityComponentBuilder {
            component,
            type_idx: None,
            previous: self,
        }
    }

    /// Inherits from another built entity.
    ///
    /// Components, children and systems of the parent entity are added to the built entity.
    ///
    /// If the parent has a component with a type already added to the built entity, the parent's
    /// component overwrites the existing one.<br>
    /// If after calling this method, a component with a type contained in the parent entity is
    /// added to the built entity, the new component overwrites the parent's.
    fn inherited<E>(self, entity: E) -> EntityInheritedBuilder<E, Self>
    where
        E: BuiltEntity,
    {
        EntityInheritedBuilder {
            entity,
            previous: self,
        }
    }

    /// Creates a child entity.
    fn child_entity<E>(self, child: E) -> EntityChildBuilder<E, Self>
    where
        E: BuiltEntity,
    {
        EntityChildBuilder {
            child,
            previous: self,
        }
    }

    /// Creates child entities.
    ///
    /// This method can be used instead of
    /// [`BuiltEntity::child_entity`](BuiltEntity::child_entity) when children are
    /// created dynamically (e.g. with conditional creation or loops).
    fn child_entities<F>(self, builder: F) -> EntityChildrenBuilder<F, Self>
    where
        F: FnOnce(&mut EntityGenerator<'_>),
    {
        EntityChildrenBuilder {
            builder,
            previous: self,
        }
    }

    /// Creates an entity if the singleton of type `C` does not already exist.
    ///
    /// The created entity has no parent.
    fn dependency<C, E, F>(self, builder: F) -> EntityDependencyBuilder<C, E, F, Self>
    where
        C: Component<IsSingleton = True>,
        E: BuiltEntity,
        F: FnOnce() -> E,
    {
        EntityDependencyBuilder {
            builder,
            previous: self,
            phantom: PhantomData,
        }
    }
}

impl<E> BuiltEntity for E where E: BuiltEntityPart {}

impl<C> BuiltEntityPart for C
where
    C: ComponentSystems,
{
    fn create_archetype(
        &mut self,
        _core: &mut CoreStorage,
        _archetype_idx: ArchetypeIdx,
    ) -> ArchetypeIdx {
        unreachable!()
    }

    fn add_components(&mut self, _core: &mut CoreStorage, _location: EntityLocation) {
        unreachable!()
    }

    fn create_other_entities(self, _core: &mut CoreStorage, _parent_idx: Option<EntityIdx>) {
        unreachable!()
    }

    fn build(self, core: &mut CoreStorage, parent_idx: Option<EntityIdx>) -> EntityIdx {
        EntityComponentBuilder {
            component: Some(self),
            type_idx: None,
            previous: EntityBuilder::new(),
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
#[non_exhaustive]
#[derive(Default)]
pub struct EntityBuilder;

impl EntityBuilder {
    /// Creates a new builder.
    pub const fn new() -> Self {
        Self
    }
}

impl BuiltEntityPart for EntityBuilder {
    fn create_archetype(
        &mut self,
        _core: &mut CoreStorage,
        archetype_idx: ArchetypeIdx,
    ) -> ArchetypeIdx {
        archetype_idx
    }

    fn add_components(&mut self, _core: &mut CoreStorage, _location: EntityLocation) {}

    fn create_other_entities(self, _core: &mut CoreStorage, _parent_idx: Option<EntityIdx>) {}
}

pub(crate) mod child;
pub(crate) mod children;
pub(crate) mod component;
pub(crate) mod dependency;
pub(crate) mod inherited;
mod internal;
