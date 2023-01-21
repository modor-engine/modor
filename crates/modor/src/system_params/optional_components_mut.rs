use crate::optional_components_mut::internal::{
    ComponentMutOptionGuard, ComponentMutOptionGuardBorrow,
};
use crate::storages::archetypes::EntityLocation;
use crate::storages::core::CoreStorage;
use crate::storages::systems::{Access, ComponentTypeAccess, SystemProperties};
use crate::system_params::internal::{
    LockableSystemParam, Mut, QuerySystemParamWithLifetime, SystemParamWithLifetime,
};
use crate::system_params::optional_components::internal::ComponentOptionIter;
use crate::system_params::optional_components_mut::internal::ComponentMutOptionIter;
use crate::system_params::utils;
use crate::systems::context::SystemContext;
use crate::{Component, QuerySystemParam, SystemParam};

impl<'a, C> SystemParamWithLifetime<'a> for Option<&mut C>
where
    C: Component,
{
    type Param = Option<&'a mut C>;
    type Guard = ComponentMutOptionGuard<'a, C>;
    type GuardBorrow = ComponentMutOptionGuardBorrow<'a, C>;
    type Stream = ComponentMutOptionIter<'a, C>;
}

impl<C> SystemParam for Option<&mut C>
where
    C: Component,
{
    type Filter = ();
    type InnerTuple = ();

    fn properties(core: &mut CoreStorage) -> SystemProperties {
        let type_idx = core.register_component_type::<C>();
        SystemProperties {
            component_types: vec![ComponentTypeAccess {
                access: Access::Write,
                type_idx,
            }],
            can_update: false,
            mutation_component_type_idxs: vec![],
        }
    }

    fn lock(context: SystemContext<'_>) -> <Self as SystemParamWithLifetime<'_>>::Guard {
        ComponentMutOptionGuard::new(context)
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
    C: Component,
{
    type ConstParam = Option<&'a C>;
    type Iter = ComponentOptionIter<'a, C>;
    type IterMut = ComponentMutOptionIter<'a, C>;
}

impl<C> QuerySystemParam for Option<&mut C>
where
    C: Component,
{
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
    C: Component,
{
    type LockedType = C;
    type Mutability = Mut;
}

pub(crate) mod internal {
    use crate::storages::archetypes::{ArchetypeEntityPos, ArchetypeIdx};
    use crate::storages::components::{ComponentArchetypes, ComponentTypeIdx};
    use crate::systems::context::SystemContext;
    use crate::systems::iterations::FilteredArchetypeIdxIter;
    use crate::Component;
    use std::iter::Flatten;
    use std::ops::Range;
    use std::slice::IterMut;
    use std::sync::RwLockWriteGuard;
    use typed_index_collections::TiVec;

    pub struct ComponentMutOptionGuard<'a, C> {
        components: RwLockWriteGuard<'a, ComponentArchetypes<C>>,
        context: SystemContext<'a>,
    }

    impl<'a, C> ComponentMutOptionGuard<'a, C>
    where
        C: Component,
    {
        pub(crate) fn new(context: SystemContext<'a>) -> Self {
            Self {
                components: context.storages.components.write_components::<C>(),
                context,
            }
        }

        pub(crate) fn borrow(&mut self) -> ComponentMutOptionGuardBorrow<'_, C> {
            ComponentMutOptionGuardBorrow {
                components: &mut *self.components,
                item_count: self.context.item_count,
                sorted_archetype_idxs: self.context.filter_archetype_idx_iter(),
                context: self.context,
            }
        }
    }

    pub struct ComponentMutOptionGuardBorrow<'a, C> {
        pub(crate) components: &'a mut ComponentArchetypes<C>,
        pub(crate) item_count: usize,
        pub(crate) sorted_archetype_idxs: FilteredArchetypeIdxIter<'a>,
        pub(crate) context: SystemContext<'a>,
    }

    pub struct ComponentMutOptionIter<'a, C> {
        components: Flatten<ArchetypeComponentIter<'a, C>>,
        len: usize,
    }

    impl<'a, C> ComponentMutOptionIter<'a, C>
    where
        C: Component,
    {
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
        type_idx: ComponentTypeIdx,
        context: SystemContext<'a>,
    }

    impl<'a, C> ArchetypeComponentIter<'a, C>
    where
        C: Component,
    {
        fn new(guard: &'a mut ComponentMutOptionGuardBorrow<'_, C>) -> Self {
            Self {
                last_archetype_idx: None,
                components: guard.components.iter_mut(),
                sorted_archetype_idxs: guard.sorted_archetype_idxs.clone(),
                type_idx: guard.context.component_type_idx::<C>(),
                context: guard.context,
            }
        }
    }

    impl<'a, C> Iterator for ArchetypeComponentIter<'a, C> {
        type Item = ComponentIter<'a, C>;

        fn next(&mut self) -> Option<Self::Item> {
            let archetype_idx = self.sorted_archetype_idxs.next()?;
            self.context
                .add_mutated_component(self.type_idx, archetype_idx);
            let nth = usize::from(archetype_idx)
                - self.last_archetype_idx.map_or(0, |i| usize::from(i) + 1);
            self.last_archetype_idx = Some(archetype_idx);
            Some(ComponentIter::new(
                self.components.nth(nth).map(|c| c.iter_mut()),
                self.context
                    .storages
                    .archetypes
                    .entity_idxs(archetype_idx)
                    .len(),
            ))
        }
    }

    impl<'a, C> DoubleEndedIterator for ArchetypeComponentIter<'a, C> {
        fn next_back(&mut self) -> Option<Self::Item> {
            let archetype_idx = self.sorted_archetype_idxs.next_back()?;
            let nth_back = self
                .components
                .len()
                .checked_sub(usize::from(archetype_idx))
                .and_then(|n| n.checked_sub(1));
            Some(ComponentIter::new(
                nth_back.and_then(|n| self.components.nth_back(n).map(|c| c.iter_mut())),
                self.context
                    .storages
                    .archetypes
                    .entity_idxs(archetype_idx)
                    .len(),
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
