use crate::storages::archetypes::EntityLocation;
use crate::storages::core::CoreStorage;
use crate::storages::systems::SystemProperties;
use crate::system_params::internal::{QuerySystemParamWithLifetime, SystemParamWithLifetime};
use crate::{SystemData, SystemInfo};

pub(crate) mod components;
pub(crate) mod components_mut;
pub(crate) mod entity;
pub(crate) mod optional_components;
pub(crate) mod optional_components_mut;
pub(crate) mod queries;
pub(crate) mod tuples;
pub(crate) mod world;

/// A trait implemented for valid system parameters.
pub trait SystemParam: for<'a> SystemParamWithLifetime<'a> {
    #[doc(hidden)]
    type Tuple: SystemParam;
    #[doc(hidden)]
    type InnerTuple: SystemParam;

    #[doc(hidden)]
    fn properties(core: &mut CoreStorage) -> SystemProperties;

    #[doc(hidden)]
    fn lock<'a>(
        data: SystemData<'a>,
        info: SystemInfo<'a>,
    ) -> <Self as SystemParamWithLifetime<'a>>::Guard;

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
