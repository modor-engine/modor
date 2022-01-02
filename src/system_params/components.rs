use crate::components::internal::{ComponentGuard, ComponentGuardBorrow, ComponentIter};
use crate::storages::archetypes::ArchetypeFilter;
use crate::storages::core::CoreStorage;
use crate::storages::systems::{Access, ComponentTypeAccess, SystemProperties};
use crate::system_params::internal::{
    Const, LockableSystemParam, QuerySystemParamWithLifetime, SystemParamWithLifetime,
};
use crate::{QuerySystemParam, SystemData, SystemInfo, SystemParam};
use std::any::Any;

impl<'a, C> SystemParamWithLifetime<'a> for &C
where
    C: Any + Sync + Send,
{
    type Param = &'a C;
    type Guard = ComponentGuard<'a, C>;
    type GuardBorrow = ComponentGuardBorrow<'a, C>;
    type Stream = ComponentIter<'a, C>;
}

impl<C> SystemParam for &C
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
            has_entity_actions: false,
            archetype_filter: ArchetypeFilter::Intersection(ne_vec![type_idx]),
        }
    }

    fn lock<'a>(
        data: SystemData<'a>,
        info: SystemInfo<'a>,
    ) -> <Self as SystemParamWithLifetime<'a>>::Guard {
        ComponentGuard::new(data, info)
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
        ComponentIter::new(guard)
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

impl<'a, C> QuerySystemParamWithLifetime<'a> for &C
where
    C: Any + Sync + Send,
{
    type ConstParam = &'a C;
    type Iter = ComponentIter<'a, C>;
    type IterMut = ComponentIter<'a, C>;
}

impl<C> QuerySystemParam for &C
where
    C: Any + Sync + Send,
{
    fn query_iter<'a, 'b>(
        guard: &'a <Self as SystemParamWithLifetime<'b>>::GuardBorrow,
    ) -> <Self as QuerySystemParamWithLifetime<'a>>::Iter
    where
        'b: 'a,
    {
        ComponentIter::new(guard)
    }

    fn query_iter_mut<'a, 'b>(
        guard: &'a mut <Self as SystemParamWithLifetime<'b>>::GuardBorrow,
    ) -> <Self as QuerySystemParamWithLifetime<'a>>::IterMut
    where
        'b: 'a,
    {
        ComponentIter::new(guard)
    }
}

impl<C> LockableSystemParam for &C
where
    C: Any + Sync + Send,
{
    type LockedType = C;
    type Mutability = Const;
}

pub(crate) mod internal {
    use crate::components_mut::internal::ComponentMutGuardBorrow;
    use crate::storages::archetypes::{ArchetypeEntityPos, ArchetypeIdx, FilteredArchetypeIdxIter};
    use crate::storages::components::ComponentArchetypes;
    use crate::{SystemData, SystemInfo};
    use std::any::Any;
    use std::iter::Flatten;
    use std::slice::Iter;
    use std::sync::RwLockReadGuard;
    use typed_index_collections::TiVec;

