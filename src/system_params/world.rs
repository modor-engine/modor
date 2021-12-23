use crate::storages::core::CoreStorage;
use crate::storages::systems::SystemProperties;
use crate::system_params::internal::{
    LockableSystemParam, Mut, SystemParamIterInfo, SystemParamWithLifetime,
};
use crate::system_params::world::internal::WorldStream;
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
    data: &'a SystemData<'a>,
}

impl<'a> World<'a> {
    /// Deletes an entity.
    ///
    /// The entity is actually deleted once all registered systems have been run.
    pub fn delete_entity(&mut self, entity_id: usize) {
        self.data
            .entity_actions
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
            .entity_actions
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
                .entity_actions
                .try_lock()
                .expect("internal error: cannot lock entity actions to delete component")
                .delete_component(entity_id.into(), type_idx);
        }
    }
}

impl<'a> SystemParamWithLifetime<'a> for World<'_> {
    type Param = World<'a>;
    type Guard = &'a SystemData<'a>;
    type GuardBorrow = &'a SystemData<'a>;
    type Stream = WorldStream<'a>;
}

impl SystemParam for World<'_> {
    type Tuple = (Self,);
    type InnerTuple = ();

    fn properties(_core: &mut CoreStorage) -> SystemProperties {
        SystemProperties {
            component_types: vec![],
            has_entity_actions: true,
        }
    }

    fn iter_info(_data: &SystemData<'_>, _info: &SystemInfo) -> SystemParamIterInfo {
        SystemParamIterInfo::None
    }

    fn lock<'a>(data: &'a SystemData<'_>) -> <Self as SystemParamWithLifetime<'a>>::Guard {
        data
    }

    fn borrow_guard<'a, 'b>(
        guard: &'a mut <Self as SystemParamWithLifetime<'b>>::Guard,
    ) -> <Self as SystemParamWithLifetime<'a>>::GuardBorrow
    where
        'b: 'a,
    {
        guard
    }

    fn stream<'a, 'b>(
        guard: &'a mut <Self as SystemParamWithLifetime<'b>>::GuardBorrow,
        info: &'a SystemParamIterInfo,
    ) -> <Self as SystemParamWithLifetime<'a>>::Stream
    where
        'b: 'a,
    {
        WorldStream::new(info, guard)
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
    use crate::system_params::internal::SystemParamIterInfo;
    use crate::SystemData;
    use std::ops::Range;

    pub struct WorldStream<'a> {
        pub(crate) data: &'a SystemData<'a>,
        pub(crate) entity_positions: Range<usize>,
    }

    impl<'a> WorldStream<'a> {
        pub(crate) fn new(info: &'a SystemParamIterInfo, data: &'a SystemData<'a>) -> Self {
            Self {
                data,
                entity_positions: 0..info.item_count(),
            }
        }
    }
}

#[cfg(test)]
mod world_tests {
    use super::*;
    use crate::storages::archetypes::{ArchetypeStorage, EntityLocationInArchetype};
    use crate::storages::core::CoreStorage;
    use crate::storages::entities::EntityIdx;
    use crate::storages::entity_actions::EntityState;

    assert_impl_all!(World<'_>: Sync, Send, Unpin);

    #[test]
    fn delete_entity() {
        let core = CoreStorage::default();
        let data = core.system_data();
        let mut world = World { data: &data };

        world.delete_entity(2);

        let mut entity_actions = data.entity_actions.try_lock().unwrap();
        let states: Vec<_> = entity_actions.drain_entity_states().collect();
        assert_eq!(states.len(), 1);
        assert!(matches!(states[0], (EntityIdx(2), EntityState::Deleted)));
    }

    #[test]
    fn add_component() {
        let mut core = CoreStorage::default();
        let data = core.system_data();
        let mut world = World { data: &data };

        world.add_component(0, 10_u32);

        let mut states: Vec<_> = {
            let mut entity_actions = data.entity_actions.try_lock().unwrap();
            entity_actions.drain_entity_states().collect()
        };
        assert_eq!(states.len(), 1);
        let state = states.pop().unwrap();
        assert_eq!(state.0, 0.into());
        if let EntityState::Unchanged(mut add_fns, deleted_type_idxs) = state.1 {
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
        let mut world = World { data: &data };

        world.delete_component::<u32>(2);

        let mut entity_actions = data.entity_actions.try_lock().unwrap();
        let mut states: Vec<_> = entity_actions.drain_entity_states().collect();
        assert_eq!(states.len(), 1);
        let state = states.pop().unwrap();
        assert_eq!(state.0, 2.into());
        if let EntityState::Unchanged(add_fns, deleted_type_idxs) = state.1 {
            assert_eq!(add_fns.len(), 0);
            assert_eq!(deleted_type_idxs, &[1.into()]);
        } else {
            panic!("assertion failed: `states[0].1` matches `EntityState::Unchanged(_, _, _)`");
        }
    }
}

#[cfg(test)]
mod world_system_param_tests {
    use super::*;
    use crate::storages::core::CoreStorage;
    use std::ptr;

    #[test]
    fn retrieve_properties() {
        let mut core = CoreStorage::default();

        let properties = World::properties(&mut core);

        assert_eq!(properties.component_types.len(), 0);
        assert!(properties.has_entity_actions);
    }

    #[test]
    fn retrieve_iter_info() {
        let core = CoreStorage::default();
        let info = SystemInfo {
            filtered_component_types: vec![],
        };

        let iter_info = World::iter_info(&core.system_data(), &info);

        assert_eq!(iter_info, SystemParamIterInfo::None);
    }

    #[test]
    fn lock() {
        let core = CoreStorage::default();
        let data = core.system_data();

        let mut guard = World::lock(&data);
        let guard_borrow = World::borrow_guard(&mut guard);

        assert!(ptr::eq(guard_borrow, &data));
    }

    #[test]
    fn retrieve_stream_when_no_iteration() {
        let core = CoreStorage::default();
        let mut guard_borrow = &core.system_data();
        let iter_info = SystemParamIterInfo::None;

        let mut stream = World::stream(&mut guard_borrow, &iter_info);

        assert!(World::stream_next(&mut stream).is_some());
        assert!(World::stream_next(&mut stream).is_none());
    }

    #[test]
    fn retrieve_stream_when_iteration_on_entities() {
        let core = CoreStorage::default();
        let mut guard_borrow = &core.system_data();
        let iter_info = SystemParamIterInfo::new_union(vec![(0.into(), 1), (2.into(), 2)]);

        let mut stream = World::stream(&mut guard_borrow, &iter_info);

        assert!(World::stream_next(&mut stream).is_some());
        assert!(World::stream_next(&mut stream).is_some());
        assert!(World::stream_next(&mut stream).is_some());
        assert!(World::stream_next(&mut stream).is_none());
    }
}
