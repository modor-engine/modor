use crate::entity::internal::{EntityGuard, EntityGuardBorrow, EntityIter};
use crate::storages::archetypes::ArchetypeFilter;
use crate::storages::core::CoreStorage;
use crate::storages::entities::EntityIdx;
use crate::storages::systems::SystemProperties;
use crate::system_params::internal::{QuerySystemParamWithLifetime, SystemParamWithLifetime};
use crate::{QuerySystemParam, SystemData, SystemInfo, SystemParam};

/// A system parameter for retrieving information about the entity.
///
/// # Examples
///
/// ```rust
/// # use modor::Entity;
/// #
/// #[derive(Debug)]
/// struct Position(f32, f32);
///
/// fn print_position(position: &Position, entity: Entity<'_>) {
///     println!("Entity with ID {} has position {:?}", entity.id(), position)
/// }
/// ```
pub struct Entity<'a> {
    entity_idx: EntityIdx,
    #[allow(dead_code)] // will be used in the future
    data: SystemData<'a>,
}

impl<'a> Entity<'a> {
    /// Returns the entity ID.
    ///
    /// Entity IDs are unique and can be recycled in case the entity is deleted.
    pub fn id(&self) -> usize {
        self.entity_idx.into()
    }
}

impl<'a> SystemParamWithLifetime<'a> for Entity<'_> {
    type Param = Entity<'a>;
    type Guard = EntityGuard<'a>;
    type GuardBorrow = EntityGuardBorrow<'a>;
    type Stream = EntityIter<'a>;
}

impl SystemParam for Entity<'_> {
    type Tuple = (Self,);
    type InnerTuple = ();

    fn properties(_core: &mut CoreStorage) -> SystemProperties {
        SystemProperties {
            component_types: vec![],
            has_entity_actions: false,
            archetype_filter: ArchetypeFilter::All,
        }
    }

    fn lock<'a>(
        data: SystemData<'a>,
        info: SystemInfo<'a>,
    ) -> <Self as SystemParamWithLifetime<'a>>::Guard {
        EntityGuard::new(data, info)
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
        EntityIter::new(guard)
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

impl<'a> QuerySystemParamWithLifetime<'a> for Entity<'_> {
    type ConstParam = Entity<'a>;
    type Iter = EntityIter<'a>;
    type IterMut = EntityIter<'a>;
}

impl QuerySystemParam for Entity<'_> {
    fn query_iter<'a, 'b>(
        guard: &'a <Self as SystemParamWithLifetime<'b>>::GuardBorrow,
    ) -> <Self as QuerySystemParamWithLifetime<'a>>::Iter
    where
        'b: 'a,
    {
        EntityIter::new(guard)
    }

    fn query_iter_mut<'a, 'b>(
        guard: &'a mut <Self as SystemParamWithLifetime<'b>>::GuardBorrow,
    ) -> <Self as QuerySystemParamWithLifetime<'a>>::IterMut
    where
        'b: 'a,
    {
        EntityIter::new(guard)
    }
}

mod internal {
    use crate::storages::archetypes::FilteredArchetypeIdxIter;
    use crate::storages::entities::EntityIdx;
    use crate::{Entity, SystemData, SystemInfo};
    use std::iter::Flatten;
    use std::slice::Iter;