    pub struct ComponentGuard<'a, C> {
        components: RwLockReadGuard<'a, ComponentArchetypes<C>>,
        data: SystemData<'a>,
        info: SystemInfo<'a>,
    }

    impl<'a, C> ComponentGuard<'a, C>
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

        pub(crate) fn borrow(&mut self) -> ComponentGuardBorrow<'_, C> {
            ComponentGuardBorrow {
                components: &*self.components,
                item_count: self.info.item_count,
                sorted_archetype_idxs: self.data.filter_archetype_idx_iter(
                    self.info.filtered_component_type_idxs,
                    self.info.archetype_filter,
                ),
            }
        }
    }

    pub struct ComponentGuardBorrow<'a, C> {
        pub(crate) components: &'a ComponentArchetypes<C>,
        pub(crate) item_count: usize,
        pub(crate) sorted_archetype_idxs: FilteredArchetypeIdxIter<'a>,
    }

    pub struct ComponentIter<'a, C> {
        components: Flatten<ArchetypeComponentIter<'a, C>>,
        len: usize,
    }

    impl<'a, C> ComponentIter<'a, C> {
        pub(crate) fn new(guard: &'a ComponentGuardBorrow<'_, C>) -> Self {
            Self {
                components: ArchetypeComponentIter::new(guard).flatten(),
                len: guard.item_count,
            }
        }

        pub(crate) fn new_mut(guard: &'a ComponentMutGuardBorrow<'_, C>) -> Self {
            Self {
                components: ArchetypeComponentIter::new_mut(guard).flatten(),
                len: guard.item_count,
            }
        }
    }

    impl<'a, C> Iterator for ComponentIter<'a, C> {
        type Item = &'a C;

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

    impl<'a, C> DoubleEndedIterator for ComponentIter<'a, C> {
        #[inline]
        fn next_back(&mut self) -> Option<Self::Item> {
            self.components.next_back().map(|c| {
                self.len -= 1;
                c
            })
        }
    }

    impl<'a, C> ExactSizeIterator for ComponentIter<'a, C> {}

    struct ArchetypeComponentIter<'a, C> {
        last_archetype_idx: Option<ArchetypeIdx>,
        components: Iter<'a, TiVec<ArchetypeEntityPos, C>>,
        sorted_archetype_idxs: FilteredArchetypeIdxIter<'a>,
    }

    impl<'a, C> ArchetypeComponentIter<'a, C> {
        fn new(guard: &'a ComponentGuardBorrow<'_, C>) -> Self {
            Self {
                last_archetype_idx: None,
                components: guard.components.iter(),
                sorted_archetype_idxs: guard.sorted_archetype_idxs.clone(),
            }
        }

        fn new_mut(guard: &'a ComponentMutGuardBorrow<'_, C>) -> Self {
            Self {
                last_archetype_idx: None,
                components: guard.components.iter(),
                sorted_archetype_idxs: guard.sorted_archetype_idxs.clone(),
            }
        }
    }

    impl<'a, C> Iterator for ArchetypeComponentIter<'a, C> {
        type Item = Iter<'a, C>;

        fn next(&mut self) -> Option<Self::Item> {
            let archetype_idx = self.sorted_archetype_idxs.next()?;
            let nth = usize::from(archetype_idx)
                - self.last_archetype_idx.map_or(0, |i| usize::from(i) + 1);
            self.last_archetype_idx = Some(archetype_idx);
            self.components.nth(nth).map(|c| c.iter())
        }
    }

    impl<'a, C> DoubleEndedIterator for ArchetypeComponentIter<'a, C> {
        fn next_back(&mut self) -> Option<Self::Item> {
            let archetype_idx = self.sorted_archetype_idxs.next_back()?;
            let nth_back = self.components.len() - usize::from(archetype_idx) - 1;
            self.components.nth_back(nth_back).map(|c| c.iter())
        }
    }
}

#[cfg(test)]
mod component_ref_system_param_tests {
    use crate::components::internal::ComponentGuardBorrow;
    use crate::storages::archetypes::{
        ArchetypeFilter, ArchetypeStorage, FilteredArchetypeIdxIter,
    };
    use crate::storages::core::CoreStorage;
    use crate::storages::systems::Access;
    use crate::{QuerySystemParam, SystemInfo, SystemParam};

    #[test]
    fn retrieve_properties() {
        let mut core = CoreStorage::default();

        let properties = <&u32>::properties(&mut core);

        assert_eq!(properties.component_types.len(), 1);
        assert_eq!(properties.component_types[0].access, Access::Read);
        assert_eq!(properties.component_types[0].type_idx, 0.into());
        assert!(!properties.has_entity_actions);
        let archetype_filter = ArchetypeFilter::Intersection(ne_vec![0.into()]);
        assert_eq!(properties.archetype_filter, archetype_filter);
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

        let mut guard = <&u32>::lock(data, info);
        let mut guard_borrow = <&u32>::borrow_guard(&mut guard);

        let components = guard_borrow.components;
        assert_eq!(components, &ti_vec![ti_vec![], ti_vec![10_u32]]);
        assert_eq!(guard_borrow.item_count, 1);
        let next_archetype_idx = guard_borrow.sorted_archetype_idxs.next();
        assert_eq!(next_archetype_idx, Some(archetype2_idx));
        assert_eq!(guard_borrow.sorted_archetype_idxs.next(), None);
    }

    #[test]
    fn retrieve_stream() {
        let mut components = ti_vec![ti_vec![10], ti_vec![20], ti_vec![30]];
        components.extend(vec![ti_vec![40, 50], ti_vec![60]]);
        let archetype_idxs = [1.into(), 3.into()];
        let archetype_type_idxs = ti_vec![vec![0.into()]; 5];
        let mut guard_borrow = ComponentGuardBorrow {
            components: &components,
            item_count: 3,
            sorted_archetype_idxs: FilteredArchetypeIdxIter::new(
                &archetype_idxs,
                &archetype_type_idxs,
            ),
        };

        let mut stream = <&u32>::stream(&mut guard_borrow);

        assert_eq!(<&u32>::stream_next(&mut stream), Some(&20));
        assert_eq!(<&u32>::stream_next(&mut stream), Some(&40));
        assert_eq!(<&u32>::stream_next(&mut stream), Some(&50));
        assert_eq!(<&u32>::stream_next(&mut stream), None);
    }

