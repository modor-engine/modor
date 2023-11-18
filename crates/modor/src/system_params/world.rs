use crate::storages::core::CoreStorage;
use crate::storages::systems::SystemProperties;
use crate::system_params::internal::{LockableSystemParam, Mut};
use crate::system_params::world::internal::{WorldGuard, WorldStream};
use crate::systems::context::SystemContext;
use crate::world::internal::WorldGuardBorrow;
use crate::{
    BuildableEntity, Component, ComponentSystems, SystemParam, SystemParamWithLifetime,
    SystemRunner, VariableSend, VariableSync,
};
use std::any::{self, Any, TypeId};

/// A system parameter for applying actions on entities.
///
/// # Examples
///
/// ```rust
/// # use modor::*;
/// #
/// #[derive(Component, NoSystem)]
/// struct Name(String);
///
/// fn add_string_component(mut world: World<'_>, entity: Entity<'_>) {
///     let component = Name(format!("entity_{}", entity.id()));
///     world.add_component(entity.id(), component);
/// }
/// ```
///
/// Note that for this specific case, it is shorter to use [`EntityMut`](crate::EntityMut).
pub struct World<'a> {
    context: SystemContext<'a>,
}

impl<'a> World<'a> {
    /// Creates a new root entity.
    ///
    /// The entity is actually created once all registered systems have been run.
    pub fn create_root_entity<T>(
        &mut self,
        entity: impl BuildableEntity<T> + Any + VariableSync + VariableSend,
    ) {
        self.context
            .storages
            .updates
            .try_lock()
            .expect("internal error: cannot lock updates to create root entity")
            .create_entity(
                None,
                Box::new(|c| {
                    let entity_idx = entity.build_entity(c, None);
                    trace!("root entity created with ID {}", entity_idx.0);
                }),
            );
    }

    /// Creates a new entity with parent entity with ID `parent_id`.
    ///
    /// The entity is actually created once all registered systems have been run.
    pub fn create_child_entity<T>(
        &mut self,
        parent_id: usize,
        entity: impl BuildableEntity<T> + Any + VariableSync + VariableSend,
    ) {
        self.context
            .storages
            .updates
            .try_lock()
            .expect("internal error: cannot lock updates to create child entity")
            .create_entity(
                Some(parent_id.into()),
                Box::new(move |c| {
                    let entity_idx = entity.build_entity(c, Some(parent_id.into()));
                    trace!(
                        "child entity created with ID `{}` for entity with ID {parent_id}", // no-coverage
                        entity_idx.0 // no-coverage
                    );
                }),
            );
    }

    /// Deletes an entity.
    ///
    /// The entity is actually deleted once all registered systems have been run.
    pub fn delete_entity(&mut self, entity_id: usize) {
        self.context
            .storages
            .updates
            .try_lock()
            .expect("internal error: cannot lock updates to delete entity")
            .delete_entity(entity_id.into());
    }

    /// Adds a component of type `C` to an entity.
    ///
    /// The component is actually added once all registered systems have been run.
    ///
    /// If the entity already has a component of type `C`, it is overwritten.
    pub fn add_component<C>(&mut self, entity_id: usize, component: C)
    where
        C: ComponentSystems,
    {
        self.context
            .storages
            .updates
            .try_lock()
            .expect("internal error: cannot lock updates to add component")
            .add_component(
                entity_id.into(),
                |c, a| c.add_component_type::<C>(a).1,
                Box::new(move |c, l| {
                    let type_idx = if c.components().has_systems_loaded::<C>() {
                        c.components()
                            .type_idx(TypeId::of::<C>())
                            .expect("internal error: add component with not registered type")
                    } else {
                        let component_type_idx = c.set_systems_as_loaded::<C>();
                        C::on_update(SystemRunner {
                            core: c,
                            component_action_type: TypeId::of::<C::Action>(),
                            component_type_idx,
                            action_idxs: vec![],
                        });
                        component_type_idx
                    };
                    c.add_component(component, type_idx, l, false);
                    trace!(
                        "component of type `{}` added for entity with ID {entity_id}", // no-coverage
                        any::type_name::<C>()                                          // no-coverage
                    );
                }),
            );
    }

