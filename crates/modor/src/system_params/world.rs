use crate::storages::core::CoreStorage;
use crate::storages::systems::SystemProperties;
use crate::system_params::internal::{LockableSystemParam, Mut, SystemParamWithLifetime};
use crate::system_params::world::internal::{WorldGuard, WorldStream};
use crate::world::internal::WorldGuardBorrow;
use crate::{Built, EntityMainComponent, SystemData, SystemInfo, SystemParam};
use std::any::{Any, TypeId};

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
    data: SystemData<'a>,
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
        self.data
            .updates
            .try_lock()
            .expect("internal error: cannot lock updates to create root entity")
            .create_entity(
                None,
                Box::new(|c| {
                    entity.build(c, None);
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
        self.data
            .updates
            .try_lock()
            .expect("internal error: cannot lock updates to create child entity")
            .create_entity(
                Some(parent_id.into()),
                Box::new(move |c| {
                    entity.build(c, Some(parent_id.into()));
                }),
            );
    }

    /// Deletes an entity.
    ///
    /// The entity is actually deleted once all registered systems have been run.
    pub fn delete_entity(&mut self, entity_id: usize) {
        self.data
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
        self.data
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
        if let Some(type_idx) = self.data.components.type_idx(TypeId::of::<C>()) {
            self.data
                .updates
                .try_lock()
                .expect("internal error: cannot lock updates to delete component")
                .delete_component(entity_id.into(), type_idx);
        }
    }
}

impl<'a> SystemParamWithLifetime<'a> for World<'_> {
    type Param = World<'a>;
    type Guard = WorldGuard<'a>;
    type GuardBorrow = WorldGuardBorrow<'a>;
    type Stream = WorldStream<'a>;
}

impl SystemParam for World<'_> {
    type Tuple = (Self,);
    type InnerTuple = ();

    fn properties(_core: &mut CoreStorage) -> SystemProperties {
        SystemProperties {
            component_types: vec![],
            can_update: true,
            filtered_component_type_idxs: vec![],
        }
    }

    fn lock<'a>(
        data: SystemData<'a>,
        info: SystemInfo<'a>,
    ) -> <Self as SystemParamWithLifetime<'a>>::Guard {
        WorldGuard::new(data, info)
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
            .map(move |_| World { data: stream.data })
    }
}

impl LockableSystemParam for World<'_> {
    type LockedType = World<'static>;
    type Mutability = Mut;
}

mod internal {
    use crate::{SystemData, SystemInfo};
    use std::ops::Range;

    pub struct WorldGuard<'a> {
        data: SystemData<'a>,
        info: SystemInfo<'a>,
    }

    impl<'a> WorldGuard<'a> {
        pub(crate) fn new(data: SystemData<'a>, info: SystemInfo<'a>) -> Self {
            Self { data, info }
        }

        pub(crate) fn borrow(&mut self) -> WorldGuardBorrow<'_> {
            WorldGuardBorrow {
                item_count: self.info.item_count,
                data: self.data,
            }
        }
    }

    pub struct WorldGuardBorrow<'a> {
        pub(crate) item_count: usize,
        pub(crate) data: SystemData<'a>,
    }

    pub struct WorldStream<'a> {
        pub(crate) data: SystemData<'a>,
        pub(crate) item_positions: Range<usize>,
    }

    impl<'a> WorldStream<'a> {
        pub(crate) fn new(guard: &'a WorldGuardBorrow<'_>) -> Self {
            Self {
                data: guard.data,
                item_positions: 0..guard.item_count,
            }
        }
    }
}