    #[test]
    fn retrieve_query_iter() {
        let mut components = ti_vec![ti_vec![10], ti_vec![20], ti_vec![30]];
        components.extend(vec![ti_vec![40, 50], ti_vec![60]]);
        let archetype_idxs = [1.into(), 3.into()];
        let archetype_type_idxs = ti_vec![vec![0.into()]; 5];
        let guard_borrow = ComponentGuardBorrow {
            components: &components,
            item_count: 3,
            sorted_archetype_idxs: FilteredArchetypeIdxIter::new(
                &archetype_idxs,
                &archetype_type_idxs,
            ),
        };

        let mut iter = <&u32>::query_iter(&guard_borrow);

        assert_eq!(iter.len(), 3);
        assert_eq!(iter.next(), Some(&20));
        assert_eq!(iter.len(), 2);
        assert_eq!(iter.next(), Some(&40));
        assert_eq!(iter.len(), 1);
        assert_eq!(iter.next(), Some(&50));
        assert_eq!(iter.len(), 0);
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn retrieve_reversed_query_iter() {
        let mut components = ti_vec![ti_vec![10], ti_vec![20], ti_vec![30]];
        components.extend(vec![ti_vec![40, 50], ti_vec![60]]);
        let archetype_idxs = [1.into(), 3.into()];
        let archetype_type_idxs = ti_vec![vec![0.into()]; 5];
        let guard_borrow = ComponentGuardBorrow {
            components: &components,
            item_count: 3,
            sorted_archetype_idxs: FilteredArchetypeIdxIter::new(
                &archetype_idxs,
                &archetype_type_idxs,
            ),
        };

        let mut iter = <&u32>::query_iter(&guard_borrow).rev();

        assert_eq!(iter.len(), 3);
        assert_eq!(iter.next(), Some(&50));
        assert_eq!(iter.len(), 2);
        assert_eq!(iter.next(), Some(&40));
        assert_eq!(iter.len(), 1);
        assert_eq!(iter.next(), Some(&20));
        assert_eq!(iter.len(), 0);
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn retrieve_query_iter_mut() {
        let mut components = ti_vec![ti_vec![10], ti_vec![20], ti_vec![30]];
        components.extend(vec![ti_vec![40, 50], ti_vec![60]]);
        let archetype_idxs = [1.into(), 3.into()];
        let archetype_type_idxs = ti_vec![vec![0.into()]; 5];
        let mut guard_borrow = ComponentGuardBorrow {
            components: &components,
            item_count: 3,
            sorted_archetype_idxs: FilteredArchetypeIdxIter::new(
                &archetype_idxs,
                &archetype_type_idxs,
            ),
        };

        let mut iter = <&u32>::query_iter_mut(&mut guard_borrow);

        assert_eq!(iter.len(), 3);
        assert_eq!(iter.next(), Some(&20));
        assert_eq!(iter.len(), 2);
        assert_eq!(iter.next(), Some(&40));
        assert_eq!(iter.len(), 1);
        assert_eq!(iter.next(), Some(&50));
        assert_eq!(iter.len(), 0);
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn retrieve_reversed_query_iter_mut() {
        let mut components = ti_vec![ti_vec![10], ti_vec![20], ti_vec![30]];
        components.extend(vec![ti_vec![40, 50], ti_vec![60]]);
        let archetype_idxs = [1.into(), 3.into()];
        let archetype_type_idxs = ti_vec![vec![0.into()]; 5];
        let mut guard_borrow = ComponentGuardBorrow {
            components: &components,
            item_count: 3,
            sorted_archetype_idxs: FilteredArchetypeIdxIter::new(
                &archetype_idxs,
                &archetype_type_idxs,
            ),
        };

        let mut iter = <&u32>::query_iter_mut(&mut guard_borrow).rev();

        assert_eq!(iter.len(), 3);
        assert_eq!(iter.next(), Some(&50));
        assert_eq!(iter.len(), 2);
        assert_eq!(iter.next(), Some(&40));
        assert_eq!(iter.len(), 1);
        assert_eq!(iter.next(), Some(&20));
        assert_eq!(iter.len(), 0);
        assert_eq!(iter.next(), None);
    }
}
