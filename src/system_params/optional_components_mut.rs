use crate::storages::components::ComponentArchetypes;
use crate::storages::core::CoreStorage;
use crate::storages::systems::{Access, ComponentTypeAccess, SystemProperties};
use crate::system_params::internal::{
    EntityIterInfo, LockableSystemParam, Mut, QuerySystemParamWithLifetime, SystemParamIterInfo,
    SystemParamWithLifetime,
};
use crate::system_params::optional_components::internal::ComponentOptionIter;
use crate::system_params::optional_components_mut::internal::ComponentOptionIterMut;
use crate::{QuerySystemParam, SystemData, SystemInfo, SystemParam};
use std::any::{Any, TypeId};
use std::sync::RwLockWriteGuard;

impl<'a, C> SystemParamWithLifetime<'a> for Option<&mut C>
where
    C: Any + Sync + Send,
{
    type Param = Option<&'a mut C>;
    type Guard = RwLockWriteGuard<'a, ComponentArchetypes<C>>;
    type GuardBorrow = &'a mut ComponentArchetypes<C>;
    type Stream = ComponentOptionIterMut<'a, C>;
}

impl<C> SystemParam for Option<&mut C>
where
    C: Any + Sync + Send,
{
    type Tuple = (Self,);
    type InnerTuple = ();

    fn properties(core: &mut CoreStorage) -> SystemProperties {
        SystemProperties {
            component_types: vec![ComponentTypeAccess {
                access: Access::Write,
                type_idx: core.register_component_type::<C>(),
            }],
            has_entity_actions: false,
        }
    }

    fn iter_info(data: &SystemData<'_>, info: &SystemInfo) -> SystemParamIterInfo {
        let mut component_type_idxs = info.filtered_component_type_idxs.clone();
        component_type_idxs.push(
            data.components.type_idx(TypeId::of::<C>()).expect(
                "internal error: missing mut component type option when retrieving iter info",
            ),
        );
        SystemParamIterInfo::ComponentUnionEntities(EntityIterInfo {
            sorted_archetypes: data.archetypes.sorted_with_all_types(&component_type_idxs),
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
        ComponentOptionIterMut::new(info, guard)
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

impl<'a, C> QuerySystemParamWithLifetime<'a> for Option<&mut C>
where
    C: Any + Sync + Send,
{
    type ConstParam = Option<&'a C>;
    type Iter = ComponentOptionIter<'a, C>;
    type IterMut = ComponentOptionIterMut<'a, C>;
}

impl<C> QuerySystemParam for Option<&mut C>
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
        ComponentOptionIter::new(info, guard)
    }

    fn query_iter_mut<'a, 'b>(
        guard: &'a mut <Self as SystemParamWithLifetime<'b>>::GuardBorrow,
        info: &'a SystemParamIterInfo,
    ) -> <Self as QuerySystemParamWithLifetime<'a>>::IterMut
    where
        'b: 'a,
    {
        ComponentOptionIterMut::new(info, guard)
    }
}

impl<C> LockableSystemParam for Option<&mut C>
where
    C: Any + Sync + Send,
{
    type LockedType = C;
    type Mutability = Mut;
}

pub(crate) mod internal {
    use crate::storages::archetypes::{ArchetypeEntityPos, ArchetypeIdx, ArchetypeInfo};
    use crate::storages::components::ComponentArchetypes;
    use crate::system_params::internal::SystemParamIterInfo;
    use std::iter::Flatten;
    use std::ops::Range;
    use std::slice::{Iter, IterMut};
    use typed_index_collections::TiVec;

    pub struct ComponentOptionIterMut<'a, C> {
        components: Flatten<ArchetypeComponentIter<'a, C>>,
        len: usize,
    }

    impl<'a, C> ComponentOptionIterMut<'a, C> {
        pub(super) fn new(
            info: &'a SystemParamIterInfo,
            component_archetypes: &'a mut ComponentArchetypes<C>,
        ) -> Self {
            let sorted_archetypes = info
                .sorted_archetypes()
                .expect("internal error: wrong iter mode for optional mut components");
            let archetype_iter =
                ArchetypeComponentIter::new(sorted_archetypes, component_archetypes);
            Self {
                components: archetype_iter.flatten(),
                len: info.item_count(),
            }
        }
    }

    impl<'a, C> Iterator for ComponentOptionIterMut<'a, C> {
        type Item = Option<&'a mut C>;

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

    impl<'a, C> DoubleEndedIterator for ComponentOptionIterMut<'a, C> {
        #[inline]
        fn next_back(&mut self) -> Option<Self::Item> {
            self.components.next_back().map(|c| {
                self.len -= 1;
                c
            })
        }
    }

    impl<'a, C> ExactSizeIterator for ComponentOptionIterMut<'a, C> {}

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
        type Item = ComponentIter<'a, C>;

        fn next(&mut self) -> Option<Self::Item> {
            let archetype = self.sorted_archetypes.next()?;
            let nth = usize::from(archetype.idx)
                - self.last_archetype_idx.map_or(0, |i| usize::from(i) + 1);
            self.last_archetype_idx = Some(archetype.idx);
            Some(ComponentIter::new(
                self.component_archetypes.nth(nth).map(|c| c.iter_mut()),
                archetype.entity_count,
            ))
        }
    }

    impl<'a, C> DoubleEndedIterator for ArchetypeComponentIter<'a, C> {
        fn next_back(&mut self) -> Option<Self::Item> {
            let archetype = self.sorted_archetypes.next_back()?;
            let nth_back = self.component_archetypes.len() - usize::from(archetype.idx) - 1;
            Some(ComponentIter::new(
                self.component_archetypes
                    .nth_back(nth_back)
                    .map(|c| c.iter_mut()),
                archetype.entity_count,
            ))
        }
    }

    struct ComponentIter<'a, C> {
        components: Option<IterMut<'a, C>>,
        entity_positions: Range<usize>,
    }

    impl<'a, C> ComponentIter<'a, C> {
        fn new(components: Option<IterMut<'a, C>>, entity_count: usize) -> Self {
            Self {
                components,
                entity_positions: 0..entity_count,
            }
        }
    }

    impl<'a, C> Iterator for ComponentIter<'a, C> {
        type Item = Option<&'a mut C>;

        #[inline]
        fn next(&mut self) -> Option<Self::Item> {
            self.entity_positions.next().map(|_| {
                self.components
                    .as_mut()
                    .expect("internal error: missing component mut during iteration")
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
                    .expect("internal error: missing component mut during reversed iteration")
                    .next_back()
            })
        }
    }
}

