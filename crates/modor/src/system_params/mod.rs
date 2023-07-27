use crate::storages::archetypes::EntityLocation;
use crate::storages::core::CoreStorage;
use crate::storages::systems::SystemProperties;
use crate::system_params::internal::{QuerySystemParamWithLifetime, SystemParamWithLifetime};
use crate::systems::context::SystemContext;
use crate::EntityFilter;

/// A trait implemented for valid system parameters.
pub trait SystemParam: for<'a> SystemParamWithLifetime<'a> {
    #[doc(hidden)]
    type Filter: EntityFilter;
    #[doc(hidden)]
    type InnerTuple: SystemParam;

    #[doc(hidden)]
    fn properties(core: &mut CoreStorage) -> SystemProperties;

    #[doc(hidden)]
    fn lock(context: SystemContext<'_>) -> <Self as SystemParamWithLifetime<'_>>::Guard;

    #[doc(hidden)]
    fn borrow_guard<'a, 'b>(
        guard: &'a mut <Self as SystemParamWithLifetime<'b>>::Guard,
    ) -> <Self as SystemParamWithLifetime<'a>>::GuardBorrow
    where
        'b: 'a;

    #[doc(hidden)]
    fn stream<'a, 'b>(
        guard: &'a mut <Self as SystemParamWithLifetime<'b>>::GuardBorrow,
    ) -> <Self as SystemParamWithLifetime<'a>>::Stream
    where
        'b: 'a;

    #[doc(hidden)]
    fn stream_next<'a, 'b>(
        stream: &'a mut <Self as SystemParamWithLifetime<'b>>::Stream,
    ) -> Option<<Self as SystemParamWithLifetime<'a>>::Param>
    where
        'b: 'a;
}

/// A trait implemented for valid [`Query`](crate::Query) parameters.
pub trait QuerySystemParam: SystemParam + for<'a> QuerySystemParamWithLifetime<'a> {
    #[doc(hidden)]
    fn query_iter<'a, 'b>(
        guard: &'a <Self as SystemParamWithLifetime<'b>>::GuardBorrow,
    ) -> <Self as QuerySystemParamWithLifetime<'a>>::Iter
    where
        'b: 'a;

    #[doc(hidden)]
    fn query_iter_mut<'a, 'b>(
        guard: &'a mut <Self as SystemParamWithLifetime<'b>>::GuardBorrow,
    ) -> <Self as QuerySystemParamWithLifetime<'a>>::IterMut
    where
        'b: 'a;

    #[doc(hidden)]
    fn get<'a, 'b>(
        guard: &'a <Self as SystemParamWithLifetime<'b>>::GuardBorrow,
        location: EntityLocation,
    ) -> Option<<Self as QuerySystemParamWithLifetime<'a>>::ConstParam>
    where
        'b: 'a;

    #[doc(hidden)]
    fn get_mut<'a, 'b>(
        guard: &'a mut <Self as SystemParamWithLifetime<'b>>::GuardBorrow,
        location: EntityLocation,
    ) -> Option<<Self as SystemParamWithLifetime<'a>>::Param>
    where
        'b: 'a;

    #[doc(hidden)]
    fn get_both_mut<'a, 'b>(
        guard: &'a mut <Self as SystemParamWithLifetime<'b>>::GuardBorrow,
        location1: EntityLocation,
        location2: EntityLocation,
    ) -> (
        Option<<Self as SystemParamWithLifetime<'a>>::Param>,
        Option<<Self as SystemParamWithLifetime<'a>>::Param>,
    )
    where
        'b: 'a;
}

pub(crate) mod internal {
    use crate::SystemParam;
    use std::any::Any;

    pub trait SystemParamWithLifetime<'a> {
        type Param: 'a;
        type Guard: 'a;
        type GuardBorrow: 'a;
        type Stream: 'a;
    }

    pub trait QuerySystemParamWithLifetime<'a>: SystemParamWithLifetime<'a> {
        type ConstParam: 'a + SystemParamWithLifetime<'a>;
        type Iter: 'a
            + Sync
            + Send
            + Iterator<Item = <Self::ConstParam as SystemParamWithLifetime<'a>>::Param>
            + DoubleEndedIterator
            + ExactSizeIterator;
        type IterMut: 'a
            + Sync
            + Send
            + Iterator<Item = <Self as SystemParamWithLifetime<'a>>::Param>
            + DoubleEndedIterator
            + ExactSizeIterator;
    }

    pub trait LockableSystemParam: SystemParam {
        type LockedType: Any;
        type Mutability: Mutability;
    }

    #[allow(unreachable_pub)]
    pub trait Mutability {}

    pub struct Const;

    impl Mutability for Const {}

    pub struct Mut;

    impl Mutability for Mut {}
}

pub(crate) mod utils {
    use crate::storages::archetypes::{ArchetypeEntityPos, ArchetypeIdx, EntityLocation};
    use typed_index_collections::TiVec;

    pub(crate) fn get_both_mut<T>(
        data: &mut TiVec<ArchetypeIdx, TiVec<ArchetypeEntityPos, T>>,
        location1: EntityLocation,
        location2: EntityLocation,
    ) -> (Option<&mut T>, Option<&mut T>) {
        if location1.idx == location2.idx {
            data.get_mut(location1.idx).map_or((None, None), |c| {
                get_both_mut_internal(c, location1.pos, location2.pos)
            })
        } else {
            let (sub_data1, sub_data2) = get_both_mut_internal(data, location1.idx, location2.idx);
            (
                sub_data1.and_then(|d| d.get_mut(location1.pos)),
                sub_data2.and_then(|d| d.get_mut(location2.pos)),
            )
        }
    }

    fn get_both_mut_internal<K, T>(
        data: &mut TiVec<K, T>,
        key1: K,
        key2: K,
    ) -> (Option<&mut T>, Option<&mut T>)
    where
        K: Ord + From<usize> + Copy,
        usize: From<K>,
    {
        if key2 >= data.next_key() {
            (data.get_mut(key1), None)
        } else if key1 >= data.next_key() {
            (None, data.get_mut(key2))
        } else if key1 > key2 {
            let (left, right) = data.split_at_mut(key1);
            (Some(&mut right[K::from(0)]), Some(&mut left[key2]))
        } else {
            let (left, right) = data.split_at_mut(key2);
            (Some(&mut left[key1]), Some(&mut right[K::from(0)]))
        }
    }
}

pub(crate) mod components;
pub(crate) mod components_mut;
pub(crate) mod entity;
pub(crate) mod entity_mut;
pub(crate) mod filter;
pub(crate) mod optional_components;
pub(crate) mod optional_components_mut;
pub(crate) mod optional_singleton;
pub(crate) mod query;
pub(crate) mod singleton;
pub(crate) mod tuples;
pub(crate) mod world;
