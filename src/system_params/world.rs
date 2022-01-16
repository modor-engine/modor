use crate::storages::archetypes::ArchetypeFilter;
use crate::storages::core::CoreStorage;
use crate::storages::systems::SystemProperties;
use crate::system_params::internal::{LockableSystemParam, Mut, SystemParamWithLifetime};
use crate::system_params::world::internal::{WorldGuard, WorldStream};
use crate::world::internal::WorldGuardBorrow;
use crate::{SystemData, SystemInfo, SystemParam};
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
    /// Deletes an entity.
    ///
    /// The entity is actually deleted once all registered systems have been run.
    pub fn delete_entity(&mut self, entity_id: usize) {
        self.data
            .updates
            .try_lock()
            .expect("internal error: cannot lock entity actions to delete entity")
            .delete_entity(entity_id.into());
    }

    /// Adds a component of type `C` to an entity.
    ///
    /// The component is actually added once all registered systems have been run.
    ///
    /// If the entity already has a component of type `C`, it is overwritten.<br>
    /// If `C` implements [`EntityMainComponent`](crate::EntityMainComponent),
    /// systems defined for `C` will now be run for the entity.
    pub fn add_component<C>(&mut self, entity_id: usize, component: C)
    where
        C: Any + Sync + Send,
    {
        self.data
            .updates
            .try_lock()
            .expect("internal error: cannot lock entity actions to add component")
            .add_component(
                entity_id.into(),
                |c, a| c.add_component_type::<C>(a).1,
                Box::new(move |c, l| {
                    let type_idx = c
                        .components()
                        .type_idx(TypeId::of::<C>())
                        .expect("internal error: add component with not registered type");
                    c.add_component::<C>(component, type_idx, l);
                }),
            );
    }

    /// Deletes the component of type `C` from an entity.
    ///
    /// The component is actually deleted once all registered systems have been run.
    ///
    /// If the entity does not have a component of type `C`, nothing is done.<br>
    /// If `C` implements [`EntityMainComponent`](crate::EntityMainComponent),
    /// systems defined for `C` will not be run anymore for the entity.
    pub fn delete_component<C>(&mut self, entity_id: usize)
    where
        C: Any + Sync + Send,
    {
        if let Some(type_idx) = self.data.components.type_idx(TypeId::of::<C>()) {
            self.data
                .updates
                .try_lock()
                .expect("internal error: cannot lock entity actions to delete component")
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
            archetype_filter: ArchetypeFilter::None,
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
            .entity_positions
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
        pub(crate) entity_positions: Range<usize>,
    }

    impl<'a> WorldStream<'a> {
        pub(crate) fn new(guard: &'a WorldGuardBorrow<'_>) -> Self {
            Self {
                data: guard.data,
                entity_positions: 0..guard.item_count,
            }
        }
    }
}

#[cfg(test)]
mod world_tests {
    use crate::storages::archetypes::{ArchetypeStorage, EntityLocationInArchetype};
    use crate::storages::core::CoreStorage;
    use crate::storages::entities::EntityIdx;
    use crate::storages::updates::EntityUpdate;
    use crate::World;
    use std::any::TypeId;

