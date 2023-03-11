use crate::storages::archetypes::EntityLocation;
use crate::storages::core::CoreStorage;
use crate::storages::systems::{Access, ComponentTypeAccess, SystemProperties};
use crate::system_params::components::internal::{
    ComponentGuard, ComponentGuardBorrow, ComponentIter,
};
use crate::system_params::internal::{
    Const, LockableSystemParam, QuerySystemParamWithLifetime, SystemParamWithLifetime,
};
use crate::systems::context::SystemContext;
use crate::{Component, QuerySystemParam, SystemParam, With};

impl<'a, C> SystemParamWithLifetime<'a> for &C
where
    C: Component,
{
    type Param = &'a C;
    type Guard = ComponentGuard<'a, C>;
    type GuardBorrow = ComponentGuardBorrow<'a, C>;
    type Stream = ComponentIter<'a, C>;
}

impl<C> SystemParam for &C
where
    C: Component,
{
    type Filter = With<C>;
    type InnerTuple = ();

    fn properties(core: &mut CoreStorage) -> SystemProperties {
        let type_idx = core.register_component_type::<C>();
        SystemProperties {
            component_types: vec![ComponentTypeAccess {
                access: Access::Read,
                type_idx,
            }],
            can_update: false,
            mutation_component_type_idxs: vec![],
        }
    }

    fn lock(context: SystemContext<'_>) -> <Self as SystemParamWithLifetime<'_>>::Guard {
        ComponentGuard::new(context)
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
    C: Component,
{
    type ConstParam = &'a C;
    type Iter = ComponentIter<'a, C>;
    type IterMut = ComponentIter<'a, C>;
}

impl<C> QuerySystemParam for &C
where
    C: Component,
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
            .get(location.idx)
            .and_then(|a| a.get(location.pos))
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
        (Self::get(guard, location1), Self::get(guard, location2))
    }
}

impl<C> LockableSystemParam for &C
where
    C: Component,
{
    type LockedType = C;
    type Mutability = Const;
}

pub(crate) mod internal {
    use crate::components_mut::internal::ComponentMutGuardBorrow;
    use crate::storages::archetypes::{ArchetypeEntityPos, ArchetypeIdx};
    use crate::storages::components::ComponentArchetypes;
    use crate::systems::context::SystemContext;
    use crate::systems::iterations::FilteredArchetypeIdxIter;
    use crate::Component;
    use std::iter::Flatten;
    use std::slice::Iter;
    use std::sync::RwLockReadGuard;
    use typed_index_collections::TiVec;

    pub struct ComponentGuard<'a, C> {
        components: RwLockReadGuard<'a, ComponentArchetypes<C>>,
        context: SystemContext<'a>,
    }

    impl<'a, C> ComponentGuard<'a, C>
    where
        C: Component,
    {
        pub(crate) fn new(context: SystemContext<'a>) -> Self {
            Self {
                components: context.storages.components.read_components::<C>(),
                context,
            }
        }

        pub(crate) fn borrow(&mut self) -> ComponentGuardBorrow<'_, C> {
            ComponentGuardBorrow {
                components: &*self.components,
                item_count: self.context.item_count,
                sorted_archetype_idxs: self.context.filter_archetype_idx_iter(),
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