    pub struct EntityGuard<'a> {
        data: SystemData<'a>,
        info: SystemInfo<'a>,
    }

    impl<'a> EntityGuard<'a> {
        pub(crate) fn new(data: SystemData<'a>, info: SystemInfo<'a>) -> Self {
            Self { data, info }
        }

        pub(crate) fn borrow(&mut self) -> EntityGuardBorrow<'_> {
            EntityGuardBorrow {
                item_count: self.data.item_count(self.info),
                sorted_archetype_idxs: self.data.filter_archetype_idx_iter(self.info),
                data: self.data,
            }
        }
    }

    pub struct EntityGuardBorrow<'a> {
        pub(crate) item_count: usize,
        pub(crate) sorted_archetype_idxs: FilteredArchetypeIdxIter<'a>,
        pub(crate) data: SystemData<'a>,
    }

    pub struct EntityIter<'a> {
        entity_idxs: Flatten<ArchetypeEntityIdxIter<'a>>,
        len: usize,
        data: SystemData<'a>,
    }

    impl<'a> EntityIter<'a> {
        pub fn new(guard: &'a EntityGuardBorrow<'_>) -> Self {
            Self {
                entity_idxs: ArchetypeEntityIdxIter::new(guard).flatten(),
                len: guard.item_count,
                data: guard.data,
            }
        }
    }

    impl<'a> Iterator for EntityIter<'a> {
        type Item = Entity<'a>;

        #[inline]
        fn next(&mut self) -> Option<Self::Item> {
            self.entity_idxs.next().map(|&e| {
                self.len -= 1;
                Entity {
                    entity_idx: e,
                    data: self.data,
                }
            })
        }

        fn size_hint(&self) -> (usize, Option<usize>) {
            (self.len, Some(self.len))
        }
    }

    impl DoubleEndedIterator for EntityIter<'_> {
        #[inline]
        fn next_back(&mut self) -> Option<Self::Item> {
            self.entity_idxs.next_back().map(|&e| {
                self.len -= 1;
                Entity {
                    entity_idx: e,
                    data: self.data,
                }
            })
        }
    }

    impl ExactSizeIterator for EntityIter<'_> {}

    struct ArchetypeEntityIdxIter<'a> {
        sorted_archetype_idxs: FilteredArchetypeIdxIter<'a>,
        data: SystemData<'a>,
    }

    impl<'a> ArchetypeEntityIdxIter<'a> {
        fn new(guard: &'a EntityGuardBorrow<'_>) -> Self {
            Self {
                sorted_archetype_idxs: guard.sorted_archetype_idxs.clone(),
                data: guard.data,
            }
        }
    }

    impl<'a> Iterator for ArchetypeEntityIdxIter<'a> {
        type Item = Iter<'a, EntityIdx>;

        fn next(&mut self) -> Option<Self::Item> {
            self.sorted_archetype_idxs
                .next()
                .map(|a| self.data.archetypes.entity_idxs(a).iter())
        }
    }

    impl DoubleEndedIterator for ArchetypeEntityIdxIter<'_> {
        fn next_back(&mut self) -> Option<Self::Item> {
            self.sorted_archetype_idxs
                .next_back()
                .map(|a| self.data.archetypes.entity_idxs(a).iter())
        }
    }
}

#[cfg(test)]
mod entity_tests {
    use super::*;
    use crate::storages::core::CoreStorage;

    assert_impl_all!(Entity<'_>: Sync, Send, Unpin);

    #[test]
    fn retrieve_id() {
        let core = CoreStorage::default();
        let entity = Entity {
            entity_idx: 2.into(),
            data: core.system_data(),
        };

        let id = entity.id();

        assert_eq!(id, 2);
    }
}

#[cfg(test)]
mod entity_system_param_tests {
    use super::*;
    use crate::storages::archetypes::{ArchetypeStorage, FilteredArchetypeIdxIter};
    use crate::storages::core::CoreStorage;
    use crate::{QuerySystemParam, SystemInfo, SystemParam};
    use std::any::Any;

    #[test]
    fn retrieve_properties() {
        let mut core = CoreStorage::default();

        let properties = Entity::properties(&mut core);

        assert_eq!(properties.component_types.len(), 0);
        assert!(!properties.has_entity_actions);
        assert_eq!(properties.archetype_filter, ArchetypeFilter::All);
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
        };

        let mut guard = Entity::lock(data, info);
        let mut guard_borrow = Entity::borrow_guard(&mut guard);

        assert_eq!(guard_borrow.item_count, 1);
        let next_archetype_idx = guard_borrow.sorted_archetype_idxs.next();
        assert_eq!(next_archetype_idx, Some(archetype2_idx));
        assert_eq!(guard_borrow.sorted_archetype_idxs.next(), None);
    }

    #[test]
    fn retrieve_stream() {
        let mut core = CoreStorage::default();
        create_entity(&mut core, 0_i16);
        create_entity(&mut core, 10_i64);
        create_entity(&mut core, 20_u32);
        create_entity(&mut core, 30_u16);
        create_entity(&mut core, 40_u16);
        create_entity(&mut core, 50_i8);
        let archetype_idxs = [2.into(), 4.into()];
        let archetype_type_idxs = ti_vec![vec![0.into()]; 6];
        let mut guard_borrow = EntityGuardBorrow {
            item_count: 3,
            sorted_archetype_idxs: FilteredArchetypeIdxIter::new(
                &archetype_idxs,
                &archetype_type_idxs,
            ),
            data: core.system_data(),
        };

        let mut stream = Entity::stream(&mut guard_borrow);

        assert_eq!(Entity::stream_next(&mut stream).unwrap().id(), 1);
        assert_eq!(Entity::stream_next(&mut stream).unwrap().id(), 3);
        assert_eq!(Entity::stream_next(&mut stream).unwrap().id(), 4);
        assert!(Entity::stream_next(&mut stream).is_none());
    }