#[cfg(test)]
mod component_mut_option_system_param_tests {
    use crate::storages::archetypes::ArchetypeStorage;
    use crate::storages::core::CoreStorage;
    use crate::storages::systems::Access;
    use crate::system_params::internal::SystemParamIterInfo;
    use crate::{QuerySystemParam, SystemInfo, SystemParam};

    #[test]
    fn retrieve_properties() {
        let mut core = CoreStorage::default();

        let properties = Option::<&mut u32>::properties(&mut core);

        assert_eq!(properties.component_types.len(), 1);
        assert_eq!(properties.component_types[0].access, Access::Write);
        assert_eq!(properties.component_types[0].type_idx, 0.into());
        assert!(!properties.has_entity_actions);
    }

    #[test]
    fn retrieve_iter_info() {
        let mut core = CoreStorage::default();
        let (_, archetype1_idx) = core.add_component_type::<i64>(ArchetypeStorage::DEFAULT_IDX);
        let (_, archetype2_idx) = core.add_component_type::<u32>(archetype1_idx);
        let info = SystemInfo::from_one_filtered_type(0.into());

        let iter_info = Option::<&mut u32>::iter_info(&core.system_data(), &info);

        let expected_iter_info = SystemParamIterInfo::new_union(vec![(archetype2_idx, 0)]);
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

        let mut guard = Option::<&mut u32>::lock(&data);
        let guard_borrow = Option::<&mut u32>::borrow_guard(&mut guard);

        assert_eq!(guard_borrow, &ti_vec![ti_vec![], ti_vec![10_u32]]);
    }

    #[test]
    fn retrieve_stream() {
        let mut guard = ti_vec![ti_vec![10], ti_vec![20], ti_vec![30]];
        guard.extend(vec![ti_vec![], ti_vec![40, 50], ti_vec![60]]);
        let mut guard_borrow = &mut guard;
        let iter_info =
            SystemParamIterInfo::new_union(vec![(1.into(), 1), (3.into(), 2), (4.into(), 2)]);

        let mut stream = Option::<&mut u32>::stream(&mut guard_borrow, &iter_info);

        let next = Option::<&mut u32>::stream_next(&mut stream);
        assert_eq!(next, Some(Some(&mut 20)));
        assert_eq!(Option::<&mut u32>::stream_next(&mut stream), Some(None));
        assert_eq!(Option::<&mut u32>::stream_next(&mut stream), Some(None));
        let next = Option::<&mut u32>::stream_next(&mut stream);
        assert_eq!(next, Some(Some(&mut 40)));
        let next = Option::<&mut u32>::stream_next(&mut stream);
        assert_eq!(next, Some(Some(&mut 50)));
        assert_eq!(Option::<&mut u32>::stream_next(&mut stream), None);
    }

