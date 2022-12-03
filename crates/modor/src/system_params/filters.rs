use self::internal::FilterIter;
use super::internal::{QuerySystemParamWithLifetime, SystemParamWithLifetime};
use crate::entity::internal::{EntityGuard, EntityGuardBorrow};
use crate::storages::archetypes::EntityLocation;
use crate::storages::core::CoreStorage;
use crate::storages::systems::SystemProperties;
use crate::systems::context::SystemInfo;
use crate::{EntityFilter, QuerySystemParam, SystemParam};
use std::marker::PhantomData;

/// A system parameter for fitlering entities on which the system iterates.
///
/// # Examples
///
/// ```rust
/// # use modor::{Filter, With};
/// #
/// #[derive(Debug)]
/// struct Position(f32, f32);
///
/// #[derive(Debug)]
/// struct Velocity(f32, f32);
///
/// fn print_position(position: &Position, _filter: Filter<With<Velocity>>) {
///     println!("Entity with velocity has position {:?}", position)
/// }
/// ```
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Filter<F>(PhantomData<fn(F)>)
where
    F: EntityFilter;

impl<'a, F> SystemParamWithLifetime<'a> for Filter<F>
where
    F: EntityFilter,
{
    type Param = Self;
    type Guard = EntityGuard<'a>;
    type GuardBorrow = EntityGuardBorrow<'a>;
    type Stream = FilterIter<'a, F>;
}

impl<F> SystemParam for Filter<F>
where
    F: EntityFilter,
{
    type Filter = F;
    type InnerTuple = ();

    fn properties(_core: &mut CoreStorage) -> SystemProperties {
        SystemProperties {
            component_types: vec![],
            can_update: false,
        }
    }

    fn lock(info: SystemInfo<'_>) -> <Self as SystemParamWithLifetime<'_>>::Guard {
        EntityGuard::new(info)
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
        FilterIter::new(guard)
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

impl<'a, F> QuerySystemParamWithLifetime<'a> for Filter<F>
where
    F: EntityFilter,
{
    type ConstParam = Self;
    type Iter = FilterIter<'a, F>;
    type IterMut = FilterIter<'a, F>;
}

impl<F> QuerySystemParam for Filter<F>
where
    F: EntityFilter,
{
    fn query_iter<'a, 'b>(
        guard: &'a <Self as SystemParamWithLifetime<'b>>::GuardBorrow,
    ) -> <Self as QuerySystemParamWithLifetime<'a>>::Iter
    where
        'b: 'a,
    {
        FilterIter::new(guard)
    }

    fn query_iter_mut<'a, 'b>(
        guard: &'a mut <Self as SystemParamWithLifetime<'b>>::GuardBorrow,
    ) -> <Self as QuerySystemParamWithLifetime<'a>>::IterMut
    where
        'b: 'a,
    {
        FilterIter::new(guard)
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
            .info
            .storages
            .archetypes
            .entity_idxs(location.idx)
            .get(location.pos)
            .map(|_| Self(PhantomData))
    }

    #[inline]
    fn get_mut<'a, 'b>(
        guard: &'a mut <Self as SystemParamWithLifetime<'b>>::GuardBorrow,
        location: EntityLocation,
    ) -> Option<<Self as SystemParamWithLifetime<'a>>::Param>
    where
        'b: 'a,
    {
        Self::get(guard, location)
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

mod internal {
    use super::Filter;
    use crate::entity::internal::{EntityGuardBorrow, EntityIter};
    use crate::EntityFilter;
    use std::marker::PhantomData;

    pub struct FilterIter<'a, F> {
        inner: EntityIter<'a>,
        phantom: PhantomData<fn(F)>,
    }

    impl<'a, F> FilterIter<'a, F> {
        pub fn new(guard: &'a EntityGuardBorrow<'_>) -> Self {
            Self {
                inner: EntityIter::new(guard),
                phantom: PhantomData,
            }
        }
    }

    impl<'a, F> Iterator for FilterIter<'a, F>
    where
        F: EntityFilter,
    {
        type Item = Filter<F>;

        #[inline]
        fn next(&mut self) -> Option<Self::Item> {
            self.inner.next().map(|_| Filter(PhantomData))
        }

        fn size_hint(&self) -> (usize, Option<usize>) {
            self.inner.size_hint()
        }
    }

    impl<F> DoubleEndedIterator for FilterIter<'_, F>
    where
        F: EntityFilter,
    {
        #[inline]
        fn next_back(&mut self) -> Option<Self::Item> {
            self.inner.next().map(|_| Filter(PhantomData))
        }
    }

    impl<F> ExactSizeIterator for FilterIter<'_, F> where F: EntityFilter {}
}
