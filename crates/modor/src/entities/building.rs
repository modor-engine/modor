use self::internal::{BuildEntity, BuildEntityPart};
use crate::storages::archetypes::{ArchetypeIdx, ArchetypeStorage, EntityLocation};
use crate::storages::components::ComponentTypeIdx;
use crate::storages::core::CoreStorage;
use crate::storages::entities::EntityIdx;
use crate::{Component, EntityMainComponent, False, Inheritable, SystemRunner, True};
use std::any;
use std::any::TypeId;
use std::marker::PhantomData;

// TODO: export rarely used types in a public submodule of the crate

/// A trait implemented for all types able to build an entity.
///
/// This trait is particularly useful when defining a building method for an entity.
///
/// # Examples
///
/// See [`EntityBuilder`](crate::EntityBuilder).
#[must_use]
pub trait Built<E>: BuildEntity + BuildEntityPart
where
    E: EntityMainComponent,
{
    /// Retrieves an immutable reference to the main component of the entity.
    fn main(&self) -> &E;

    /// Retrieves a mutable reference to the main component of the entity.
    fn main_mut(&mut self) -> &mut E;
}

/// A builder for defining the components and children of an entity.
///
/// # Examples
///
///
/// ```rust
/// # use modor::*;
/// #
/// #[derive(Component)]
/// struct Position(f32, f32);
/// #[derive(Component)]
/// struct Velocity(f32, f32);
/// #[derive(Component)]
/// struct Acceleration(f32, f32);
///
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
pub struct EntityBuilder<E, P> {
    main_component: ComponentPart<E, E>,
    parts: P,
}

impl<E> EntityBuilder<E, ()>
where
    E: EntityMainComponent,
{
    /// Starts building an entity by providing its `main_component`.
    pub fn new(main_component: E) -> Self {
        Self {
            main_component: ComponentPart {
                component: Some(main_component),
                type_idx: None,
                phantom: PhantomData,
            },
            parts: (),
        }
    }
}

