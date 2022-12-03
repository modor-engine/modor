use crate::storages::core::CoreStorage;
use crate::storages::systems::SystemProperties;
use crate::systems::context::SystemInfo;
use crate::system_params::internal::{LockableSystemParam, Mut, SystemParamWithLifetime};
use crate::system_params::world::internal::{WorldGuard, WorldStream};
use crate::world::internal::WorldGuardBorrow;
use crate::{Built, EntityMainComponent, SystemParam};
use std::any::{self, Any, TypeId};

/// A system parameter for applying actions on entities.
///
/// # Examples
///
/// ```rust
/// # use modor::{Entity, World};
/// #
/// fn add_string_component(mut world: World<'_>, entity: Entity<'_>) {
///     let component = format!("entity_{}", entity.id());
///     world.add_component(entity.id(), component);
/// }
/// ```
pub struct World<'a> {
    info: SystemInfo<'a>,
}

impl<'a> World<'a> {
    /// Creates a new root entity of type `E`.
    ///
    /// The entity is actually created once all registered systems have been run.
    pub fn create_root_entity<E, B>(&mut self, entity: B)
    where
        E: EntityMainComponent,
        B: Built<E>,
    {
        self.info
            .storages
            .updates
            .try_lock()
            .expect("internal error: cannot lock updates to create root entity")
            .create_entity(
                None,
                Box::new(|c| {
                    let entity_idx = entity.build(c, None);
                    trace!(
                        "root entity of type `{}` created with ID `{}`",
                        any::type_name::<E>(),
                        entity_idx.0
                    );
                }),
            );
    }

    /// Creates a new entity of type `E` with parent entity with ID `parent_id`.
    ///
    /// The entity is actually created once all registered systems have been run.
    pub fn create_child_entity<E, B>(&mut self, parent_id: usize, entity: B)
    where
        E: EntityMainComponent,
        B: Built<E>,
    {
        self.info
            .storages
            .updates
            .try_lock()
            .expect("internal error: cannot lock updates to create child entity")
            .create_entity(
                Some(parent_id.into()),
                Box::new(move |c| {
                    let entity_idx = entity.build(c, Some(parent_id.into()));
                    trace!(
                        "child entity of type `{}` created with ID `{}` for entity with ID {parent_id}",
                        any::type_name::<E>(),
                        entity_idx.0
                    );
                }),
            );
    }

    /// Deletes an entity.
    ///
    /// The entity is actually deleted once all registered systems have been run.
    pub fn delete_entity(&mut self, entity_id: usize) {
        self.info
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
    /// If the entity already has a component of type `C`, it is overwritten.<br>
    /// If `C` is the main component of an entity defined with the [`entity`](macro@crate::entity)
    /// or [`singleton`](macro@crate::singleton) proc macro, then systems defined for `C` will now
    /// be run for the entity.
    pub fn add_component<C>(&mut self, entity_id: usize, component: C)
    where
        C: Any + Sync + Send,
    {
        self.info
            .storages
            .updates
            .try_lock()
            .expect("internal error: cannot lock updates to add component")
            .add_component(
                entity_id.into(),
                |c, a| c.add_component_type::<C>(a).1,
                Box::new(move |c, l| {
                    let type_idx = c
                        .components()
                        .type_idx(TypeId::of::<C>())
                        .expect("internal error: add component with not registered type");
                    c.add_component::<C>(component, type_idx, l, false);
                    trace!(
                        "component of type `{}` added for entity with ID {entity_id}",
                        any::type_name::<C>()
                    );
                }),
            );
    }

    /// Deletes the component of type `C` from an entity.
    ///
    /// The component is actually deleted once all registered systems have been run.
    ///
    /// If the entity does not have a component of type `C`, nothing is done.<br>
    /// If `C` is the main component of an entity defined with the [`entity`](macro@crate::entity)
    /// or [`singleton`](macro@crate::singleton) proc macro, then systems defined for `C` will now
    /// be run for the entity.
    pub fn delete_component<C>(&mut self, entity_id: usize)
    where
        C: Any + Sync + Send,
    {
        if let Some(type_idx) = self.info.storages.components.type_idx(TypeId::of::<C>()) {
            self.info
                .storages
                .updates
                .try_lock()
                .expect("internal error: cannot lock updates to delete component")
                .delete_component(entity_id.into(), type_idx);
        }
        trace!(
            "component of type `{}` deleted from entity with ID {entity_id}",
            any::type_name::<C>()
        );
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
        }
    }

    fn lock(info: SystemInfo<'_>) -> <Self as SystemParamWithLifetime<'_>>::Guard {
        WorldGuard::new(info)
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
        stream
            .item_positions
            .next()
            .map(move |_| World { info: stream.info })
    }
}

impl LockableSystemParam for World<'_> {
    type LockedType = World<'static>;
    type Mutability = Mut;
}

mod internal {
    use crate::systems::context::SystemInfo;
    use std::ops::Range;

    pub struct WorldGuard<'a> {
        info: SystemInfo<'a>,
    }

    impl<'a> WorldGuard<'a> {
        pub(crate) fn new(info: SystemInfo<'a>) -> Self {
            Self { info }
        }

        pub(crate) fn borrow(&mut self) -> WorldGuardBorrow<'_> {
            WorldGuardBorrow {
                item_count: self.info.item_count,
                info: self.info,
            }
        }
    }

    pub struct WorldGuardBorrow<'a> {
        pub(crate) item_count: usize,
        pub(crate) info: SystemInfo<'a>,
    }

    pub struct WorldStream<'a> {
        pub(crate) info: SystemInfo<'a>,
        pub(crate) item_positions: Range<usize>,
    }

    impl<'a> WorldStream<'a> {
        pub(crate) fn new(guard: &'a WorldGuardBorrow<'_>) -> Self {
            Self {
                info: guard.info,
                item_positions: 0..guard.item_count,
            }
        }
    }
}
