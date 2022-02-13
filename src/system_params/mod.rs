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
    use crate::utils;
    use typed_index_collections::TiVec;

    pub(crate) fn get_both_mut<T>(
        data: &mut TiVec<ArchetypeIdx, TiVec<ArchetypeEntityPos, T>>,
        location1: EntityLocation,
        location2: EntityLocation,
    ) -> (Option<&mut T>, Option<&mut T>) {
        if location1.idx == location2.idx {
            if location1.idx >= data.next_key() {
                (None, None)
            } else {
                utils::get_both_mut(&mut data[location1.idx], location1.pos, location2.pos)
            }
        } else {
            let (sub_data1, sub_data2) = utils::get_both_mut(data, location1.idx, location2.idx);
            (
                sub_data1.and_then(|d| d.get_mut(location1.pos)),
                sub_data2.and_then(|d| d.get_mut(location2.pos)),
            )
        }
    }
}

#[cfg(test)]
mod system_param_utils_tests {
    use crate::storages::archetypes::EntityLocation;
    use crate::system_params::utils::get_both_mut;

    #[test]
    fn retrieve_both_mut() {
        let mut data = ti_vec![ti_vec![10_u32, 20], ti_vec![30, 40]];
        let location1 = EntityLocation::new(0.into(), 1.into());
        let location2 = EntityLocation::new(1.into(), 0.into());
        let location3 = EntityLocation::new(1.into(), 1.into());
        let wrong_idx_location = EntityLocation::new(2.into(), 1.into());
        let wrong_pos_location = EntityLocation::new(0.into(), 2.into());
        let result = get_both_mut(&mut data, location1, location2);
        assert_eq!(result, (Some(&mut 20), Some(&mut 30)));
        let result = get_both_mut(&mut data, location2, location1);
        assert_eq!(result, (Some(&mut 30), Some(&mut 20)));
        let result = get_both_mut(&mut data, location2, location3);
        assert_eq!(result, (Some(&mut 30), Some(&mut 40)));
        let result = get_both_mut(&mut data, location3, location2);
        assert_eq!(result, (Some(&mut 40), Some(&mut 30)));
        let result = get_both_mut(&mut data, location1, location1);
        assert_eq!(result, (Some(&mut 20), None));
        let result = get_both_mut(&mut data, location1, wrong_idx_location);
        assert_eq!(result, (Some(&mut 20), None));
        let result = get_both_mut(&mut data, wrong_pos_location, location1);
        assert_eq!(result, (None, Some(&mut 20)));
        let result = get_both_mut(&mut data, wrong_idx_location, wrong_idx_location);
        assert_eq!(result, (None, None));
    }
}