impl<E, P> EntityBuilder<E, P>
where
    E: EntityMainComponent,
{
    /// Adds a component of type `C`.
    ///
    /// If a component of type `C` already exists, it is overwritten.
    pub fn with<C>(self, component: C) -> EntityBuilder<E, (P, ComponentPart<E, C>)>
    where
        C: Component<IsEntityMainComponent = False>,
    {
        EntityBuilder {
            main_component: self.main_component,
            parts: (
                self.parts,
                ComponentPart {
                    component: Some(component),
                    type_idx: None,
                    phantom: PhantomData,
                },
            ),
        }
    }

    /// Adds a component of type `C` only if `component` is not `None`.
    ///
    /// If `component` is not `None` and a component of type `C` already exists, it is overwritten.
    pub fn with_option<C>(self, component: Option<C>) -> EntityBuilder<E, (P, ComponentPart<E, C>)>
    where
        C: Component<IsEntityMainComponent = False>,
    {
        EntityBuilder {
            main_component: self.main_component,
            parts: (
                self.parts,
                ComponentPart {
                    component,
                    type_idx: None,
                    phantom: PhantomData,
                },
            ),
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
    ) -> EntityBuilder<E, (P, InheritedPart<impl Built<I>>)>
    where
        I: EntityMainComponent + Inheritable<E>,
    {
        EntityBuilder {
            main_component: self.main_component,
            parts: (self.parts, InheritedPart { entity: inherited }),
        }
    }

    /// Creates a child entity with main component of type `C`.
    pub fn with_child<C>(
        self,
        child: impl Built<C>,
    ) -> EntityBuilder<E, (P, ChildPart<impl Built<C>>)>
    where
        C: EntityMainComponent,
    {
        EntityBuilder {
            main_component: self.main_component,
            parts: (self.parts, ChildPart { entity: child }),
        }
    }

    /// Creates child entities.
    ///
    /// This method can be used instead of
    /// [`EntityBuilder::with_child`](crate::EntityBuilder::with_child) when children are
    /// created dynamically (e.g. with conditional creation or loops).
    pub fn with_children<F>(self, build_fn: F) -> EntityBuilder<E, (P, ChildrenPart<F>)>
    where
        F: FnOnce(&mut ChildBuilder<'_>),
    {
        EntityBuilder {
            main_component: self.main_component,
            parts: (self.parts, ChildrenPart { build_fn }),
        }
    }

    /// Creates a singleton entity with main component of type `D` if the singleton does
    /// not already exist.
    ///
    /// The created entity has no parent.
    pub fn with_dependency<D>(
        self,
        dependency: impl Built<D>,
    ) -> EntityBuilder<E, (P, DependencyPart<D, impl Built<D>>)>
    where
        D: EntityMainComponent<IsSingleton = True>,
    {
        EntityBuilder {
            main_component: self.main_component,
            parts: (
                self.parts,
                DependencyPart {
                    entity: dependency,
                    phantom: PhantomData,
                },
            ),
        }
    }

    fn init_entity_type(core: &mut CoreStorage) {
        if let Some(type_idx) = core.components().type_idx(TypeId::of::<E>()) {
            if let Some(location) = core.components().singleton_location(type_idx) {
                let entity_idx = core.archetypes().entity_idxs(location.idx)[location.pos];
                core.delete_entity(entity_idx);
                warn!("singleton `{}` has been replaced", any::type_name::<E>());
            }
        }
    }
}

impl<E, P> BuildEntity for EntityBuilder<E, P>
where
    E: EntityMainComponent,
    P: BuildEntityPart,
{
    fn build(mut self, core: &mut CoreStorage, parent_idx: Option<EntityIdx>) -> EntityIdx {
        let archetype_idx = self.create_archetype(core, ArchetypeStorage::DEFAULT_IDX);
        let (entity_idx, location) = core.create_entity(archetype_idx, parent_idx);
        self.add_components(core, location);
        self.create_other_entities(core, Some(entity_idx));
        entity_idx
    }
}

impl<E, P> BuildEntityPart for EntityBuilder<E, P>
where
    E: EntityMainComponent,
    P: BuildEntityPart,
{
    fn create_archetype(
        &mut self,
        core: &mut CoreStorage,
        archetype_idx: ArchetypeIdx,
    ) -> ArchetypeIdx {
        Self::init_entity_type(core);
        let archetype_idx = self.main_component.create_archetype(core, archetype_idx);
        self.parts.create_archetype(core, archetype_idx)
    }

    fn add_components(&mut self, core: &mut CoreStorage, location: EntityLocation) {
        self.main_component.add_components(core, location);
        self.parts.add_components(core, location);
    }

    fn create_other_entities(self, core: &mut CoreStorage, parent_idx: Option<EntityIdx>) {
        self.main_component.create_other_entities(core, parent_idx);
        self.parts.create_other_entities(core, parent_idx);
    }
}

impl<E, P> Built<E> for EntityBuilder<E, P>
where
    E: EntityMainComponent,
    P: BuildEntityPart,
{
    fn main(&self) -> &E {
        self.main_component
            .component
            .as_ref()
            .expect("internal error: missing main component of entity")
    }

    fn main_mut(&mut self) -> &mut E {
        self.main_component
            .component
            .as_mut()
            .expect("internal error: missing main component of entity")
    }
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

pub struct ComponentPart<E, C> {
    component: Option<C>,
    type_idx: Option<ComponentTypeIdx>,
    phantom: PhantomData<E>,
}

impl<E, C> BuildEntityPart for ComponentPart<E, C>
where
    E: EntityMainComponent,
    C: Component,
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
                component_action_type: TypeId::of::<<C as Component>::Action>(),
                component_type_idx,
                action_idxs: vec![],
            });
        };
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
                    && TypeId::of::<E::IsSingleton>() == TypeId::of::<True>(),
            );
            trace!(
                "component `{}` added to entity of type `{}`",
                any::type_name::<C>(),
                any::type_name::<E>(),
            );
        } else {
            trace!(
                "component `{}` not added to entity of type `{}` as condition is false",
                any::type_name::<C>(),
                any::type_name::<E>(),
            );
        }
    }
}

pub struct InheritedPart<E> {
    entity: E,
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
    entity: E,
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
    build_fn: F,
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
    entity: B,
    phantom: PhantomData<E>,
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
        if singleton_exists {
            trace!(
                "dependency entity of type `{}` not created as already existing",
                any::type_name::<E>(),
            );
        } else {
            self.entity.build(core, None);
            trace!(
                "dependency entity of type `{}` created",
                any::type_name::<E>(),
            );
        }
    }
}

mod internal {
    use crate::storages::archetypes::{ArchetypeIdx, EntityLocation};
    use crate::storages::core::CoreStorage;
    use crate::storages::entities::EntityIdx;
    use std::any::Any;

    pub trait BuildEntity: Any + Sync + Send {
        fn build(self, core: &mut CoreStorage, parent_idx: Option<EntityIdx>) -> EntityIdx;
    }

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

    impl<T, U> BuildEntityPart for (T, U)
    where
        T: BuildEntityPart,
        U: BuildEntityPart,
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
}
