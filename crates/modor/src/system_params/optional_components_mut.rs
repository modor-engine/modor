use crate::optional_components_mut::internal::{
    ComponentMutOptionGuard, ComponentMutOptionGuardBorrow,
};
use crate::storages::archetypes::EntityLocation;
use crate::storages::components::ComponentTypeIdx;
use crate::storages::core::CoreStorage;
use crate::storages::systems::{Access, ComponentTypeAccess, SystemProperties};
use crate::system_params::internal::{
    LockableSystemParam, Mut, QuerySystemParamWithLifetime, SystemParamWithLifetime,
};
use crate::system_params::optional_components::internal::ComponentOptionIter;
use crate::system_params::optional_components_mut::internal::ComponentMutOptionIter;
use crate::system_params::utils;
use crate::{QuerySystemParam, SystemData, SystemInfo, SystemParam};
use std::any::Any;

impl<'a, C> SystemParamWithLifetime<'a> for Option<&mut C>
where
    C: Any + Sync + Send,
{
    type Param = Option<&'a mut C>;
    type Guard = ComponentMutOptionGuard<'a, C>;
    type GuardBorrow = ComponentMutOptionGuardBorrow<'a, C>;
    type Stream = ComponentMutOptionIter<'a, C>;
}

impl<C> SystemParam for Option<&mut C>
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
            can_update: false,
            filtered_component_type_idxs: vec![],
        }
    }

    fn lock<'a>(
        data: SystemData<'a>,
        info: SystemInfo<'a>,
    ) -> <Self as SystemParamWithLifetime<'a>>::Guard {
        ComponentMutOptionGuard::new(data, info)
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
        ComponentMutOptionIter::new(guard)
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
    type IterMut = ComponentMutOptionIter<'a, C>;
}

