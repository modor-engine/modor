use crate::optional_components::internal::{ComponentOptionGuard, ComponentOptionGuardBorrow};
use crate::storages::archetypes::{ArchetypeFilter, EntityLocation};
use crate::storages::core::CoreStorage;
use crate::storages::systems::{Access, ComponentTypeAccess, SystemProperties};
use crate::system_params::internal::{
    Const, LockableSystemParam, QuerySystemParamWithLifetime, SystemParamWithLifetime,
};
use crate::system_params::optional_components::internal::ComponentOptionIter;
use crate::{QuerySystemParam, SystemData, SystemInfo, SystemParam};
use std::any::Any;

impl<'a, C> SystemParamWithLifetime<'a> for Option<&C>
where
    C: Any + Sync + Send,
{
    type Param = Option<&'a C>;
    type Guard = ComponentOptionGuard<'a, C>;
    type GuardBorrow = ComponentOptionGuardBorrow<'a, C>;
    type Stream = ComponentOptionIter<'a, C>;
}

impl<C> SystemParam for Option<&C>
where
    C: Any + Sync + Send,
{
    type Tuple = (Self,);
    type InnerTuple = ();

    fn properties(core: &mut CoreStorage) -> SystemProperties {
        let type_idx = core.register_component_type::<C>();
        SystemProperties {
            component_types: vec![ComponentTypeAccess {
                access: Access::Read,
                type_idx,
            }],
            can_update: false,
            archetype_filter: ArchetypeFilter::Union(ne_vec![type_idx]),
        }
    }

    fn lock<'a>(
        data: SystemData<'a>,
        info: SystemInfo<'a>,
    ) -> <Self as SystemParamWithLifetime<'a>>::Guard {
        ComponentOptionGuard::new(data, info)
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
        ComponentOptionIter::new(guard)
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

impl<'a, C> QuerySystemParamWithLifetime<'a> for Option<&C>
where
    C: Any + Sync + Send,
{
    type ConstParam = Option<&'a C>;
    type Iter = ComponentOptionIter<'a, C>;
    type IterMut = ComponentOptionIter<'a, C>;
}

impl<C> QuerySystemParam for Option<&C>
where
    C: Any + Sync + Send,
{
    fn query_iter<'a, 'b>(
        guard: &'a <Self as SystemParamWithLifetime<'b>>::GuardBorrow,
    ) -> <Self as QuerySystemParamWithLifetime<'a>>::Iter
    where
        'b: 'a,
    {
        ComponentOptionIter::new(guard)
    }

    fn query_iter_mut<'a, 'b>(
        guard: &'a mut <Self as SystemParamWithLifetime<'b>>::GuardBorrow,
    ) -> <Self as QuerySystemParamWithLifetime<'a>>::IterMut
    where
        'b: 'a,
    {
        ComponentOptionIter::new(guard)
    }

    #[inline]
    fn get<'a, 'b>(
        guard: &'a <Self as SystemParamWithLifetime<'b>>::GuardBorrow,
        location: EntityLocation,
    ) -> Option<<Self as QuerySystemParamWithLifetime<'a>>::ConstParam>
    where
        'b: 'a,
    {
        Some(
            guard
                .components
                .get(location.idx)
                .and_then(|a| a.get(location.pos)),
        )
    }

    #[inline]
    fn get_mut<'a, 'b>(
        guard: &'a mut <Self as SystemParamWithLifetime<'b>>::GuardBorrow,
        location: EntityLocation,
    ) -> Option<<Self as SystemParamWithLifetime<'a>>::Param>
    where
        'b: 'a,
    {
        Some(
            guard
                .components
                .get(location.idx)
                .and_then(|a| a.get(location.pos)),
        )
    }
}

impl<C> LockableSystemParam for Option<&C>
where
    C: Any + Sync + Send,
{
    type LockedType = C;
    type Mutability = Const;
}

pub(crate) mod internal {
    use crate::optional_components_mut::internal::ComponentMutOptionGuardBorrow;
    use crate::storages::archetypes::{ArchetypeEntityPos, ArchetypeIdx, FilteredArchetypeIdxIter};
    use crate::storages::components::ComponentArchetypes;
    use crate::{SystemData, SystemInfo};
    use std::any::Any;
    use std::iter::Flatten;
    use std::ops::Range;
    use std::slice::Iter;
    use std::sync::RwLockReadGuard;
    use typed_index_collections::TiVec;