    #[test]
    fn retrieve_query_iter() {
        let mut guard = ti_vec![ti_vec![10], ti_vec![20], ti_vec![30]];
        guard.extend(vec![ti_vec![], ti_vec![40, 50], ti_vec![60]]);
        let guard_borrow = &mut guard;
        let iter_info =
            SystemParamIterInfo::new_union(vec![(1.into(), 1), (3.into(), 2), (4.into(), 2)]);

        let mut iter = Option::<&mut u32>::query_iter(&guard_borrow, &iter_info);

        assert_eq!(iter.len(), 5);
        assert_eq!(iter.next(), Some(Some(&20)));
        assert_eq!(iter.len(), 4);
        assert_eq!(iter.next(), Some(None));
        assert_eq!(iter.len(), 3);
        assert_eq!(iter.next(), Some(None));
        assert_eq!(iter.len(), 2);
        assert_eq!(iter.next(), Some(Some(&40)));
        assert_eq!(iter.len(), 1);
        assert_eq!(iter.next(), Some(Some(&50)));
        assert_eq!(iter.len(), 0);
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn retrieve_reversed_query_iter() {
        let mut guard = ti_vec![ti_vec![10], ti_vec![20], ti_vec![30]];
        guard.extend(vec![ti_vec![], ti_vec![40, 50], ti_vec![60]]);
        let guard_borrow = &mut guard;
        let iter_info =
            SystemParamIterInfo::new_union(vec![(1.into(), 1), (3.into(), 2), (4.into(), 2)]);

        let mut iter = Option::<&mut u32>::query_iter(&guard_borrow, &iter_info).rev();

        assert_eq!(iter.len(), 5);
        assert_eq!(iter.next(), Some(Some(&50)));
        assert_eq!(iter.len(), 4);
        assert_eq!(iter.next(), Some(Some(&40)));
        assert_eq!(iter.len(), 3);
        assert_eq!(iter.next(), Some(None));
        assert_eq!(iter.len(), 2);
        assert_eq!(iter.next(), Some(None));
        assert_eq!(iter.len(), 1);
        assert_eq!(iter.next(), Some(Some(&20)));
        assert_eq!(iter.len(), 0);
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn retrieve_query_iter_mut() {
        let mut guard = ti_vec![ti_vec![10], ti_vec![20], ti_vec![30]];
        guard.extend(vec![ti_vec![], ti_vec![40, 50], ti_vec![60]]);
        let mut guard_borrow = &mut guard;
        let iter_info =
            SystemParamIterInfo::new_union(vec![(1.into(), 1), (3.into(), 2), (4.into(), 2)]);

        let mut iter = Option::<&mut u32>::query_iter_mut(&mut guard_borrow, &iter_info);

        assert_eq!(iter.len(), 5);
        assert_eq!(iter.next(), Some(Some(&mut 20)));
        assert_eq!(iter.len(), 4);
        assert_eq!(iter.next(), Some(None));
        assert_eq!(iter.len(), 3);
        assert_eq!(iter.next(), Some(None));
        assert_eq!(iter.len(), 2);
        assert_eq!(iter.next(), Some(Some(&mut 40)));
        assert_eq!(iter.len(), 1);
        assert_eq!(iter.next(), Some(Some(&mut 50)));
        assert_eq!(iter.len(), 0);
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn retrieve_reversed_query_iter_mut() {
        let mut guard = ti_vec![ti_vec![10], ti_vec![20], ti_vec![30]];
        guard.extend(vec![ti_vec![], ti_vec![40, 50], ti_vec![60]]);
        let mut guard_borrow = &mut guard;
        let iter_info =
            SystemParamIterInfo::new_union(vec![(1.into(), 1), (3.into(), 2), (4.into(), 2)]);

        let mut iter = Option::<&mut u32>::query_iter_mut(&mut guard_borrow, &iter_info).rev();

        assert_eq!(iter.len(), 5);
        assert_eq!(iter.next(), Some(Some(&mut 50)));
        assert_eq!(iter.len(), 4);
        assert_eq!(iter.next(), Some(Some(&mut 40)));
        assert_eq!(iter.len(), 3);
        assert_eq!(iter.next(), Some(None));
        assert_eq!(iter.len(), 2);
        assert_eq!(iter.next(), Some(None));
        assert_eq!(iter.len(), 1);
        assert_eq!(iter.next(), Some(Some(&mut 20)));
        assert_eq!(iter.len(), 0);
        assert_eq!(iter.next(), None);
    }
}