impl<C> QuerySystemParam for Option<&mut C>
where
    C: Any + Sync + Send,
{
    fn filtered_component_type_idxs(_data: SystemData<'_>) -> Vec<ComponentTypeIdx> {
        vec![]
    }

    fn query_iter<'a, 'b>(
        guard: &'a <Self as SystemParamWithLifetime<'b>>::GuardBorrow,
    ) -> <Self as QuerySystemParamWithLifetime<'a>>::Iter
    where
        'b: 'a,
    {
        ComponentOptionIter::new_mut(guard)
    }

    fn query_iter_mut<'a, 'b>(
        guard: &'a mut <Self as SystemParamWithLifetime<'b>>::GuardBorrow,
    ) -> <Self as QuerySystemParamWithLifetime<'a>>::IterMut
    where
        'b: 'a,
    {
        ComponentMutOptionIter::new(guard)
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
                .get_mut(location.idx)
                .and_then(|a| a.get_mut(location.pos)),
        )
    }

    #[inline]
    fn get_both_mut<'a, 'b>(
        guard: &'a mut <Self as SystemParamWithLifetime<'b>>::GuardBorrow,
        location1: EntityLocation,
        location2: EntityLocation,
    ) -> (
        Option<<Self as SystemParamWithLifetime<'a>>::Param>,
        Option<<Self as SystemParamWithLifetime<'a>>::Param>,
    )
    where
        'b: 'a,
    {
        let (item1, item2) = utils::get_both_mut(guard.components, location1, location2);
        (Some(item1), Some(item2))
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
    use crate::storages::archetypes::{ArchetypeEntityPos, ArchetypeIdx, FilteredArchetypeIdxIter};
    use crate::storages::components::ComponentArchetypes;
    use crate::{SystemData, SystemInfo};
    use std::any::Any;
    use std::iter::Flatten;
    use std::ops::Range;
    use std::slice::IterMut;
    use std::sync::RwLockWriteGuard;
    use typed_index_collections::TiVec;

    pub struct ComponentMutOptionGuard<'a, C> {
        components: RwLockWriteGuard<'a, ComponentArchetypes<C>>,
        data: SystemData<'a>,
        info: SystemInfo<'a>,
    }

    impl<'a, C> ComponentMutOptionGuard<'a, C>
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

        pub(crate) fn borrow(&mut self) -> ComponentMutOptionGuardBorrow<'_, C> {
            ComponentMutOptionGuardBorrow {
                components: &mut *self.components,
                item_count: self.info.item_count,
                sorted_archetype_idxs: self
                    .data
                    .filter_archetype_idx_iter(self.info.filtered_component_type_idxs),
                data: self.data,
            }
        }
    }

    pub struct ComponentMutOptionGuardBorrow<'a, C> {
        pub(crate) components: &'a mut ComponentArchetypes<C>,
        pub(crate) item_count: usize,
        pub(crate) sorted_archetype_idxs: FilteredArchetypeIdxIter<'a>,
        pub(crate) data: SystemData<'a>,
    }

    pub struct ComponentMutOptionIter<'a, C> {
        components: Flatten<ArchetypeComponentIter<'a, C>>,
        len: usize,
    }

    impl<'a, C> ComponentMutOptionIter<'a, C> {
        pub(super) fn new(guard: &'a mut ComponentMutOptionGuardBorrow<'_, C>) -> Self {
            Self {
                len: guard.item_count,
                components: ArchetypeComponentIter::new(guard).flatten(),
            }
        }
    }

    impl<'a, C> Iterator for ComponentMutOptionIter<'a, C> {
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

    impl<'a, C> DoubleEndedIterator for ComponentMutOptionIter<'a, C> {
        #[inline]
        fn next_back(&mut self) -> Option<Self::Item> {
            self.components.next_back().map(|c| {
                self.len -= 1;
                c
            })
        }
    }

    impl<'a, C> ExactSizeIterator for ComponentMutOptionIter<'a, C> {}

    struct ArchetypeComponentIter<'a, C> {
        last_archetype_idx: Option<ArchetypeIdx>,
        components: IterMut<'a, TiVec<ArchetypeEntityPos, C>>,
        sorted_archetype_idxs: FilteredArchetypeIdxIter<'a>,
        data: SystemData<'a>,
    }

    impl<'a, C> ArchetypeComponentIter<'a, C> {
        fn new(guard: &'a mut ComponentMutOptionGuardBorrow<'_, C>) -> Self {
            Self {
                last_archetype_idx: None,
                components: guard.components.iter_mut(),
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
                self.components.nth(nth).map(|c| c.iter_mut()),
                self.data.archetypes.entity_idxs(archetype_idx).len(),
            ))
        }
    }

    impl<'a, C> DoubleEndedIterator for ArchetypeComponentIter<'a, C> {
        fn next_back(&mut self) -> Option<Self::Item> {
            let archetype_idx = self.sorted_archetype_idxs.next_back()?;
            let nth_back = self.components.len() - usize::from(archetype_idx) - 1;
            Some(ComponentIter::new(
                self.components.nth_back(nth_back).map(|c| c.iter_mut()),
                self.data.archetypes.entity_idxs(archetype_idx).len(),
            ))
        }
    }

    struct ComponentIter<'a, C> {
        components: Option<IterMut<'a, C>>,
        item_positions: Range<usize>,
    }

    impl<'a, C> ComponentIter<'a, C> {
        fn new(components: Option<IterMut<'a, C>>, entity_count: usize) -> Self {
            Self {
                components,
                item_positions: 0..entity_count,
            }
        }
    }

    impl<'a, C> Iterator for ComponentIter<'a, C> {
        type Item = Option<&'a mut C>;

        #[inline]
        fn next(&mut self) -> Option<Self::Item> {
            self.item_positions
                .next()
                .map(|_| self.components.as_mut().and_then(IterMut::next))
        }
    }

    impl<'a, C> DoubleEndedIterator for ComponentIter<'a, C> {
        #[inline]
        fn next_back(&mut self) -> Option<Self::Item> {
            self.item_positions
                .next()
                .map(|_| self.components.as_mut().and_then(IterMut::next_back))
        }
    }
}
