use crate::optional_components::internal::{ComponentOptionGuard, ComponentOptionGuardBorrow};
use crate::storages::archetypes::EntityLocation;
use crate::storages::core::CoreStorage;
use crate::storages::systems::{Access, ComponentTypeAccess, SystemProperties};
use crate::system_params::internal::{Const, LockableSystemParam};
use crate::system_params::optional_components::internal::ComponentOptionIter;
use crate::system_params::query::internal::QueryFilterProperties;
use crate::systems::context::SystemContext;
use crate::{
    Component, ConstSystemParam, QuerySystemParam, QuerySystemParamWithLifetime, SystemParam,
    SystemParamWithLifetime,
};
use std::any;

impl<'a, C> SystemParamWithLifetime<'a> for Option<&C>
where
    C: Component,
{
    type Param = Option<&'a C>;
    type Guard = ComponentOptionGuard<'a, C>;
    type GuardBorrow = ComponentOptionGuardBorrow<'a, C>;
    type Stream = ComponentOptionIter<'a, C>;
}

impl<C> SystemParam for Option<&C>
where
    C: Component,
{
    type Filter = ();
    type InnerTuple = ();

    fn properties(core: &mut CoreStorage) -> SystemProperties {
        let type_idx = core.register_component_type::<C>();
        SystemProperties {
            component_types: vec![ComponentTypeAccess {
                access: Access::Read,
                type_idx,
                type_name: any::type_name::<C>(),
            }],
            can_update: false,
            mutation_component_type_idxs: vec![],
        }
    }

    fn lock(context: SystemContext<'_>) -> <Self as SystemParamWithLifetime<'_>>::Guard {
        ComponentOptionGuard::new(context)
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
        ComponentOptionIter::new(guard, None)
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
    C: Component,
{
    type ConstParam = Option<&'a C>;
    type Iter = ComponentOptionIter<'a, C>;
    type IterMut = ComponentOptionIter<'a, C>;
}

impl<C> QuerySystemParam for Option<&C>
where
    C: Component,
{
    fn query_iter<'a, 'b>(
        guard: &'a <Self as SystemParamWithLifetime<'b>>::GuardBorrow,
        filter: Option<QueryFilterProperties>,
    ) -> <Self as QuerySystemParamWithLifetime<'a>>::Iter
    where
        'b: 'a,
    {
        ComponentOptionIter::new(guard, filter)
    }

    fn query_iter_mut<'a, 'b>(
        guard: &'a mut <Self as SystemParamWithLifetime<'b>>::GuardBorrow,
        filter: Option<QueryFilterProperties>,
    ) -> <Self as QuerySystemParamWithLifetime<'a>>::IterMut
    where
        'b: 'a,
    {
        ComponentOptionIter::new(guard, filter)
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

impl<C> LockableSystemParam for Option<&C>
where
    C: Component,
{
    type LockedType = C;
    type Mutability = Const;
}

impl<C> ConstSystemParam for Option<&C> where C: Component {}

pub(crate) mod internal {
    use crate::optional_components_mut::internal::ComponentMutOptionGuardBorrow;
    use crate::storages::archetypes::{ArchetypeEntityPos, ArchetypeIdx};
    use crate::storages::components::ComponentArchetypes;
    use crate::system_params::query::internal::QueryFilterProperties;
    use crate::systems::context::SystemContext;
    use crate::systems::iterations::FilteredArchetypeIdxIter;
    use crate::Component;
    use std::iter::Flatten;
    use std::ops::Range;
    use std::slice::Iter;
    use std::sync::RwLockReadGuard;
    use typed_index_collections::TiVec;

    pub struct ComponentOptionGuard<'a, C> {
        components: RwLockReadGuard<'a, ComponentArchetypes<C>>,
        context: SystemContext<'a>,
    }

    impl<'a, C> ComponentOptionGuard<'a, C>
    where
        C: Component,
    {
        pub(crate) fn new(context: SystemContext<'a>) -> Self {
            Self {
                components: context.storages.components.read_components::<C>(),
                context,
            }
        }

        pub(crate) fn borrow(&mut self) -> ComponentOptionGuardBorrow<'_, C> {
            ComponentOptionGuardBorrow {
                components: &*self.components,
                item_count: self.context.item_count,
                sorted_archetype_idxs: self.context.filter_archetype_idx_iter(),
                context: self.context,
            }
        }
    }

    pub struct ComponentOptionGuardBorrow<'a, C> {
        pub(crate) components: &'a ComponentArchetypes<C>,
        pub(crate) item_count: usize,
        pub(crate) sorted_archetype_idxs: FilteredArchetypeIdxIter<'a>,
        pub(crate) context: SystemContext<'a>,
    }

    pub struct ComponentOptionIter<'a, C> {
        components: Flatten<ArchetypeComponentIter<'a, C>>,
        len: usize,
    }

    impl<'a, C> ComponentOptionIter<'a, C> {
        pub(crate) fn new(
            guard: &'a ComponentOptionGuardBorrow<'_, C>,
            filter: Option<QueryFilterProperties>,
        ) -> Self {
            Self {
                components: ArchetypeComponentIter::new(guard, filter).flatten(),
                len: if let Some(filter) = filter {
                    filter.item_count
                } else {
                    guard.item_count
                },
            }
        }

        pub(crate) fn new_mut(
            guard: &'a ComponentMutOptionGuardBorrow<'_, C>,
            filter: Option<QueryFilterProperties>,
        ) -> Self {
            Self {
                components: ArchetypeComponentIter::new_mut(guard, filter).flatten(),
                len: if let Some(filter) = filter {
                    filter.item_count
                } else {
                    guard.item_count
                },
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
        context: SystemContext<'a>,
    }

    impl<'a, C> ArchetypeComponentIter<'a, C> {
        fn new(
            guard: &'a ComponentOptionGuardBorrow<'_, C>,
            filter: Option<QueryFilterProperties>,
        ) -> Self {
            Self {
                last_archetype_idx: None,
                components: guard.components.iter(),
                sorted_archetype_idxs: guard.sorted_archetype_idxs.clone_with_filter(filter),
                context: guard.context,
            }
        }

        fn new_mut(
            guard: &'a ComponentMutOptionGuardBorrow<'_, C>,
            filter: Option<QueryFilterProperties>,
        ) -> Self {
            Self {
                last_archetype_idx: None,
                components: guard.components.iter(),
                sorted_archetype_idxs: guard.sorted_archetype_idxs.clone_with_filter(filter),
                context: guard.context,
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
                nth_back.and_then(|n| self.components.nth_back(n).map(|c| c.iter())),
                self.context
                    .storages
                    .archetypes
                    .entity_idxs(archetype_idx)
                    .len(),
            ))
        }
    }

    struct ComponentIter<'a, C> {
        components: Option<Iter<'a, C>>,
        item_positions: Range<usize>,
    }

    impl<'a, C> ComponentIter<'a, C> {
        fn new(components: Option<Iter<'a, C>>, entity_count: usize) -> Self {
            Self {
                components,
                item_positions: 0..entity_count,
            }
        }
    }

    impl<'a, C> Iterator for ComponentIter<'a, C> {
        type Item = Option<&'a C>;

        #[inline]
        fn next(&mut self) -> Option<Self::Item> {
            self.item_positions
                .next()
                .map(|_| self.components.as_mut().and_then(Iter::next))
        }
    }

    impl<'a, C> DoubleEndedIterator for ComponentIter<'a, C> {
        #[inline]
        fn next_back(&mut self) -> Option<Self::Item> {
            self.item_positions
                .next()
                .map(|_| self.components.as_mut().and_then(Iter::next_back))
        }
    }
}
