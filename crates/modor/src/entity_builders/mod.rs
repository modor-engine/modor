use crate::entity_builders::child_entities::{EntityChildEntitiesBuilder, EntityGenerator};
use crate::entity_builders::child_entity::EntityChildEntityBuilder;
use crate::entity_builders::component::EntityComponentBuilder;
use crate::entity_builders::dependency::EntityDependencyBuilder;
use crate::entity_builders::inherited::EntityInheritedBuilder;
use crate::entity_builders::internal::BuiltEntityPart;
use crate::storages::archetypes::{ArchetypeIdx, EntityLocation};
use crate::storages::core::CoreStorage;
use crate::storages::entities::EntityIdx;
use crate::{Component, ComponentSystems, EntityChildComponentBuilder, True};
use std::marker::PhantomData;

/// A trait implemented for an entity builder.
pub trait BuiltEntity: Sized + BuiltEntityPart {
    /// Adds a component of type `C`.
    ///
    /// If a component of type `C` already exists, it is overwritten.
    fn component<C>(self, component: C) -> EntityBuilder<Self, EntityComponentBuilder<C>>
    where
        C: ComponentSystems,
    {
        EntityBuilder {
            previous: self,
            last: EntityComponentBuilder {
                component: Some(component),
                type_idx: None,
            },
        }
    }

    /// Adds a component of type `C` only if `component` is not `None`.
    ///
    /// If `component` is not `None` and a component of type `C` already exists, it is overwritten.
    fn component_option<C>(
        self,
        component: Option<C>,
    ) -> EntityBuilder<Self, EntityComponentBuilder<C>>
    where
        C: ComponentSystems,
    {
        EntityBuilder {
            previous: self,
            last: EntityComponentBuilder {
                component,
                type_idx: None,
            },
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
    fn inherited<E>(self, entity: E) -> EntityBuilder<Self, EntityInheritedBuilder<E>>
    where
        E: BuiltEntity,
    {
        EntityBuilder {
            previous: self,
            last: EntityInheritedBuilder { entity },
        }
    }

    /// Creates a child entity containing a single `component`.
    fn child_component<C>(self, component: C) -> EntityBuilder<Self, EntityChildComponentBuilder<C>>
    where
        C: ComponentSystems,
    {
        EntityBuilder {
            previous: self,
            last: EntityChildComponentBuilder { component },
        }
    }

    /// Creates a child entity.
    fn child_entity<E>(self, child: E) -> EntityBuilder<Self, EntityChildEntityBuilder<E>>
    where
        E: BuiltEntity,
    {
        EntityBuilder {
            previous: self,
            last: EntityChildEntityBuilder { child },
        }
    }

    /// Creates child entities.
    ///
    /// This method can be used instead of
    /// [`BuiltEntity::child_entity`](BuiltEntity::child_entity) when children are
    /// created dynamically (e.g. with conditional creation or loops).
    fn child_entities<F>(self, builder: F) -> EntityBuilder<Self, EntityChildEntitiesBuilder<F>>
    where
        F: FnOnce(&mut EntityGenerator<'_>),
    {
        EntityBuilder {
            previous: self,
            last: EntityChildEntitiesBuilder { builder },
        }
    }

    /// Creates an entity if the singleton of type `C` does not already exist.
    ///
    /// The created entity has no parent.
    fn dependency<C, E, F>(
        self,
        builder: F,
    ) -> EntityBuilder<Self, EntityDependencyBuilder<C, E, F>>
    where
        C: Component<IsSingleton = True>,
        E: BuiltEntity,
        F: FnOnce() -> E,
    {
        EntityBuilder {
            previous: self,
            last: EntityDependencyBuilder {
                builder,
                phantom: PhantomData,
            },
        }
    }

    /// Updates the component of type `C` already added to the entity.
    ///
    /// If the entity does not have a component of type `C`, nothing is done.
    fn updated<C>(mut self, updater: impl FnMut(&mut C)) -> Self
    where
        C: Component,
    {
        self.update_component(updater);
        self
    }
}

impl<E> BuiltEntity for E where E: BuiltEntityPart {}

/// A trait implemented for types that can be used to create an entity.
pub trait BuildableEntity<S> {
    #[doc(hidden)]
    fn build_entity(self, core: &mut CoreStorage, parent_idx: Option<EntityIdx>) -> EntityIdx;
}

/// The [`BuildableEntity`] source when the entity is created from a single component.
pub struct ComponentSource;

/// The [`BuildableEntity`] source when the entity is created from a type implementing [`BuiltEntity`].
pub struct BuiltEntitySource;

impl<C> BuildableEntity<ComponentSource> for C
where
    C: ComponentSystems,
{
    fn build_entity(self, core: &mut CoreStorage, parent_idx: Option<EntityIdx>) -> EntityIdx {
        EntityComponentBuilder {
            component: Some(self),
            type_idx: None,
        }
        .build(core, parent_idx)
    }
}

impl<E> BuildableEntity<BuiltEntitySource> for E
where
    E: BuiltEntity,
{
    fn build_entity(self, core: &mut CoreStorage, parent_idx: Option<EntityIdx>) -> EntityIdx {
        BuiltEntityPart::build(self, core, parent_idx)
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
#[derive(Default)]
pub struct EntityBuilder<P = EntityBuilderRoot, L = EntityBuilderRoot> {
    previous: P,
    last: L,
}

impl EntityBuilder {
    /// Creates a new builder.
    pub const fn new() -> Self {
        Self {
            previous: EntityBuilderRoot,
            last: EntityBuilderRoot,
        }
    }
}

impl<T, U> BuiltEntityPart for EntityBuilder<T, U>
where
    T: BuiltEntityPart,
    U: BuiltEntityPart,
{
    fn create_archetype(
        &mut self,
        core: &mut CoreStorage,
        archetype_idx: ArchetypeIdx,
    ) -> ArchetypeIdx {
        let archetype_idx = self.previous.create_archetype(core, archetype_idx);
        self.last.create_archetype(core, archetype_idx)
    }

    fn add_components(&mut self, core: &mut CoreStorage, location: EntityLocation) {
        self.previous.add_components(core, location);
        self.last.add_components(core, location);
    }

    fn create_other_entities(self, core: &mut CoreStorage, parent_idx: Option<EntityIdx>) {
        self.previous.create_other_entities(core, parent_idx);
        self.last.create_other_entities(core, parent_idx);
    }

    fn update_component<C>(&mut self, mut updater: impl FnMut(&mut C))
    where
        C: Component,
    {
        self.last.update_component(&mut updater);
        self.previous.update_component(updater);
    }
}

/// An entity builder that has no effect.
///
/// It is used as root type for the [`EntityBuilder`](EntityBuilder) type.
#[non_exhaustive]
pub struct EntityBuilderRoot;

impl BuiltEntityPart for EntityBuilderRoot {}

pub(crate) mod child_component;
pub(crate) mod child_entities;
pub(crate) mod child_entity;
pub(crate) mod component;
pub(crate) mod dependency;
pub(crate) mod inherited;
mod internal;
