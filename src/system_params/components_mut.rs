use crate::storages::components::{ComponentArchetypes, ComponentStorage};
use crate::storages::core::{ComponentTypeIdAccess, SystemProperties};
use crate::storages::systems::Access;
use crate::system_params::components::internal::ComponentIter;
use crate::system_params::components_mut::internal::ComponentIterMut;
use crate::system_params::internal::{
    EntityIterInfo, LockableSystemParam, Mut, QuerySystemParamWithLifetime, SystemParamIterInfo,
    SystemParamWithLifetime,
};
use crate::{QuerySystemParam, SystemData, SystemInfo, SystemParam};
use std::any::{Any, TypeId};
use std::sync::RwLockWriteGuard;

impl<'a, C> SystemParamWithLifetime<'a> for &mut C
where
    C: Any + Sync + Send,
{
    type Param = &'a mut C;
    type Guard = RwLockWriteGuard<'a, ComponentArchetypes<C>>;
    type GuardBorrow = &'a mut ComponentArchetypes<C>;
    type Stream = ComponentIterMut<'a, C>;
}

impl<C> SystemParam for &mut C
where
    C: Any + Sync + Send,
{
    type Tuple = (Self,);
    type InnerTuple = ();

    fn properties() -> SystemProperties {
        SystemProperties {
            component_types: vec![ComponentTypeIdAccess {
                access: Access::Write,
                type_idx_or_create_fn: ComponentStorage::type_idx_or_create::<C>,
            }],
            has_entity_actions: false,
        }
    }

    fn iter_info(data: &SystemData<'_>, info: &SystemInfo) -> SystemParamIterInfo {
        let mut component_types = info.filtered_component_types.clone();
        component_types.push(TypeId::of::<C>());
        SystemParamIterInfo::ComponentIntersectionEntities(EntityIterInfo {
            sorted_archetypes: data
                .components
                .type_idxs(&component_types)
                .map_or_else(Vec::new, |i| data.archetypes.sorted_with_all_types(&i)),
        })
    }

    fn lock<'a>(data: &'a SystemData<'_>) -> <Self as SystemParamWithLifetime<'a>>::Guard {
        data.components.write_components::<C>()
    }

    fn borrow_guard<'a, 'b>(
        guard: &'a mut <Self as SystemParamWithLifetime<'b>>::Guard,
    ) -> <Self as SystemParamWithLifetime<'a>>::GuardBorrow
    where
        'b: 'a,
    {
        &mut *guard
    }

    fn stream<'a, 'b>(
        guard: &'a mut <Self as SystemParamWithLifetime<'b>>::GuardBorrow,
        info: &'a SystemParamIterInfo,
    ) -> <Self as SystemParamWithLifetime<'a>>::Stream
    where
        'b: 'a,
    {
        ComponentIterMut::new(info, guard)
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
        info: &'a SystemParamIterInfo,
    ) -> <Self as QuerySystemParamWithLifetime<'a>>::Iter
    where
        'b: 'a,
    {
        ComponentIter::new(info, guard)
    }

    fn query_iter_mut<'a, 'b>(
        guard: &'a mut <Self as SystemParamWithLifetime<'b>>::GuardBorrow,
        info: &'a SystemParamIterInfo,
    ) -> <Self as QuerySystemParamWithLifetime<'a>>::IterMut
    where
        'b: 'a,
    {
        ComponentIterMut::new(info, guard)
    }
}

impl<C> LockableSystemParam for &mut C
where
    C: Any + Sync + Send,
{
    type LockedType = C;
    type Mutability = Mut;
}

mod internal {
    use crate::storages::archetypes::{ArchetypeEntityPos, ArchetypeIdx, ArchetypeInfo};
    use crate::storages::components::ComponentArchetypes;
    use crate::system_params::internal::SystemParamIterInfo;
    use std::iter::Flatten;
    use std::slice::{Iter, IterMut};
    use typed_index_collections::TiVec;

