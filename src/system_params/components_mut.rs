use crate::components_mut::internal::{ComponentMutGuard, ComponentMutGuardBorrow};
use crate::storages::archetypes::ArchetypeFilter;
use crate::storages::core::CoreStorage;
use crate::storages::systems::{Access, ComponentTypeAccess, SystemProperties};
use crate::system_params::components::internal::ComponentIter;
use crate::system_params::components_mut::internal::ComponentIterMut;
use crate::system_params::internal::{
    LockableSystemParam, Mut, QuerySystemParamWithLifetime, SystemParamWithLifetime,
};
use crate::{QuerySystemParam, SystemData, SystemInfo, SystemParam};
use std::any::Any;

impl<'a, C> SystemParamWithLifetime<'a> for &mut C
where
    C: Any + Sync + Send,
{
    type Param = &'a mut C;
    type Guard = ComponentMutGuard<'a, C>;
    type GuardBorrow = ComponentMutGuardBorrow<'a, C>;
    type Stream = ComponentIterMut<'a, C>;
}

impl<C> SystemParam for &mut C
where
    C: Any + Sync + Send,
{
    type Tuple = (Self,);
    type InnerTuple = ();

    fn properties(core: &mut CoreStorage) -> SystemProperties {
        let type_idx = core.register_component_type::<C>();
        SystemProperties {
            component_types: vec![ComponentTypeAccess {
                access: Access::Write,
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
        ComponentMutGuard::new(data, info)
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
        ComponentIterMut::new(guard)
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

impl<'a, C> QuerySystemParamWithLifetime<'a> for &mut C
where
    C: Any + Sync + Send,
{
    type ConstParam = &'a C;
    type Iter = ComponentIter<'a, C>;
    type IterMut = ComponentIterMut<'a, C>;
}

impl<C> QuerySystemParam for &mut C
where
    C: Any + Sync + Send,
{
    fn query_iter<'a, 'b>(
        guard: &'a <Self as SystemParamWithLifetime<'b>>::GuardBorrow,
    ) -> <Self as QuerySystemParamWithLifetime<'a>>::Iter
    where
        'b: 'a,
    {
        ComponentIter::new_mut(guard)
    }

    fn query_iter_mut<'a, 'b>(
        guard: &'a mut <Self as SystemParamWithLifetime<'b>>::GuardBorrow,
    ) -> <Self as QuerySystemParamWithLifetime<'a>>::IterMut
    where
        'b: 'a,
    {
        ComponentIterMut::new(guard)
    }
}

impl<C> LockableSystemParam for &mut C
where
    C: Any + Sync + Send,
{
    type LockedType = C;
    type Mutability = Mut;
}

pub(crate) mod internal {
    use crate::storages::archetypes::{ArchetypeEntityPos, ArchetypeIdx, FilteredArchetypeIdxIter};
    use crate::storages::components::ComponentArchetypes;
    use crate::{SystemData, SystemInfo};
    use std::any::Any;
    use std::iter::Flatten;
    use std::slice::IterMut;
    use std::sync::RwLockWriteGuard;
    use typed_index_collections::TiVec;

    pub struct ComponentMutGuard<'a, C> {
        components: RwLockWriteGuard<'a, ComponentArchetypes<C>>,
        data: SystemData<'a>,
        info: SystemInfo<'a>,
    }

    impl<'a, C> ComponentMutGuard<'a, C>
    where
        C: Any,
    {
        pub(crate) fn new(data: SystemData<'a>, info: SystemInfo<'a>) -> Self {
            Self {
                components: data.components.write_components::<C>(),
                data,
                info,
            }
        }

        pub(crate) fn borrow(&mut self) -> ComponentMutGuardBorrow<'_, C> {
            ComponentMutGuardBorrow {
                components: &mut *self.components,
                item_count: self.data.item_count(self.info),
                sorted_archetype_idxs: self.data.filter_archetype_idx_iter(self.info),
            }
        }
    }

    pub struct ComponentMutGuardBorrow<'a, C> {
        pub(crate) components: &'a mut ComponentArchetypes<C>,
        pub(crate) item_count: usize,
        pub(crate) sorted_archetype_idxs: FilteredArchetypeIdxIter<'a>,
    }

    pub struct ComponentIterMut<'a, C> {
        components: Flatten<ArchetypeComponentIter<'a, C>>,
        len: usize,
    }

    impl<'a, C> ComponentIterMut<'a, C> {
        pub(super) fn new(guard: &'a mut ComponentMutGuardBorrow<'_, C>) -> Self {
            Self {
                len: guard.item_count,
                components: ArchetypeComponentIter::new(guard).flatten(),
            }
        }
    }

    impl<'a, C> Iterator for ComponentIterMut<'a, C> {
        type Item = &'a mut C;

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

    impl<'a, C> DoubleEndedIterator for ComponentIterMut<'a, C> {
        #[inline]
        fn next_back(&mut self) -> Option<Self::Item> {
            self.components.next_back().map(|c| {
                self.len -= 1;
                c
            })
        }
    }

    impl<'a, C> ExactSizeIterator for ComponentIterMut<'a, C> {}

    struct ArchetypeComponentIter<'a, C> {
        last_archetype_idx: Option<ArchetypeIdx>,
        components: IterMut<'a, TiVec<ArchetypeEntityPos, C>>,
        sorted_archetype_idxs: FilteredArchetypeIdxIter<'a>,
    }

    impl<'a, C> ArchetypeComponentIter<'a, C> {
        fn new(guard: &'a mut ComponentMutGuardBorrow<'_, C>) -> Self {
            Self {
                last_archetype_idx: None,
                components: guard.components.iter_mut(),
                sorted_archetype_idxs: guard.sorted_archetype_idxs.clone(),
            }
        }
    }

    impl<'a, C> Iterator for ArchetypeComponentIter<'a, C> {
        type Item = IterMut<'a, C>;

        fn next(&mut self) -> Option<Self::Item> {
            let archetype_idx = self.sorted_archetype_idxs.next()?;
            let nth = usize::from(archetype_idx)
                - self.last_archetype_idx.map_or(0, |i| usize::from(i) + 1);
            self.last_archetype_idx = Some(archetype_idx);
            self.components.nth(nth).map(|c| c.iter_mut())
        }
    }

    impl<'a, C> DoubleEndedIterator for ArchetypeComponentIter<'a, C> {
        fn next_back(&mut self) -> Option<Self::Item> {
            let archetype_idx = self.sorted_archetype_idxs.next_back()?;
            let nth_back = self.components.len() - usize::from(archetype_idx) - 1;
            self.components.nth_back(nth_back).map(|c| c.iter_mut())
        }
    }
}

#[cfg(test)]
mod component_mut_system_param_tests {
    use crate::components_mut::internal::ComponentMutGuardBorrow;
    use crate::storages::archetypes::{
        ArchetypeFilter, ArchetypeStorage, FilteredArchetypeIdxIter,
    };
    use crate::storages::core::CoreStorage;
    use crate::storages::systems::Access;
    use crate::{QuerySystemParam, SystemInfo, SystemParam};

    #[test]
    fn retrieve_properties() {
        let mut core = CoreStorage::default();

        let properties = <&mut u32>::properties(&mut core);

        assert_eq!(properties.component_types.len(), 1);
        assert_eq!(properties.component_types[0].access, Access::Write);
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
        };

        let mut guard = <&mut u32>::lock(data, info);
        let mut guard_borrow = <&mut u32>::borrow_guard(&mut guard);

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
        let mut guard_borrow = ComponentMutGuardBorrow {
            components: &mut components,
            item_count: 3,
            sorted_archetype_idxs: FilteredArchetypeIdxIter::new(
                &archetype_idxs,
                &archetype_type_idxs,
            ),
        };

        let mut stream = <&mut u32>::stream(&mut guard_borrow);

        assert_eq!(<&mut u32>::stream_next(&mut stream), Some(&mut 20));
        assert_eq!(<&mut u32>::stream_next(&mut stream), Some(&mut 40));
        assert_eq!(<&mut u32>::stream_next(&mut stream), Some(&mut 50));
        assert_eq!(<&mut u32>::stream_next(&mut stream), None);
    }

    #[test]
    fn retrieve_query_iter() {
        let mut components = ti_vec![ti_vec![10], ti_vec![20], ti_vec![30]];
        components.extend(vec![ti_vec![40, 50], ti_vec![60]]);
        let archetype_idxs = [1.into(), 3.into()];
        let archetype_type_idxs = ti_vec![vec![0.into()]; 5];
        let guard_borrow = ComponentMutGuardBorrow {
            components: &mut components,
            item_count: 3,
            sorted_archetype_idxs: FilteredArchetypeIdxIter::new(
                &archetype_idxs,
                &archetype_type_idxs,
            ),
        };

        let mut iter = <&mut u32>::query_iter(&guard_borrow);

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
        let guard_borrow = ComponentMutGuardBorrow {
            components: &mut components,
            item_count: 3,
            sorted_archetype_idxs: FilteredArchetypeIdxIter::new(
                &archetype_idxs,
                &archetype_type_idxs,
            ),
        };

        let mut iter = <&mut u32>::query_iter(&guard_borrow).rev();

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
        let mut guard_borrow = ComponentMutGuardBorrow {
            components: &mut components,
            item_count: 3,
            sorted_archetype_idxs: FilteredArchetypeIdxIter::new(
                &archetype_idxs,
                &archetype_type_idxs,
            ),
        };

        let mut iter = <&mut u32>::query_iter_mut(&mut guard_borrow);

        assert_eq!(iter.len(), 3);
        assert_eq!(iter.next(), Some(&mut 20));
        assert_eq!(iter.len(), 2);
        assert_eq!(iter.next(), Some(&mut 40));
        assert_eq!(iter.len(), 1);
        assert_eq!(iter.next(), Some(&mut 50));
        assert_eq!(iter.len(), 0);
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn retrieve_reversed_query_iter_mut() {
        let mut components = ti_vec![ti_vec![10], ti_vec![20], ti_vec![30]];
        components.extend(vec![ti_vec![40, 50], ti_vec![60]]);
        let archetype_idxs = [1.into(), 3.into()];
        let archetype_type_idxs = ti_vec![vec![0.into()]; 5];
        let mut guard_borrow = ComponentMutGuardBorrow {
            components: &mut components,
            item_count: 3,
            sorted_archetype_idxs: FilteredArchetypeIdxIter::new(
                &archetype_idxs,
                &archetype_type_idxs,
            ),
        };

        let mut iter = <&mut u32>::query_iter_mut(&mut guard_borrow).rev();

        assert_eq!(iter.len(), 3);
        assert_eq!(iter.next(), Some(&mut 50));
        assert_eq!(iter.len(), 2);
        assert_eq!(iter.next(), Some(&mut 40));
        assert_eq!(iter.len(), 1);
        assert_eq!(iter.next(), Some(&mut 20));
        assert_eq!(iter.len(), 0);
        assert_eq!(iter.next(), None);
    }
}
