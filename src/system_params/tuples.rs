use crate::storages::archetypes::{ArchetypeFilter, EntityLocation};
use crate::storages::core::CoreStorage;
use crate::storages::systems::SystemProperties;
use crate::system_params::internal::{QuerySystemParamWithLifetime, SystemParamWithLifetime};
use crate::system_params::tuples::internal::{EmptyTupleGuard, EmptyTupleIter};
use crate::tuples::internal::EmptyTupleGuardBorrow;
use crate::{QuerySystemParam, SystemData, SystemInfo, SystemParam};
use std::iter::{Map, Zip};

impl<'a> SystemParamWithLifetime<'a> for () {
    type Param = ();
    type Guard = EmptyTupleGuard;
    type GuardBorrow = EmptyTupleGuardBorrow;
    type Stream = EmptyTupleIter;
}

impl SystemParam for () {
    type Tuple = Self;
    type InnerTuple = Self;

    fn properties(_core: &mut CoreStorage) -> SystemProperties {
        SystemProperties {
            component_types: vec![],
            globals: vec![],
            can_update: false,
            archetype_filter: ArchetypeFilter::None,
        }
    }

    fn lock<'a>(
        data: SystemData<'a>,
        info: SystemInfo<'a>,
    ) -> <Self as SystemParamWithLifetime<'a>>::Guard {
        EmptyTupleGuard::new(data, info)
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
        EmptyTupleIter::new(guard)
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

impl<'a> QuerySystemParamWithLifetime<'a> for () {
    type ConstParam = ();
    type Iter = EmptyTupleIter;
    type IterMut = EmptyTupleIter;
}

impl QuerySystemParam for () {
    fn query_iter<'a, 'b>(
        guard: &'a <Self as SystemParamWithLifetime<'b>>::GuardBorrow,
    ) -> <Self as QuerySystemParamWithLifetime<'a>>::Iter
    where
        'b: 'a,
    {
        EmptyTupleIter::new(guard)
    }

    fn query_iter_mut<'a, 'b>(
        guard: &'a mut <Self as SystemParamWithLifetime<'b>>::GuardBorrow,
    ) -> <Self as QuerySystemParamWithLifetime<'a>>::IterMut
    where
        'b: 'a,
    {
        EmptyTupleIter::new(guard)
    }

    #[inline]
    fn get<'a, 'b>(
        _guard: &'a <Self as SystemParamWithLifetime<'b>>::GuardBorrow,
        _location: EntityLocation,
    ) -> Option<<Self as QuerySystemParamWithLifetime<'a>>::ConstParam>
    where
        'b: 'a,
    {
        Some(())
    }

    #[inline]
    fn get_mut<'a, 'b>(
        _guard: &'a mut <Self as SystemParamWithLifetime<'b>>::GuardBorrow,
        _location: EntityLocation,
    ) -> Option<<Self as SystemParamWithLifetime<'a>>::Param>
    where
        'b: 'a,
    {
        Some(())
    }

    #[inline]
    fn get_both_mut<'a, 'b>(
        _guard: &'a mut <Self as SystemParamWithLifetime<'b>>::GuardBorrow,
        _location1: EntityLocation,
        _location2: EntityLocation,
    ) -> (
        Option<<Self as SystemParamWithLifetime<'a>>::Param>,
        Option<<Self as SystemParamWithLifetime<'a>>::Param>,
    )
    where
        'b: 'a,
    {
        (Some(()), Some(()))
    }
}

