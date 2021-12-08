use crate::entity::internal::EntityIter;
use crate::storages::core::SystemProperties;
use crate::storages::entities::EntityIdx;
use crate::system_params::internal::{
    EntityIterInfo, QuerySystemParamWithLifetime, SystemParamIterInfo, SystemParamWithLifetime,
};
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
    data: &'a SystemData<'a>,
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
    type Guard = &'a SystemData<'a>;
    type GuardBorrow = &'a SystemData<'a>;
    type Stream = EntityIter<'a>;
}

impl SystemParam for Entity<'_> {
    type Tuple = (Self,);
    type InnerTuple = ();

    fn properties() -> SystemProperties {
        SystemProperties {
            component_types: vec![],
            has_entity_actions: false,
        }
    }

    fn iter_info(data: &SystemData<'_>, info: &SystemInfo) -> SystemParamIterInfo {
        SystemParamIterInfo::ComponentUnionEntities(EntityIterInfo {
            sorted_archetypes: if info.filtered_component_types.is_empty() {
                data.archetypes.all_sorted()
            } else {
                data.components
                    .type_idxs(&info.filtered_component_types)
                    .map_or_else(Vec::new, |i| data.archetypes.sorted_with_all_types(&i))
            },
        })
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
        EntityIter::new(info, guard)
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
        info: &'a SystemParamIterInfo,
    ) -> <Self as QuerySystemParamWithLifetime<'a>>::Iter
    where
        'b: 'a,
    {
        EntityIter::new(info, guard)
    }

    fn query_iter_mut<'a, 'b>(
        guard: &'a mut <Self as SystemParamWithLifetime<'b>>::GuardBorrow,
        info: &'a SystemParamIterInfo,
    ) -> <Self as QuerySystemParamWithLifetime<'a>>::IterMut
    where
        'b: 'a,
    {
        EntityIter::new(info, guard)
    }
}

mod internal {
    use crate::storages::archetypes::ArchetypeInfo;
    use crate::storages::entities::EntityIdx;
    use crate::system_params::internal::SystemParamIterInfo;
    use crate::{Entity, SystemData};
    use std::iter::Flatten;
    use std::slice::Iter;

    pub struct EntityIter<'a> {
        entity_idxs: Flatten<ArchetypeEntityIdxIter<'a>>,
        len: usize,
        data: &'a SystemData<'a>,
    }

    impl<'a> EntityIter<'a> {
        pub fn new(info: &'a SystemParamIterInfo, data: &'a SystemData<'a>) -> Self {
            Self {
                entity_idxs: ArchetypeEntityIdxIter {
                    sorted_archetypes: info
                        .sorted_archetypes()
                        .expect("internal error: wrong iter mode for components")
                        .iter(),
                    data,
                }
                .flatten(),
                len: info.item_count(),
                data,
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
        sorted_archetypes: Iter<'a, ArchetypeInfo>,
        data: &'a SystemData<'a>,
    }

    impl<'a> Iterator for ArchetypeEntityIdxIter<'a> {
        type Item = Iter<'a, EntityIdx>;

        fn next(&mut self) -> Option<Self::Item> {
            self.sorted_archetypes
                .next()
                .map(|a| self.data.archetypes.entity_idxs(a.idx).iter())
        }
    }

    impl DoubleEndedIterator for ArchetypeEntityIdxIter<'_> {
        fn next_back(&mut self) -> Option<Self::Item> {
            self.sorted_archetypes
                .next_back()
                .map(|a| self.data.archetypes.entity_idxs(a.idx).iter())
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
            data: &core.system_data(),
        };

        let id = entity.id();

        assert_eq!(id, 2);
    }
}

#[cfg(test)]
mod entity_system_param_tests {
    use super::*;
    use crate::storages::archetypes::ArchetypeStorage;
    use crate::storages::core::CoreStorage;
    use crate::system_params::internal::SystemParamIterInfo;
    use crate::{QuerySystemParam, SystemInfo, SystemParam};
    use std::any::Any;
    use std::ptr;