    pub struct ComponentOptionGuard<'a, C> {
        components: RwLockReadGuard<'a, ComponentArchetypes<C>>,
        data: SystemData<'a>,
        info: SystemInfo<'a>,
    }

    impl<'a, C> ComponentOptionGuard<'a, C>
    where
        C: Any,
    {
        pub(crate) fn new(data: SystemData<'a>, info: SystemInfo<'a>) -> Self {
            Self {
                components: data.components.read_components::<C>(),
                data,
                info,
            }
        }

        pub(crate) fn borrow(&mut self) -> ComponentOptionGuardBorrow<'_, C> {
            ComponentOptionGuardBorrow {
                components: &*self.components,
                item_count: self.info.item_count,
                sorted_archetype_idxs: self.data.filter_archetype_idx_iter(
                    self.info.filtered_component_type_idxs,
                    self.info.archetype_filter,
                ),
                data: self.data,
            }
        }
    }

    pub struct ComponentOptionGuardBorrow<'a, C> {
        pub(crate) components: &'a ComponentArchetypes<C>,
        pub(crate) item_count: usize,
        pub(crate) sorted_archetype_idxs: FilteredArchetypeIdxIter<'a>,
        pub(crate) data: SystemData<'a>,
    }

    pub struct ComponentOptionIter<'a, C> {
        components: Flatten<ArchetypeComponentIter<'a, C>>,
        len: usize,
    }

    impl<'a, C> ComponentOptionIter<'a, C> {
        pub(crate) fn new(guard: &'a ComponentOptionGuardBorrow<'_, C>) -> Self {
            Self {
                components: ArchetypeComponentIter::new(guard).flatten(),
                len: guard.item_count,
            }
        }

        pub(crate) fn new_mut(guard: &'a ComponentMutOptionGuardBorrow<'_, C>) -> Self {
            Self {
                components: ArchetypeComponentIter::new_mut(guard).flatten(),
                len: guard.item_count,
            }
        }
    }

    impl<'a, C> Iterator for ComponentOptionIter<'a, C> {
        type Item = Option<&'a C>;

        #[inline]
        fn next(&mut self) -> Option<Self::Item> {
            self.components.next().map(|c| {
                self.len -= 1;
                c
            })
        }

        fn size_hint(&self) -> (usize, Option<usize>) {
            (self.len, Some(self.len))
        }
    }

    impl<'a, C> DoubleEndedIterator for ComponentOptionIter<'a, C> {
        #[inline]
        fn next_back(&mut self) -> Option<Self::Item> {
            self.components.next_back().map(|c| {
                self.len -= 1;
                c
            })
        }
    }

    impl<'a, C> ExactSizeIterator for ComponentOptionIter<'a, C> {}

    struct ArchetypeComponentIter<'a, C> {
        last_archetype_idx: Option<ArchetypeIdx>,
        components: Iter<'a, TiVec<ArchetypeEntityPos, C>>,
        sorted_archetype_idxs: FilteredArchetypeIdxIter<'a>,
        data: SystemData<'a>,
    }

    impl<'a, C> ArchetypeComponentIter<'a, C> {
        fn new(guard: &'a ComponentOptionGuardBorrow<'_, C>) -> Self {
            Self {
                last_archetype_idx: None,
                components: guard.components.iter(),
                sorted_archetype_idxs: guard.sorted_archetype_idxs.clone(),
                data: guard.data,
            }
        }

        fn new_mut(guard: &'a ComponentMutOptionGuardBorrow<'_, C>) -> Self {
            Self {
                last_archetype_idx: None,
                components: guard.components.iter(),
                sorted_archetype_idxs: guard.sorted_archetype_idxs.clone(),
                data: guard.data,
            }
        }
    }

    impl<'a, C> Iterator for ArchetypeComponentIter<'a, C> {
        type Item = ComponentIter<'a, C>;

        fn next(&mut self) -> Option<Self::Item> {
            let archetype_idx = self.sorted_archetype_idxs.next()?;
            let nth = usize::from(archetype_idx)
                - self.last_archetype_idx.map_or(0, |i| usize::from(i) + 1);
            self.last_archetype_idx = Some(archetype_idx);
            Some(ComponentIter::new(
                self.components.nth(nth).map(|c| c.iter()),
                self.data.archetypes.entity_idxs(archetype_idx).len(),
            ))
        }
    }

    impl<'a, C> DoubleEndedIterator for ArchetypeComponentIter<'a, C> {
        fn next_back(&mut self) -> Option<Self::Item> {
            let archetype_idx = self.sorted_archetype_idxs.next_back()?;
            let nth_back = self.components.len() - usize::from(archetype_idx) - 1;
            Some(ComponentIter::new(
                self.components.nth_back(nth_back).map(|c| c.iter()),
                self.data.archetypes.entity_idxs(archetype_idx).len(),
            ))
        }
    }

    struct ComponentIter<'a, C> {
        components: Option<Iter<'a, C>>,
        entity_positions: Range<usize>,
    }

    impl<'a, C> ComponentIter<'a, C> {
        fn new(components: Option<Iter<'a, C>>, entity_count: usize) -> Self {
            Self {
                components,
                entity_positions: 0..entity_count,
            }
        }
    }

    impl<'a, C> Iterator for ComponentIter<'a, C> {
        type Item = Option<&'a C>;

        #[inline]
        fn next(&mut self) -> Option<Self::Item> {
            self.entity_positions.next().map(|_| {
                self.components
                    .as_mut()
                    .expect("internal error: missing component during iteration")
                    .next()
            })
        }
    }

    impl<'a, C> DoubleEndedIterator for ComponentIter<'a, C> {
        #[inline]
        fn next_back(&mut self) -> Option<Self::Item> {
            self.entity_positions.next().map(|_| {
                self.components
                    .as_mut()
                    .expect("internal error: missing component during reversed iteration")
                    .next_back()
            })
        }
    }
}

