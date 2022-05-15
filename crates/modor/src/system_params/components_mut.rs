use crate::components_mut::internal::{ComponentMutGuard, ComponentMutGuardBorrow};
use crate::storages::archetypes::EntityLocation;
use crate::storages::components::ComponentTypeIdx;
use crate::storages::core::CoreStorage;
use crate::storages::systems::{Access, ComponentTypeAccess, SystemProperties};
use crate::system_params::components::internal::ComponentIter;
use crate::system_params::components_mut::internal::ComponentMutIter;
use crate::system_params::internal::{
    LockableSystemParam, Mut, QuerySystemParamWithLifetime, SystemParamWithLifetime,
};
use crate::system_params::utils;
use crate::{QuerySystemParam, SystemData, SystemInfo, SystemParam};
use std::any::{Any, TypeId};

impl<'a, C> SystemParamWithLifetime<'a> for &mut C
where
    C: Any + Sync + Send,
{
    type Param = &'a mut C;
    type Guard = ComponentMutGuard<'a, C>;
    type GuardBorrow = ComponentMutGuardBorrow<'a, C>;
    type Stream = ComponentMutIter<'a, C>;
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
            can_update: false,
            filtered_component_type_idxs: vec![type_idx],
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
        ComponentMutIter::new(guard)
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
    type IterMut = ComponentMutIter<'a, C>;
}

impl<C> QuerySystemParam for &mut C
where
    C: Any + Sync + Send,
{
    fn filtered_component_type_idxs(data: SystemData<'_>) -> Vec<ComponentTypeIdx> {
        vec![data
            .components
            .type_idx(TypeId::of::<C>())
            .expect("internal error: component type not registered")]
    }

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
        ComponentMutIter::new(guard)
    }

    #[inline]
    fn get<'a, 'b>(
        guard: &'a <Self as SystemParamWithLifetime<'b>>::GuardBorrow,
        location: EntityLocation,
    ) -> Option<<Self as QuerySystemParamWithLifetime<'a>>::ConstParam>
    where
        'b: 'a,
    {
        guard
            .components
            .get(location.idx)
            .and_then(|a| a.get(location.pos))
    }

    #[inline]
    fn get_mut<'a, 'b>(
        guard: &'a mut <Self as SystemParamWithLifetime<'b>>::GuardBorrow,
        location: EntityLocation,
    ) -> Option<<Self as SystemParamWithLifetime<'a>>::Param>
    where
        'b: 'a,
    {
        guard
            .components
            .get_mut(location.idx)
            .and_then(|a| a.get_mut(location.pos))
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
        utils::get_both_mut(guard.components, location1, location2)
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
                item_count: self.info.item_count,
                sorted_archetype_idxs: self
                    .data
                    .filter_archetype_idx_iter(self.info.filtered_component_type_idxs),
            }
        }
    }

    pub struct ComponentMutGuardBorrow<'a, C> {
        pub(crate) components: &'a mut ComponentArchetypes<C>,
        pub(crate) item_count: usize,
        pub(crate) sorted_archetype_idxs: FilteredArchetypeIdxIter<'a>,
    }

    pub struct ComponentMutIter<'a, C> {
        components: Flatten<ArchetypeComponentIter<'a, C>>,
        len: usize,
    }

    impl<'a, C> ComponentMutIter<'a, C> {
        pub(super) fn new(guard: &'a mut ComponentMutGuardBorrow<'_, C>) -> Self {
            Self {
                len: guard.item_count,
                components: ArchetypeComponentIter::new(guard).flatten(),
            }
        }
    }

    impl<'a, C> Iterator for ComponentMutIter<'a, C> {
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

    impl<'a, C> DoubleEndedIterator for ComponentMutIter<'a, C> {
        #[inline]
        fn next_back(&mut self) -> Option<Self::Item> {
            self.components.next_back().map(|c| {
                self.len -= 1;
                c
            })
        }
    }

    impl<'a, C> ExactSizeIterator for ComponentMutIter<'a, C> {}

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