    #[test]
    fn retrieve_properties() {
        let properties = Entity::properties();

        assert_eq!(properties.component_types.len(), 0);
        assert!(!properties.has_entity_actions);
    }

    #[test]
    fn retrieve_iter_info_from_no_filter_type() {
        let mut core = CoreStorage::default();
        let (_, archetype_idx) = core.add_component_type::<i64>(ArchetypeStorage::DEFAULT_IDX);
        let info = SystemInfo {
            filtered_component_types: vec![],
        };

        let iter_info = Entity::iter_info(&core.system_data(), &info);

        let archetypes = vec![(ArchetypeStorage::DEFAULT_IDX, 0), (archetype_idx, 0)];
        assert_eq!(iter_info, SystemParamIterInfo::new_union(archetypes));
    }

    #[test]
    fn retrieve_iter_info_from_missing_filter_type() {
        let mut core = CoreStorage::default();
        core.add_component_type::<u32>(ArchetypeStorage::DEFAULT_IDX);
        let info = SystemInfo::with_one_filtered_type::<i64>();

        let iter_info = Entity::iter_info(&core.system_data(), &info);

        assert_eq!(iter_info, SystemParamIterInfo::new_union(vec![]));
    }

    #[test]
    fn retrieve_iter_info_from_existing_filter_type() {
        let mut core = CoreStorage::default();
        let (_, archetype1_idx) = core.add_component_type::<u32>(ArchetypeStorage::DEFAULT_IDX);
        let (_, archetype2_idx) = core.add_component_type::<i64>(archetype1_idx);
        let info = SystemInfo::with_one_filtered_type::<i64>();

        let iter_info = Entity::iter_info(&core.system_data(), &info);

        let expected_iter_info = SystemParamIterInfo::new_union(vec![(archetype2_idx, 0)]);
        assert_eq!(iter_info, expected_iter_info);
    }

    #[test]
    fn lock() {
        let core = CoreStorage::default();
        let data = core.system_data();

        let mut guard = Entity::lock(&data);
        let guard_borrow = Entity::borrow_guard(&mut guard);

        assert!(ptr::eq(guard_borrow, &data));
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
        let mut guard_borrow = &core.system_data();
        let iter_info = SystemParamIterInfo::new_union(vec![(2.into(), 1), (4.into(), 2)]);

        let mut stream = Entity::stream(&mut guard_borrow, &iter_info);

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
        let mut guard_borrow = &core.system_data();
        let iter_info = SystemParamIterInfo::new_union(vec![(2.into(), 1), (4.into(), 2)]);

        let iter = Entity::query_iter(&mut guard_borrow, &iter_info);

        assert_iter!(iter.map(|e| e.id()), [1, 3, 4]);
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
        let mut guard_borrow = &core.system_data();
        let iter_info = SystemParamIterInfo::new_union(vec![(2.into(), 1), (4.into(), 2)]);

        let iter = Entity::query_iter(&mut guard_borrow, &iter_info).rev();

        assert_iter!(iter.map(|e| e.id()), [4, 3, 1]);
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
        let mut guard_borrow = &core.system_data();
        let iter_info = SystemParamIterInfo::new_union(vec![(2.into(), 1), (4.into(), 2)]);

        let iter = Entity::query_iter_mut(&mut guard_borrow, &iter_info);

        assert_iter!(iter.map(|e| e.id()), [1, 3, 4]);
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
        let mut guard_borrow = &core.system_data();
        let iter_info = SystemParamIterInfo::new_union(vec![(2.into(), 1), (4.into(), 2)]);

        let iter = Entity::query_iter_mut(&mut guard_borrow, &iter_info).rev();

        assert_iter!(iter.map(|e| e.id()), [4, 3, 1]);
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