macro_rules! impl_tuple_system_param {
    ($(($params:ident, $indexes:tt)),+) => {
        impl<'a $(,$params)+> SystemParamWithLifetime<'a> for ($($params,)+)
        where
            $($params: SystemParamWithLifetime<'a>,)+
        {
            type Param = ($($params::Param,)+);
            type Guard = ($($params::Guard,)+);
            type GuardBorrow = ($($params::GuardBorrow,)+);
            type Stream = ($($params::Stream,)+);
        }

        impl<$($params),+> SystemParam for ($($params,)+)
        where
            $($params: SystemParam,)+
        {
            type Tuple = Self;
            type InnerTuple = Self;

            fn properties(core: &mut CoreStorage) -> SystemProperties {
                let properties = ($($params::properties(core),)+);
                let mut component_types = Vec::new();
                $(component_types.extend(properties.$indexes.component_types);)+
                let mut globals = Vec::new();
                $(globals.extend(properties.$indexes.globals);)+
                SystemProperties {
                    component_types,
                    globals,
                    can_update: [$(properties.$indexes.can_update),+].into_iter().any(|b| b),
                    archetype_filter:
                        ArchetypeFilter::None $(.merge(properties.$indexes.archetype_filter))+
                }
            }

            fn lock<'a>(
                data: SystemData<'a>,
                info: SystemInfo<'a>,
            ) -> <Self as SystemParamWithLifetime<'a>>::Guard {
                ($($params::lock(data, info),)+)
            }

            fn borrow_guard<'a, 'b>(
                guard: &'a mut <Self as SystemParamWithLifetime<'b>>::Guard,
            ) -> <Self as SystemParamWithLifetime<'a>>::GuardBorrow
            where
                'b: 'a,
            {
                ($($params::borrow_guard(&mut guard.$indexes),)+)
            }

            fn stream<'a, 'b>(
                guard: &'a mut <Self as SystemParamWithLifetime<'b>>::GuardBorrow,
            ) -> <Self as SystemParamWithLifetime<'a>>::Stream
            where
                'b: 'a,
            {
                ($($params::stream(&mut guard.$indexes),)+)
            }

            #[inline]
            fn stream_next<'a, 'b>(
                stream: &'a mut <Self as SystemParamWithLifetime<'b>>::Stream,
            ) -> Option<<Self as SystemParamWithLifetime<'a>>::Param>
            where
                'b: 'a,
            {
                Some(($($params::stream_next(&mut stream.$indexes)?,)*))
            }
        }

        impl<'a $(,$params)+> QuerySystemParamWithLifetime<'a> for ($($params,)+)
        where
            $($params: QuerySystemParamWithLifetime<'a>,)+
        {
            type ConstParam = ($($params::ConstParam,)+);
            iter_type!($($params),+);
            iter_mut_type!($($params),+);
        }

        impl<$($params),+> QuerySystemParam for ($($params,)+)
        where
            $($params: QuerySystemParam,)+
        {
            #[allow(non_snake_case)]
            fn query_iter<'a, 'b>(
                guard: &'a <Self as SystemParamWithLifetime<'b>>::GuardBorrow,
            ) -> <Self as QuerySystemParamWithLifetime<'a>>::Iter
            where
                'b: 'a,
            {
                query_iter!(guard $(,($params, $indexes))+)
            }

            #[allow(non_snake_case)]
            fn query_iter_mut<'a, 'b>(
                guard: &'a mut <Self as SystemParamWithLifetime<'b>>::GuardBorrow,
            ) -> <Self as QuerySystemParamWithLifetime<'a>>::IterMut
            where
                'b: 'a,
            {
                query_iter_mut!(guard $(,($params, $indexes))+)
            }

            #[inline]
            fn get<'a, 'b>(
                guard: &'a <Self as SystemParamWithLifetime<'b>>::GuardBorrow,
                location: EntityLocation,
            ) -> Option<<Self as QuerySystemParamWithLifetime<'a>>::ConstParam>
            where
                'b: 'a,
            {
                Some(($($params::get(&guard.$indexes, location)?,)+))
            }

            #[inline]
            fn get_mut<'a, 'b>(
                guard: &'a mut <Self as SystemParamWithLifetime<'b>>::GuardBorrow,
                location: EntityLocation,
            ) -> Option<<Self as SystemParamWithLifetime<'a>>::Param>
            where
                'b: 'a,
            {
                Some(($($params::get_mut(&mut guard.$indexes, location)?,)+))
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
                let items = ($($params::get_both_mut(&mut guard.$indexes, location1, location2),)+);
                (
                    (move || {Some(($(items.$indexes.0?,)+))})(),
                    (move || {Some(($(items.$indexes.1?,)+))})(),
                )
            }
        }
    };
}