    #[test]
    fn retrieve_query_iter() {
        let mut core = CoreStorage::default();
        create_entity(&mut core, 0_i16);
        create_entity(&mut core, 10_i64);
        create_entity(&mut core, 20_u32);
        create_entity(&mut core, 30_u16);
        create_entity(&mut core, 40_u16);
        create_entity(&mut core, 50_i8);
        let archetype_idxs = [2.into(), 4.into()];
        let archetype_type_idxs = ti_vec![vec![0.into()]; 6];
        let guard_borrow = EntityGuardBorrow {
            item_count: 3,
            sorted_archetype_idxs: FilteredArchetypeIdxIter::new(
                &archetype_idxs,
                &archetype_type_idxs,
            ),
            data: core.system_data(),
        };

        let mut iter = Entity::query_iter(&guard_borrow);

        assert_eq!(iter.len(), 3);
        assert_eq!(iter.next().unwrap().id(), 1);
        assert_eq!(iter.len(), 2);
        assert_eq!(iter.next().unwrap().id(), 3);
        assert_eq!(iter.len(), 1);
        assert_eq!(iter.next().unwrap().id(), 4);
        assert_eq!(iter.len(), 0);
        assert!(iter.next().is_none());
    }

    #[test]
    fn retrieve_reversed_query_iter() {
        let mut core = CoreStorage::default();
        create_entity(&mut core, 0_i16);
        create_entity(&mut core, 10_i64);
        create_entity(&mut core, 20_u32);
        create_entity(&mut core, 30_u16);
        create_entity(&mut core, 40_u16);
        create_entity(&mut core, 50_i8);
        let archetype_idxs = [2.into(), 4.into()];
        let archetype_type_idxs = ti_vec![vec![0.into()]; 6];
        let guard_borrow = EntityGuardBorrow {
            item_count: 3,
            sorted_archetype_idxs: FilteredArchetypeIdxIter::new(
                &archetype_idxs,
                &archetype_type_idxs,
            ),
            data: core.system_data(),
        };

        let mut iter = Entity::query_iter(&guard_borrow).rev();

        assert_eq!(iter.len(), 3);
        assert_eq!(iter.next().unwrap().id(), 4);
        assert_eq!(iter.len(), 2);
        assert_eq!(iter.next().unwrap().id(), 3);
        assert_eq!(iter.len(), 1);
        assert_eq!(iter.next().unwrap().id(), 1);
        assert_eq!(iter.len(), 0);
        assert!(iter.next().is_none());
    }

    #[test]
    fn retrieve_query_iter_mut() {
        let mut core = CoreStorage::default();
        create_entity(&mut core, 0_i16);
        create_entity(&mut core, 10_i64);
        create_entity(&mut core, 20_u32);
        create_entity(&mut core, 30_u16);
        create_entity(&mut core, 40_u16);
        create_entity(&mut core, 50_i8);
        let archetype_idxs = [2.into(), 4.into()];
        let archetype_type_idxs = ti_vec![vec![0.into()]; 6];
        let mut guard_borrow = EntityGuardBorrow {
            item_count: 3,
            sorted_archetype_idxs: FilteredArchetypeIdxIter::new(
                &archetype_idxs,
                &archetype_type_idxs,
            ),
            data: core.system_data(),
        };

        let mut iter = Entity::query_iter_mut(&mut guard_borrow);

        assert_eq!(iter.len(), 3);
        assert_eq!(iter.next().unwrap().id(), 1);
        assert_eq!(iter.len(), 2);
        assert_eq!(iter.next().unwrap().id(), 3);
        assert_eq!(iter.len(), 1);
        assert_eq!(iter.next().unwrap().id(), 4);
        assert_eq!(iter.len(), 0);
        assert!(iter.next().is_none());
    }

    #[test]
    fn retrieve_reversed_query_iter_mut() {
        let mut core = CoreStorage::default();
        create_entity(&mut core, 0_i16);
        create_entity(&mut core, 10_i64);
        create_entity(&mut core, 20_u32);
        create_entity(&mut core, 30_u16);
        create_entity(&mut core, 40_u16);
        create_entity(&mut core, 50_i8);
        let archetype_idxs = [2.into(), 4.into()];
        let archetype_type_idxs = ti_vec![vec![0.into()]; 6];
        let mut guard_borrow = EntityGuardBorrow {
            item_count: 3,
            sorted_archetype_idxs: FilteredArchetypeIdxIter::new(
                &archetype_idxs,
                &archetype_type_idxs,
            ),
            data: core.system_data(),
        };

        let mut iter = Entity::query_iter_mut(&mut guard_borrow).rev();

        assert_eq!(iter.len(), 3);
        assert_eq!(iter.next().unwrap().id(), 4);
        assert_eq!(iter.len(), 2);
        assert_eq!(iter.next().unwrap().id(), 3);
        assert_eq!(iter.len(), 1);
        assert_eq!(iter.next().unwrap().id(), 1);
        assert_eq!(iter.len(), 0);
        assert!(iter.next().is_none());
    }

    fn create_entity<C>(core: &mut CoreStorage, component: C)
    where
        C: Any + Sync + Send,
    {
        let archetype1_idx = ArchetypeStorage::DEFAULT_IDX;
        let (type_idx, archetype2_idx) = core.add_component_type::<C>(archetype1_idx);
        let location = core.create_entity(archetype2_idx);
        core.add_component(component, type_idx, location);
    }
}