    /// Deletes the component of type `C` from an entity.
    ///
    /// The component is actually deleted once all registered systems have been run.
    ///
    /// If the entity does not have a component of type `C`, nothing is done.
    pub fn delete_component<C>(&mut self, entity_id: usize)
    where
        C: Component,
    {
        if let Some(type_idx) = self.context.storages.components.type_idx(TypeId::of::<C>()) {
            self.context
                .storages
                .updates
                .try_lock()
                .expect("internal error: cannot lock updates to delete component")
                .delete_component(entity_id.into(), type_idx);
        }
        trace!(
            "component of type `{}` deleted from entity with ID {entity_id}", // no-coverage
            any::type_name::<C>()                                             // no-coverage
        );
    }

    /// Returns IDs of the entities transformed at the end of the previous [`App`](crate::App)
    /// update.
    ///
    /// An entity is considered as transformed if a component has been added or deleted.
    ///
    /// Some return IDs might be duplicated.<br>
    /// It is possible that some of the returned IDs correspond to deleted entities.
    pub fn transformed_entity_ids(&self) -> impl Iterator<Item = usize> + '_ {
        self.context
            .storages
            .entities
            .moved_idxs()
            .iter()
            .map(|&i| i.into())
    }

    /// Returns IDs of the entities deleted at the end of the previous [`App`](crate::App) update.
    ///
    /// Some return IDs might be duplicated.
    pub fn deleted_entity_ids(&self) -> impl Iterator<Item = usize> + '_ {
        self.context
            .storages
            .entities
            .deleted_idxs()
            .iter()
            .map(|&i| i.into())
    }
}

impl<'a> SystemParamWithLifetime<'a> for World<'_> {
    type Param = World<'a>;
    type Guard = WorldGuard<'a>;
    type GuardBorrow = WorldGuardBorrow<'a>;
    type Stream = WorldStream<'a>;
}

impl SystemParam for World<'_> {
    type Filter = ();
    type InnerTuple = ();

    fn properties(_core: &mut CoreStorage) -> SystemProperties {
        SystemProperties {
            component_types: vec![],
            can_update: true,
            mutation_component_type_idxs: vec![],
        }
    }

    fn lock(context: SystemContext<'_>) -> <Self as SystemParamWithLifetime<'_>>::Guard {
        WorldGuard::new(context)
    }

    fn borrow_guard<'a, 'b>(
        guard: &'a mut <Self as SystemParamWithLifetime<'b>>::Guard,
    ) -> <Self as SystemParamWithLifetime<'a>>::GuardBorrow
    where
        'b: 'a,
    {
        guard.borrow()
    }

    fn stream<'a, 'b>(
        guard: &'a mut <Self as SystemParamWithLifetime<'b>>::GuardBorrow,
    ) -> <Self as SystemParamWithLifetime<'a>>::Stream
    where
        'b: 'a,
    {
        WorldStream::new(guard)
    }

    #[inline]
    fn stream_next<'a, 'b>(
        stream: &'a mut <Self as SystemParamWithLifetime<'b>>::Stream,
    ) -> Option<<Self as SystemParamWithLifetime<'a>>::Param>
    where
        'b: 'a,
    {
        stream.next()
    }
}

impl LockableSystemParam for World<'_> {
    type LockedType = World<'static>;
    type Mutability = Mut;
}

pub(super) mod internal {
    use crate::systems::context::SystemContext;
    use crate::World;
    use std::ops::Range;

    pub struct WorldGuard<'a> {
        context: SystemContext<'a>,
    }

    impl<'a> WorldGuard<'a> {
        pub(crate) fn new(context: SystemContext<'a>) -> Self {
            Self { context }
        }

        pub(crate) fn borrow(&mut self) -> WorldGuardBorrow<'_> {
            WorldGuardBorrow {
                item_count: self.context.item_count,
                context: self.context,
            }
        }
    }

    pub struct WorldGuardBorrow<'a> {
        pub(crate) item_count: usize,
        pub(crate) context: SystemContext<'a>,
    }

    pub struct WorldStream<'a> {
        pub(crate) context: SystemContext<'a>,
        pub(crate) item_positions: Range<usize>,
    }

    impl<'a> WorldStream<'a> {
        pub(crate) fn new(guard: &'a WorldGuardBorrow<'_>) -> Self {
            Self {
                context: guard.context,
                item_positions: 0..guard.item_count,
            }
        }

        pub(crate) fn next(&mut self) -> Option<World<'_>> {
            self.item_positions.next().map(move |_| World {
                context: self.context,
            })
        }
    }
}