macro_rules! iter_type {
    ($param:ident) => {
        type Iter = Map<
            $param::Iter,
            fn(
                <$param::ConstParam as SystemParamWithLifetime<'a>>::Param
            ) -> (<$param::ConstParam as SystemParamWithLifetime<'a>>::Param,)
        >;
    };
    ($param1:ident, $param2:ident) => { type Iter = Zip<$param1::Iter, $param2::Iter>; };
    ($($params:ident),*) => {
        type Iter = Map<
            nested_zip_type!($($params::Iter),*),
            fn(
                nested_tuple_type!($(<$params::ConstParam as SystemParamWithLifetime<'a>>::Param),*)
            ) -> (
                $(<$params::ConstParam as SystemParamWithLifetime<'a>>::Param,)*
            )
        >;
    }
}

macro_rules! iter_mut_type {
    ($param:ident) => { type IterMut = Map<$param::IterMut, fn($param::Param) -> ($param::Param,)>; };
    ($param1:ident, $param2:ident) => { type IterMut = Zip<$param1::IterMut, $param2::IterMut>; };
    ($($params:ident),*) => {
        type IterMut = Map<
            nested_zip_type!($($params::IterMut),*),
            fn(nested_tuple_type!($($params::Param),*)) -> ($($params::Param,)*)
        >;
    }
}

macro_rules! nested_zip_type {
    ($($params:path),+) => {
        nested_zip_type!(@internal (), ($($params),+))
    };
    (@internal ($($lefts:path),*), ($right:path $(,$rights:path)+)) => {
        nested_zip_type!(@internal ($($lefts,)* $right), ($($rights),+))
    };
    (@internal ($($lefts:path),+), ($right:path)) => {
        Zip<nested_zip_type!(@internal (), ($($lefts),+)), $right>
    };
    (@internal (), ($right:path)) => {
        $right
    };
}

macro_rules! nested_tuple_type {
    ($($params:ty),*) => {
        nested_tuple_type!(@internal (), ($($params),*))
    };
    (@internal ($($lefts:ty),*), ($right:ty $(,$rights:ty)+)) => {
        nested_tuple_type!(@internal ($($lefts,)* $right), ($($rights),+))
    };
    (@internal ($($lefts:ty),+), ($right:ty)) => {
        (nested_tuple_type!(@internal (), ($($lefts),+)), $right)
    };
    (@internal (), ($right:ty)) => {
        $right
    };
    (@internal (), ()) => {
        ()
    };
}

macro_rules! query_iter {
    ($guard:expr, ($param:ident, $index:tt)) => {
        $param::query_iter(&$guard.$index).map(|i| (i,))
    };
    ($guard:expr, ($param1:ident, $index1:tt), ($param2:ident, $index2:tt)) => {
        $param1::query_iter(&$guard.$index1).zip($param2::query_iter(&$guard.$index2))
    };
    ($guard:expr, ($param:ident, $index:tt), $(($params:ident, $indexes:tt)),+) => {
        $param::query_iter(&$guard.$index)
            $(.zip($params::query_iter(&$guard.$indexes)))+
            .map(|nested_tuple!($param $(, $params)*)| ($param, $($params),*))
    };
}

macro_rules! query_iter_mut {
    ($guard:expr, ($param:ident, $index:tt)) => {
        $param::query_iter_mut(&mut $guard.$index).map(|i| (i,))
    };
    ($guard:expr, ($param1:ident, $index1:tt), ($param2:ident, $index2:tt)) => {
        $param1::query_iter_mut(&mut $guard.$index1)
            .zip($param2::query_iter_mut(&mut $guard.$index2))
    };
    ($guard:expr, ($param:ident, $index:tt), $(($params:ident, $indexes:tt)),+) => {
        $param::query_iter_mut(&mut $guard.$index)
            $(.zip($params::query_iter_mut(&mut $guard.$indexes)))+
            .map(|nested_tuple!($param $(, $params)*)| ($param, $($params),*))
    };
}

macro_rules! nested_tuple {
    ($($params:ident),+) => {
        nested_tuple!(@internal (), ($($params),+))
    };
    (@internal ($($lefts:ident),*), ($right:ident $(,$rights:ident)+)) => {
        nested_tuple!(@internal ($($lefts,)* $right), ($($rights),+))
    };
    (@internal ($($lefts:ident),+), ($right:ident)) => {
        (nested_tuple!(@internal (), ($($lefts),+)), $right)
    };
    (@internal (), ($right:ident)) => {
        $right
    };
    (@internal (), ()) => {
        ()
    };
}

run_for_tuples_with_idxs!(impl_tuple_system_param);

mod internal {
    use crate::{SystemData, SystemInfo};
    use std::ops::Range;

    pub struct EmptyTupleGuard {
        item_count: usize,
    }

    impl EmptyTupleGuard {
        pub(crate) fn new(_data: SystemData<'_>, info: SystemInfo<'_>) -> Self {
            Self {
                item_count: info.item_count,
            }
        }

        pub(crate) fn borrow(&mut self) -> EmptyTupleGuardBorrow {
            EmptyTupleGuardBorrow {
                item_count: self.item_count,
            }
        }
    }

    pub struct EmptyTupleGuardBorrow {
        pub(crate) item_count: usize,
    }

    pub struct EmptyTupleIter {
        item_positions: Range<usize>,
    }

    impl EmptyTupleIter {
        pub(crate) fn new(guard: &EmptyTupleGuardBorrow) -> Self {
            Self {
                item_positions: 0..guard.item_count,
            }
        }
    }

    impl Iterator for EmptyTupleIter {
        type Item = ();

        fn next(&mut self) -> Option<Self::Item> {
            self.item_positions.next().map(|_| ())
        }

        fn size_hint(&self) -> (usize, Option<usize>) {
            (self.item_positions.len(), Some(self.item_positions.len()))
        }
    }

    impl DoubleEndedIterator for EmptyTupleIter {
        fn next_back(&mut self) -> Option<Self::Item> {
            self.next()
        }
    }

    impl ExactSizeIterator for EmptyTupleIter {}
}

#[allow(clippy::let_unit_value)]
#[cfg(test)]
mod empty_tuple_tests {
    use crate::storages::archetypes::ArchetypeFilter;
    use crate::storages::core::CoreStorage;
    use crate::utils::test_utils::assert_iter;
    use crate::{QuerySystemParam, SystemInfo, SystemParam};
    use std::any::TypeId;

    #[test]
    fn retrieve_system_param_properties() {
        let mut core = CoreStorage::default();
        let properties = <()>::properties(&mut core);
        assert_eq!(properties.component_types.len(), 0);
        assert_eq!(properties.globals, vec![]);
        assert!(!properties.can_update);
        assert_eq!(properties.archetype_filter, ArchetypeFilter::None);
    }

    #[test]
    fn use_system_param() {
        let mut core = CoreStorage::default();
        let location = core.create_entity_with_1_component(0_u32, None);
        let filtered_type_idx = core.components().type_idx(TypeId::of::<u32>()).unwrap();
        let info = SystemInfo {
            filtered_component_type_idxs: &[filtered_type_idx],
            archetype_filter: &ArchetypeFilter::All,
            item_count: 3,
        };
        let mut guard = <()>::lock(core.system_data(), info);
        let mut borrow = <()>::borrow_guard(&mut guard);
        let mut stream = <()>::stream(&mut borrow);
        assert!(<()>::stream_next(&mut stream).is_some());
        assert!(<()>::stream_next(&mut stream).is_some());
        assert!(<()>::stream_next(&mut stream).is_some());
        assert!(<()>::stream_next(&mut stream).is_none());
        assert_iter(<()>::query_iter(&borrow), [(), (), ()]);
        assert_iter(<()>::query_iter(&borrow).rev(), [(), (), ()]);
        assert_iter(<()>::query_iter_mut(&mut borrow), [(), (), ()]);
        assert_iter(<()>::query_iter_mut(&mut borrow).rev(), [(), (), ()]);
        assert_eq!(<()>::get(&borrow, location), Some(()));
        assert_eq!(<()>::get_mut(&mut borrow, location), Some(()));
        assert_eq!(<()>::get_mut(&mut borrow, location), Some(()));
        let items = <()>::get_both_mut(&mut borrow, location, location);
        assert_eq!(items, (Some(()), Some(())));
    }
}

#[cfg(test)]
mod tuple_with_one_item_tests {
    use crate::storages::archetypes::ArchetypeFilter;
    use crate::storages::core::CoreStorage;
    use crate::storages::systems::Access;
    use crate::utils::test_utils::assert_iter;
    use crate::{QuerySystemParam, SystemInfo, SystemParam};
    use std::any::TypeId;

    #[test]
    fn retrieve_system_param_properties() {
        let mut core = CoreStorage::default();
        let properties = <(&u32,)>::properties(&mut core);
        assert_eq!(properties.component_types.len(), 1);
        assert_eq!(properties.component_types[0].access, Access::Read);
        assert_eq!(properties.component_types[0].type_idx, 0.into());
        assert_eq!(properties.globals, vec![]);
        assert!(!properties.can_update);
        let archetype_filter = ArchetypeFilter::Intersection(ne_vec![0.into()]);
        assert_eq!(properties.archetype_filter, archetype_filter);
    }

    #[test]
    fn use_system_param() {
        let mut core = CoreStorage::default();
        let location1 = core.create_entity_with_2_components(10_u32, 0_i16, None);
        core.create_entity_with_2_components(20_u32, 0_i16, None);
        let location2 = core.create_entity_with_1_component(30_u32, None);
        let location3 = core.create_entity_with_1_component(40_i64, None);
        let filtered_type_idx = core.components().type_idx(TypeId::of::<i16>()).unwrap();
        let info = SystemInfo {
            filtered_component_type_idxs: &[filtered_type_idx],
            archetype_filter: &ArchetypeFilter::All,
            item_count: 2,
        };
        let mut guard = <(&u32,)>::lock(core.system_data(), info);
        let mut borrow = <(&u32,)>::borrow_guard(&mut guard);
        let mut stream = <(&u32,)>::stream(&mut borrow);
        assert_eq!(<(&u32,)>::stream_next(&mut stream), Some((&10,)));
        assert_eq!(<(&u32,)>::stream_next(&mut stream), Some((&20,)));
        assert_eq!(<(&u32,)>::stream_next(&mut stream), None);
        assert_iter(<(&u32,)>::query_iter(&borrow), [(&10,), (&20,)]);
        assert_iter(<(&u32,)>::query_iter(&borrow).rev(), [(&20,), (&10,)]);
        assert_iter(<(&u32,)>::query_iter_mut(&mut borrow), [(&10,), (&20,)]);
        let iter = <(&u32,)>::query_iter_mut(&mut borrow).rev();
        assert_iter(iter, [(&20,), (&10,)]);
        assert_eq!(<(&u32,)>::get(&borrow, location1), Some((&10,)));
        assert_eq!(<(&u32,)>::get_mut(&mut borrow, location1), Some((&10,)));
        assert_eq!(<(&u32,)>::get(&borrow, location2), Some((&30,)));
        assert_eq!(<(&u32,)>::get_mut(&mut borrow, location2), Some((&30,)));
        assert_eq!(<(&u32,)>::get(&borrow, location3), None);
        assert_eq!(<(&u32,)>::get_mut(&mut borrow, location3), None);
        let items = <(&u32,)>::get_both_mut(&mut borrow, location2, location3);
        assert_eq!(items, (Some((&30,)), None));
    }
}

#[cfg(test)]
mod tuple_with_two_items_tests {
    use crate::storages::archetypes::ArchetypeFilter;
    use crate::storages::core::CoreStorage;
    use crate::storages::systems::Access;
    use crate::utils::test_utils::assert_iter;
    use crate::{QuerySystemParam, SystemInfo, SystemParam, World};
    use std::any::TypeId;

    #[test]
    fn retrieve_system_param_properties_when_can_update() {
        let mut core = CoreStorage::default();
        let properties = <(&u32, World<'_>)>::properties(&mut core);
        assert_eq!(properties.component_types.len(), 1);
        assert_eq!(properties.component_types[0].access, Access::Read);
        assert_eq!(properties.component_types[0].type_idx, 0.into());
        assert_eq!(properties.globals, vec![]);
        assert!(properties.can_update);
        let archetype_filter = ArchetypeFilter::Intersection(ne_vec![0.into()]);
        assert_eq!(properties.archetype_filter, archetype_filter);
    }

    #[test]
    fn retrieve_system_param_properties_when_cannot_update() {
        let mut core = CoreStorage::default();
        let properties = <(&u32, &mut i64)>::properties(&mut core);
        assert_eq!(properties.component_types.len(), 2);
        assert_eq!(properties.component_types[0].access, Access::Read);
        assert_eq!(properties.component_types[0].type_idx, 0.into());
        assert_eq!(properties.component_types[1].access, Access::Write);
        assert_eq!(properties.component_types[1].type_idx, 1.into());
        assert!(!properties.can_update);
        let archetype_filter = ArchetypeFilter::Intersection(ne_vec![0.into(), 1.into()]);
        assert_eq!(properties.archetype_filter, archetype_filter);
    }

    #[test]
    fn use_system_param() {
        let mut core = CoreStorage::default();
        let location1 = core.create_entity_with_2_components(10_u32, 100_i16, None);
        core.create_entity_with_2_components(20_u32, 200_i16, None);
        let location2 = core.create_entity_with_1_component(30_u32, None);
        let location3 = core.create_entity_with_1_component(40_i64, None);
        let filtered_type_idx = core.components().type_idx(TypeId::of::<i16>()).unwrap();
        let info = SystemInfo {
            filtered_component_type_idxs: &[filtered_type_idx],
            archetype_filter: &ArchetypeFilter::All,
            item_count: 2,
        };
        let mut guard = <(&u32, &mut i16)>::lock(core.system_data(), info);
        let mut borrow = <(&u32, &mut i16)>::borrow_guard(&mut guard);
        let mut stream = <(&u32, &mut i16)>::stream(&mut borrow);
        let item = <(&u32, &mut i16)>::stream_next(&mut stream);
        assert_eq!(item, Some((&10, &mut 100)));
        let item = <(&u32, &mut i16)>::stream_next(&mut stream);
        assert_eq!(item, Some((&20, &mut 200)));
        assert_eq!(<(&u32, &mut i16)>::stream_next(&mut stream), None);
        let iter = <(&u32, &mut i16)>::query_iter(&borrow);
        assert_iter(iter, [(&10, &100), (&20, &200)]);
        let iter = <(&u32, &mut i16)>::query_iter(&borrow).rev();
        assert_iter(iter, [(&20, &200), (&10, &100)]);
        let iter = <(&u32, &mut i16)>::query_iter_mut(&mut borrow);
        assert_iter(iter, [(&10, &mut 100), (&20, &mut 200)]);
        let iter = <(&u32, &mut i16)>::query_iter_mut(&mut borrow).rev();
        assert_iter(iter, [(&20, &mut 200), (&10, &mut 100)]);
        let item = <(&u32, &mut i16)>::get(&borrow, location1);
        assert_eq!(item, Some((&10, &100)));
        let item = <(&u32, &mut i16)>::get_mut(&mut borrow, location1);
        assert_eq!(item, Some((&10, &mut 100)));
        assert_eq!(<(&u32, &mut i16)>::get(&borrow, location2), None);
        assert_eq!(<(&u32, &mut i16)>::get_mut(&mut borrow, location2), None);
        assert_eq!(<(&u32, &mut i16)>::get(&borrow, location3), None);
        assert_eq!(<(&u32, &mut i16)>::get_mut(&mut borrow, location3), None);
        let items = <(&u32, &mut i16)>::get_both_mut(&mut borrow, location1, location2);
        assert_eq!(items, (Some((&10, &mut 100)), None));
    }
}

#[cfg(test)]
mod tuple_with_more_than_two_items_tests {
    use crate::storages::archetypes::ArchetypeFilter;
    use crate::storages::core::CoreStorage;
    use crate::storages::systems::Access;
    use crate::utils::test_utils::assert_iter;
    use crate::{QuerySystemParam, SystemInfo, SystemParam};
    use std::any::TypeId;

    macro_rules! test_tuple_retrieve_system_param_properties {
        (($($types:ident),*), ($($indexes:tt),*)) => {
            let mut core = CoreStorage::default();
            let properties = <($(&$types,)*)>::properties(&mut core);
            assert_eq!(properties.component_types.len(), [$($indexes),*].len());
            $(assert_eq!(properties.component_types[$indexes].access, Access::Read);)*
            $(assert_eq!(properties.component_types[$indexes].type_idx, $indexes.into());)*
            assert_eq!(properties.globals, vec![]);
            assert!(!properties.can_update);
            let archetype_filter = ArchetypeFilter::Intersection(ne_vec![$($indexes.into()),*]);
            assert_eq!(properties.archetype_filter, archetype_filter);
        };
    }

    #[test]
    #[allow(clippy::cognitive_complexity)]
    fn retrieve_system_param_properties() {
        test_tuple_retrieve_system_param_properties!((u8, u16, u32), (0, 1, 2));
        test_tuple_retrieve_system_param_properties!((u8, u16, u32, u64), (0, 1, 2, 3));
        test_tuple_retrieve_system_param_properties!((u8, u16, u32, u64, u128), (0, 1, 2, 3, 4));
        test_tuple_retrieve_system_param_properties!(
            (u8, u16, u32, u64, u128, i8),
            (0, 1, 2, 3, 4, 5)
        );
        test_tuple_retrieve_system_param_properties!(
            (u8, u16, u32, u64, u128, i8, i16),
            (0, 1, 2, 3, 4, 5, 6)
        );
        test_tuple_retrieve_system_param_properties!(
            (u8, u16, u32, u64, u128, i8, i16, i32),
            (0, 1, 2, 3, 4, 5, 6, 7)
        );
        test_tuple_retrieve_system_param_properties!(
            (u8, u16, u32, u64, u128, i8, i16, i32, i64),
            (0, 1, 2, 3, 4, 5, 6, 7, 8)
        );
        test_tuple_retrieve_system_param_properties!(
            (u8, u16, u32, u64, u128, i8, i16, i32, i64, i128),
            (0, 1, 2, 3, 4, 5, 6, 7, 8, 9)
        );
    }

    #[test]
    fn use_system_param() {
        let mut core = CoreStorage::default();
        let location1 = core.create_entity_with_3_components(10_u32, 100_i16, 1000_i64, None);
        let location2 = core.create_entity_with_3_components(20_u32, 200_i16, 2000_i64, None);
        let filtered_type_idx = core.components().type_idx(TypeId::of::<i16>()).unwrap();
        let info = SystemInfo {
            filtered_component_type_idxs: &[filtered_type_idx],
            archetype_filter: &ArchetypeFilter::All,
            item_count: 2,
        };
        let mut guard = <(&u32, &mut i16, &i64)>::lock(core.system_data(), info);
        let mut borrow = <(&u32, &mut i16, &i64)>::borrow_guard(&mut guard);
        let mut stream = <(&u32, &mut i16, &i64)>::stream(&mut borrow);
        let item = <(&u32, &mut i16, &i64)>::stream_next(&mut stream);
        assert_eq!(item, Some((&10, &mut 100, &1000)));
        let item = <(&u32, &mut i16, &i64)>::stream_next(&mut stream);
        assert_eq!(item, Some((&20, &mut 200, &2000)));
        assert_eq!(<(&u32, &mut i16, &i64)>::stream_next(&mut stream), None);
        let iter = <(&u32, &mut i16, &i64)>::query_iter(&borrow);
        assert_iter(iter, [(&10, &100, &1000), (&20, &200, &2000)]);
        let iter = <(&u32, &mut i16, &i64)>::query_iter(&borrow).rev();
        assert_iter(iter, [(&20, &200, &2000), (&10, &100, &1000)]);
        let iter = <(&u32, &mut i16, &i64)>::query_iter_mut(&mut borrow);
        assert_iter(iter, [(&10, &mut 100, &1000), (&20, &mut 200, &2000)]);
        let iter = <(&u32, &mut i16, &i64)>::query_iter_mut(&mut borrow).rev();
        assert_iter(iter, [(&20, &mut 200, &2000), (&10, &mut 100, &1000)]);
        let item = <(&u32, &mut i16, &i64)>::get(&borrow, location1);
        assert_eq!(item, Some((&10, &100, &1000)));
        let item = <(&u32, &mut i16, &i64)>::get_mut(&mut borrow, location1);
        assert_eq!(item, Some((&10, &mut 100, &1000)));
        let items = <(&u32, &mut i16, &i64)>::get_both_mut(&mut borrow, location1, location2);
        assert_eq!(
            items,
            (Some((&10, &mut 100, &1000)), Some((&20, &mut 200, &2000)))
        );
    }
}