    assert_impl_all!(World<'_>: Sync, Send, Unpin);

    #[test]
    fn delete_entity() {
        let core = CoreStorage::default();
        let data = core.system_data();
        let mut world = World { data };

        world.delete_entity(2);

        let mut updates = data.updates.try_lock().unwrap();
        let entity_updates: Vec<_> = updates.drain_entity_updates().collect();
        assert_eq!(entity_updates.len(), 1);
        assert!(matches!(
            entity_updates[0],
            (EntityIdx(2), EntityUpdate::Deleted)
        ));
    }

    #[test]
    fn add_component() {
        let mut core = CoreStorage::default();
        let data = core.system_data();
        let mut world = World { data };

        world.add_component(0, 10_u32);

        let mut entity_updates: Vec<_> = {
            let mut updates = data.updates.try_lock().unwrap();
            updates.drain_entity_updates().collect()
        };
        assert_eq!(entity_updates.len(), 1);
        let state = entity_updates.pop().unwrap();
        assert_eq!(state.0, 0.into());
        if let EntityUpdate::Updated(mut add_fns, deleted_type_idxs) = state.1 {
            assert_eq!(add_fns.len(), 1);
            assert_eq!(core.components().type_idx(TypeId::of::<u32>()), None);
            (add_fns[0].add_type_fn)(&mut core, ArchetypeStorage::DEFAULT_IDX);
            let component_type_idx = core.components().type_idx(TypeId::of::<u32>());
            assert_eq!(component_type_idx, Some(0.into()));
            let src_location = EntityLocationInArchetype {
                idx: ArchetypeStorage::DEFAULT_IDX,
                pos: 0.into(),
            };
            let (_, dst_archetype_idx) = core.add_component_type::<u32>(src_location.idx);
            let dst_location = core.create_entity(dst_archetype_idx);
            (add_fns.pop().unwrap().add_fn)(&mut core, dst_location);
            let components = core.components().read_components::<u32>();
            assert_eq!(&*components, &ti_vec![ti_vec![], ti_vec![10_u32]]);
            assert_eq!(deleted_type_idxs, &[]);
        } else {
            panic!("assertion failed: `state.1` matches `EntityState::Unchanged(_, _, _)`");
        }
    }

    #[test]
    fn delete_component() {
        let mut core = CoreStorage::default();
        core.add_component_type::<i64>(ArchetypeStorage::DEFAULT_IDX);
        core.add_component_type::<u32>(ArchetypeStorage::DEFAULT_IDX);
        let data = core.system_data();
        let mut world = World { data };

        world.delete_component::<u32>(2);

        let mut updates = data.updates.try_lock().unwrap();
        let mut entity_updates: Vec<_> = updates.drain_entity_updates().collect();
        assert_eq!(entity_updates.len(), 1);
        let state = entity_updates.pop().unwrap();
        assert_eq!(state.0, 2.into());
        if let EntityUpdate::Updated(add_fns, deleted_type_idxs) = state.1 {
            assert_eq!(add_fns.len(), 0);
            assert_eq!(deleted_type_idxs, &[1.into()]);
        } else {
            panic!("assertion failed: `states[0].1` matches `EntityState::Unchanged(_, _, _)`");
        }
    }
}

#[cfg(test)]
mod world_system_param_tests {
    use crate::storages::archetypes::{ArchetypeFilter, ArchetypeStorage};
    use crate::storages::core::CoreStorage;
    use crate::world::internal::WorldGuardBorrow;
    use crate::{SystemInfo, SystemParam, World};

    #[test]
    fn retrieve_properties() {
        let mut core = CoreStorage::default();

        let properties = World::properties(&mut core);

        assert_eq!(properties.component_types.len(), 0);
        assert!(properties.can_update);
        assert_eq!(properties.archetype_filter, ArchetypeFilter::None);
    }

    #[test]
    fn lock() {
        let mut core = CoreStorage::default();
        let archetype1_idx = ArchetypeStorage::DEFAULT_IDX;
        let (type_idx, archetype2_idx) = core.add_component_type::<u32>(archetype1_idx);
        let location = core.create_entity(archetype2_idx);
        core.add_component(10_u32, type_idx, location);
        let data = core.system_data();
        let info = SystemInfo {
            filtered_component_type_idxs: &[0.into()],
            archetype_filter: &ArchetypeFilter::All,
            item_count: 1,
        };

        let mut guard = World::lock(data, info);
        let guard_borrow = World::borrow_guard(&mut guard);

        assert_eq!(guard_borrow.item_count, 1);
    }

    #[test]
    fn retrieve_stream() {
        let core = CoreStorage::default();
        let mut guard_borrow = WorldGuardBorrow {
            item_count: 3,
            data: core.system_data(),
        };

        let mut stream = World::stream(&mut guard_borrow);

        assert!(World::stream_next(&mut stream).is_some());
        assert!(World::stream_next(&mut stream).is_some());
        assert!(World::stream_next(&mut stream).is_some());
        assert!(World::stream_next(&mut stream).is_none());
    }
}