#[cfg(test)]
mod component_ref_option_tests {
    use crate::storages::archetypes::ArchetypeFilter;
    use crate::storages::core::CoreStorage;
    use crate::storages::systems::Access;
    use crate::utils::test_utils::assert_iter;
    use crate::{QuerySystemParam, SystemInfo, SystemParam};
    use std::any::TypeId;

    #[test]
    fn retrieve_system_param_properties() {
        let mut core = CoreStorage::default();

        let properties = Option::<&u32>::properties(&mut core);

        assert_eq!(properties.component_types.len(), 1);
        assert_eq!(properties.component_types[0].access, Access::Read);
        assert_eq!(properties.component_types[0].type_idx, 0.into());
        assert!(!properties.can_update);
        let archetype_filter = ArchetypeFilter::Union(ne_vec![0.into()]);
        assert_eq!(properties.archetype_filter, archetype_filter);
    }

    #[test]
    fn use_system_param() {
        let mut core = CoreStorage::default();
        let location1 = core.create_entity_with_1_component(0_i8);
        let location2 = core.create_entity_with_2_components(20_u32, 0_i16);
        let location3 = core.create_entity_with_1_component(0_i16);
        core.create_entity_with_3_components(40_u32, 0_i16, 0_i64);
        core.create_entity_with_3_components(50_u32, 0_i16, 0_i64);
        core.create_entity_with_2_components(60_u32, 0_i128);
        let filtered_type_idx = core.components().type_idx(TypeId::of::<i16>()).unwrap();
        let info = SystemInfo {
            filtered_component_type_idxs: &[filtered_type_idx],
            archetype_filter: &ArchetypeFilter::All,
            item_count: 4,
        };
        let mut guard = Option::<&u32>::lock(core.system_data(), info);
        let mut borrow = Option::<&u32>::borrow_guard(&mut guard);

        let mut stream = Option::<&u32>::stream(&mut borrow);
        assert_eq!(Option::<&u32>::stream_next(&mut stream), Some(Some(&20)));
        assert_eq!(Option::<&u32>::stream_next(&mut stream), Some(None));
        assert_eq!(Option::<&u32>::stream_next(&mut stream), Some(Some(&40)));
        assert_eq!(Option::<&u32>::stream_next(&mut stream), Some(Some(&50)));
        assert_eq!(Option::<&u32>::stream_next(&mut stream), None);

        let iter = Option::<&u32>::query_iter(&borrow);
        assert_iter(iter, [Some(&20), None, Some(&40), Some(&50)]);
        let iter = Option::<&u32>::query_iter(&borrow).rev();
        assert_iter(iter, [Some(&50), Some(&40), None, Some(&20)]);
        let iter = Option::<&u32>::query_iter_mut(&mut borrow);
        assert_iter(iter, [Some(&20), None, Some(&40), Some(&50)]);
        let iter = Option::<&u32>::query_iter_mut(&mut borrow).rev();
        assert_iter(iter, [Some(&50), Some(&40), None, Some(&20)]);
        assert_eq!(Option::<&u32>::get(&borrow, location1), Some(None));
        assert_eq!(Option::<&u32>::get_mut(&mut borrow, location1), Some(None));
        assert_eq!(Option::<&u32>::get(&borrow, location2), Some(Some(&20)));
        let item = Option::<&u32>::get_mut(&mut borrow, location2);
        assert_eq!(item, Some(Some(&20)));
        assert_eq!(Option::<&u32>::get(&borrow, location3), Some(None));
        assert_eq!(Option::<&u32>::get_mut(&mut borrow, location3), Some(None));
    }
}