    pub struct ComponentIterMut<'a, C> {
        components: Flatten<ArchetypeComponentIter<'a, C>>,
        len: usize,
    }

    impl<'a, C> ComponentIterMut<'a, C> {
        pub(super) fn new(
            info: &'a SystemParamIterInfo,
            component_archetypes: &'a mut ComponentArchetypes<C>,
        ) -> Self {
            let sorted_archetypes = info
                .sorted_archetypes()
                .expect("internal error: wrong iter mode for mut components");
            let archetype_iter =
                ArchetypeComponentIter::new(sorted_archetypes, component_archetypes);
            Self {
                components: archetype_iter.flatten(),
                len: info.item_count(),
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
        sorted_archetypes: Iter<'a, ArchetypeInfo>,
        last_archetype_idx: Option<ArchetypeIdx>,
        component_archetypes: IterMut<'a, TiVec<ArchetypeEntityPos, C>>,
    }

    impl<'a, C> ArchetypeComponentIter<'a, C> {
        fn new(
            sorted_archetypes: &'a [ArchetypeInfo],
            component_archetypes: &'a mut ComponentArchetypes<C>,
        ) -> Self {
            Self {
                sorted_archetypes: sorted_archetypes.iter(),
                last_archetype_idx: None,
                component_archetypes: component_archetypes.iter_mut(),
            }
        }
    }

    impl<'a, C> Iterator for ArchetypeComponentIter<'a, C> {
        type Item = IterMut<'a, C>;

        fn next(&mut self) -> Option<Self::Item> {
            let archetype = self.sorted_archetypes.next()?;
            let nth = usize::from(archetype.idx)
                - self.last_archetype_idx.map_or(0, |i| usize::from(i) + 1);
            self.last_archetype_idx = Some(archetype.idx);
            self.component_archetypes.nth(nth).map(|c| c.iter_mut())
        }
    }

    impl<'a, C> DoubleEndedIterator for ArchetypeComponentIter<'a, C> {
        fn next_back(&mut self) -> Option<Self::Item> {
            let archetype = self.sorted_archetypes.next_back()?;
            let nth_back = self.component_archetypes.len() - usize::from(archetype.idx) - 1;
            self.component_archetypes
                .nth_back(nth_back)
                .map(|c| c.iter_mut())
        }
    }
}

#[cfg(test)]
mod component_mut_system_param_tests {
    use crate::storages::archetypes::ArchetypeStorage;
    use crate::storages::core::CoreStorage;
    use crate::storages::systems::Access;
    use crate::system_params::internal::SystemParamIterInfo;
    use crate::{QuerySystemParam, SystemInfo, SystemParam};

    #[test]
    fn retrieve_properties() {
        let properties = <&mut u32>::properties();

        assert_eq!(properties.component_types.len(), 1);
        assert_eq!(properties.component_types[0].access, Access::Write);
        assert!(!properties.has_entity_actions);
    }

    #[test]
    fn retrieve_iter_info_from_missing_component_type() {
        let mut core = CoreStorage::default();
        core.add_component_type::<i64>(ArchetypeStorage::DEFAULT_IDX);
        let info = SystemInfo::with_one_filtered_type::<i64>();

        let iter_info = <&mut u32>::iter_info(&core.system_data(), &info);

        assert_eq!(iter_info, SystemParamIterInfo::new_intersection(vec![]));
    }

    #[test]
    fn retrieve_iter_info_from_existing_component_type() {
        let mut core = CoreStorage::default();
        let (_, archetype1_idx) = core.add_component_type::<i64>(ArchetypeStorage::DEFAULT_IDX);
        let (_, archetype2_idx) = core.add_component_type::<u32>(archetype1_idx);
        let info = SystemInfo::with_one_filtered_type::<i64>();

        let iter_info = <&mut u32>::iter_info(&core.system_data(), &info);

        let expected_iter_info = SystemParamIterInfo::new_intersection(vec![(archetype2_idx, 0)]);
        assert_eq!(iter_info, expected_iter_info);
    }

    #[test]
    fn lock() {
        let mut core = CoreStorage::default();
        let archetype1_idx = ArchetypeStorage::DEFAULT_IDX;
        let (type_idx, archetype2_idx) = core.add_component_type::<u32>(archetype1_idx);
        let location = core.create_entity(archetype2_idx);
        core.add_component(10_u32, type_idx, location);
        let data = core.system_data();

        let mut guard = <&mut u32>::lock(&data);
        let guard_borrow = <&mut u32>::borrow_guard(&mut guard);

        assert_eq!(guard_borrow, &ti_vec![ti_vec![], ti_vec![10_u32]]);
    }

    #[test]
    fn retrieve_stream() {
        let mut guard = ti_vec![ti_vec![10], ti_vec![20], ti_vec![30]];
        guard.extend(vec![ti_vec![40, 50], ti_vec![60]]);
        let mut guard_borrow = &mut guard;
        let iter_info = SystemParamIterInfo::new_intersection(vec![(1.into(), 1), (3.into(), 2)]);

        let mut stream = <&mut u32>::stream(&mut guard_borrow, &iter_info);

        assert_eq!(<&mut u32>::stream_next(&mut stream), Some(&mut 20));
        assert_eq!(<&mut u32>::stream_next(&mut stream), Some(&mut 40));
        assert_eq!(<&mut u32>::stream_next(&mut stream), Some(&mut 50));
        assert_eq!(<&mut u32>::stream_next(&mut stream), None);
    }

    #[test]
    fn retrieve_query_iter() {
        let mut guard = ti_vec![ti_vec![10], ti_vec![20], ti_vec![30]];
        guard.extend(vec![ti_vec![40, 50], ti_vec![60]]);
        let guard_borrow = &mut guard;
        let iter_info = SystemParamIterInfo::new_intersection(vec![(1.into(), 1), (3.into(), 2)]);

        let mut iter = <&mut u32>::query_iter(&guard_borrow, &iter_info);

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
        let mut guard = ti_vec![ti_vec![10], ti_vec![20], ti_vec![30]];
        guard.extend(vec![ti_vec![40, 50], ti_vec![60]]);
        let guard_borrow = &mut guard;
        let iter_info = SystemParamIterInfo::new_intersection(vec![(1.into(), 1), (3.into(), 2)]);

        let mut iter = <&mut u32>::query_iter(&guard_borrow, &iter_info).rev();

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
        let mut guard = ti_vec![ti_vec![10], ti_vec![20], ti_vec![30]];
        guard.extend(vec![ti_vec![40, 50], ti_vec![60]]);
        let mut guard_borrow = &mut guard;
        let iter_info = SystemParamIterInfo::new_intersection(vec![(1.into(), 1), (3.into(), 2)]);

        let mut iter = <&mut u32>::query_iter_mut(&mut guard_borrow, &iter_info);

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
        let mut guard = ti_vec![ti_vec![10], ti_vec![20], ti_vec![30]];
        guard.extend(vec![ti_vec![40, 50], ti_vec![60]]);
        let mut guard_borrow = &mut guard;
        let iter_info = SystemParamIterInfo::new_intersection(vec![(1.into(), 1), (3.into(), 2)]);

        let mut iter = <&mut u32>::query_iter_mut(&mut guard_borrow, &iter_info).rev();

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
