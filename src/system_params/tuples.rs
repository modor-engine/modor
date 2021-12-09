use crate::storages::core::SystemProperties;
use crate::system_params::internal::{
    QuerySystemParamWithLifetime, SystemParamIterInfo, SystemParamWithLifetime,
};
use crate::system_params::tuples::internal::EmptyTupleIter;
use crate::{QuerySystemParam, SystemData, SystemInfo, SystemParam};
use std::iter;
use std::iter::{Map, Zip};

macro_rules! impl_tuple_system_param {
    ($(($params:ident, $indexes:tt)),*) => {
        impl<'a $(,$params)*> SystemParamWithLifetime<'a> for ($($params,)*)
        where
            $($params: SystemParamWithLifetime<'a>,)*
        {
            type Param = ($($params::Param,)*);
            type Guard = ($($params::Guard,)*);
            type GuardBorrow = ($($params::GuardBorrow,)*);
            stream_type!($(($params, $indexes)),*);
        }

        impl<$($params),*> SystemParam for ($($params,)*)
        where
            $($params: SystemParam,)*
        {
            type Tuple = Self;
            type InnerTuple = Self;

            #[allow(unused_variables)]
            fn properties() -> SystemProperties {
                let properties = ($($params::properties(),)*);
                SystemProperties {
                    component_types: iter::empty()
                        $(.chain(properties.$indexes.component_types))*
                        .collect(),
                    has_entity_actions: $(properties.$indexes.has_entity_actions ||)* false,
                }
            }

            #[allow(unused_variables)]
            fn iter_info(data: &SystemData<'_>, info: &SystemInfo) -> SystemParamIterInfo {
                let iter_info = ($($params::iter_info(data, info),)*);
                SystemParamIterInfo::None $(.merge(iter_info.$indexes))*
            }

            #[allow(unused_variables, clippy::unused_unit)]
            fn lock<'a>(data: &'a SystemData<'_>) -> <Self as SystemParamWithLifetime<'a>>::Guard {
                ($($params::lock(data),)*)
            }

            #[allow(unused_variables, clippy::unused_unit)]
            fn borrow_guard<'a, 'b>(
                guard: &'a mut <Self as SystemParamWithLifetime<'b>>::Guard,
            ) -> <Self as SystemParamWithLifetime<'a>>::GuardBorrow
            where
                'b: 'a,
            {
                ($($params::borrow_guard(&mut guard.$indexes),)*)
            }

            #[allow(unused_variables, clippy::unused_unit)]
            fn stream<'a, 'b>(
                guard: &'a mut <Self as SystemParamWithLifetime<'b>>::GuardBorrow,
                info: &'a SystemParamIterInfo,
            ) -> <Self as SystemParamWithLifetime<'a>>::Stream
            where
                'b: 'a,
            {
                stream_new!(guard, info $(,($params, $indexes))*)
            }

            #[allow(unused_variables)]
            #[inline]
            fn stream_next<'a, 'b>(
                stream: &'a mut <Self as SystemParamWithLifetime<'b>>::Stream,
            ) -> Option<<Self as SystemParamWithLifetime<'a>>::Param>
            where
                'b: 'a,
            {
                stream_next!(stream $(,($params, $indexes))*)
            }
        }

        impl<'a $(,$params)*> QuerySystemParamWithLifetime<'a> for ($($params,)*)
        where
            $($params: QuerySystemParamWithLifetime<'a>,)*
        {
            type ConstParam = ($($params::ConstParam,)*);
            iter_type!($($params),*);
            iter_mut_type!($($params),*);
        }

        impl<$($params),*> QuerySystemParam for ($($params,)*)
        where
            $($params: QuerySystemParam,)*
        {
            #[allow(unused_variables, non_snake_case)]
            fn query_iter<'a, 'b>(
                guard: &'a <Self as SystemParamWithLifetime<'b>>::GuardBorrow,
                info: &'a SystemParamIterInfo,
            ) -> <Self as QuerySystemParamWithLifetime<'a>>::Iter
            where
                'b: 'a,
            {
                query_iter!(guard, info $(,($params, $indexes))*)
            }

            #[allow(unused_variables, non_snake_case)]
            fn query_iter_mut<'a, 'b>(
                guard: &'a mut <Self as SystemParamWithLifetime<'b>>::GuardBorrow,
                info: &'a SystemParamIterInfo,
            ) -> <Self as QuerySystemParamWithLifetime<'a>>::IterMut
            where
                'b: 'a,
            {
                query_iter_mut!(guard, info $(,($params, $indexes))*)
            }
        }
    };
}

macro_rules! stream_type {
    () => {
        type Stream = EmptyTupleIter;
    };
    ($(($params:ident, $indexes:tt)),+) => {
        type Stream = ($($params::Stream,)+);
    };
}

macro_rules! stream_new {
    ($guard:expr, $info:expr) => {
        EmptyTupleIter::new($info.item_count())
    };
    ($guard:expr, $info:expr $(,($params:ident, $indexes:tt))+) => {
        ($($params::stream(&mut $guard.$indexes, $info),)+)
    };
}

macro_rules! stream_next {
    ($stream:expr) => {
        $stream.next()
    };
    ($stream:expr $(,($params:ident, $indexes:tt))+) => {
        Some(($($params::stream_next(&mut $stream.$indexes)?,)*))
    };
}

macro_rules! iter_type {
    () => { type Iter = EmptyTupleIter; };
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
    () => { type IterMut = EmptyTupleIter; };
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
    ($guard:expr, $info:expr) => { EmptyTupleIter::new($info.item_count()) };
    ($guard:expr, $info:expr, ($param:ident, $index:tt)) => {
        $param::query_iter(&$guard.$index, $info).map(|i| (i,))
    };
    ($guard:expr, $info:expr, ($param1:ident, $index1:tt), ($param2:ident, $index2:tt)) => {
        $param1::query_iter(&$guard.$index1, $info).zip($param2::query_iter(&$guard.$index2, $info))
    };
    ($guard:expr, $info:expr, ($param:ident, $index:tt), $(($params:ident, $indexes:tt)),+) => {
        $param::query_iter(&$guard.$index, $info)
            $(.zip($params::query_iter(&$guard.$indexes, $info)))+
            .map(|nested_tuple!($param $(, $params)*)| ($param, $($params),*))
    };
}

macro_rules! query_iter_mut {
    ($guard:expr, $info:expr) => { EmptyTupleIter::new($info.item_count()) };
    ($guard:expr, $info:expr, ($param:ident, $index:tt)) => {
        $param::query_iter_mut(&mut $guard.$index, $info).map(|i| (i,))
    };
    ($guard:expr, $info:expr, ($param1:ident, $index1:tt), ($param2:ident, $index2:tt)) => {
        $param1::query_iter_mut(&mut $guard.$index1, $info)
            .zip($param2::query_iter_mut(&mut $guard.$index2, $info))
    };
    ($guard:expr, $info:expr, ($param:ident, $index:tt), $(($params:ident, $indexes:tt)),+) => {
        $param::query_iter_mut(&mut $guard.$index, $info)
            $(.zip($params::query_iter_mut(&mut $guard.$indexes, $info)))+
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

impl_tuple_system_param!();
run_for_tuples_with_idxs!(impl_tuple_system_param);

mod internal {
    use std::ops::Range;

    pub struct EmptyTupleIter {
        item_positions: Range<usize>,
    }

    impl EmptyTupleIter {
        pub(crate) fn new(item_count: usize) -> Self {
            Self {
                item_positions: 0..item_count,
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
mod empty_tuple_system_param_tests {
    use super::*;
    use crate::storages::core::CoreStorage;
    use crate::SystemInfo;

    #[test]
    fn retrieve_properties() {
        let properties = <()>::properties();

        assert_eq!(properties.component_types.len(), 0);
        assert!(!properties.has_entity_actions);
    }

    #[test]
    fn retrieve_iter_info() {
        let core = CoreStorage::default();
        let info = SystemInfo::with_one_filtered_type::<i64>();

        let iter_info = <()>::iter_info(&core.system_data(), &info);

        assert_eq!(iter_info, SystemParamIterInfo::None);
    }

    #[test]
    fn lock() {
        let core = CoreStorage::default();
        let data = core.system_data();

        let mut guard = <()>::lock(&data);
        let guard_borrow = <()>::borrow_guard(&mut guard);

        assert!(matches!(guard_borrow, ()));
    }

    #[test]
    fn retrieve_stream_when_no_iteration() {
        let mut guard_borrow = ();
        let iter_info = SystemParamIterInfo::None;

        let mut stream = <()>::stream(&mut guard_borrow, &iter_info);

        assert_eq!(<()>::stream_next(&mut stream), Some(()));
        assert_eq!(<()>::stream_next(&mut stream), None);
    }

    #[test]
    fn retrieve_stream_when_iteration_on_entities() {
        let mut guard_borrow = ();
        let iter_info = SystemParamIterInfo::new_union(vec![(0.into(), 1), (2.into(), 2)]);

        let mut stream = <()>::stream(&mut guard_borrow, &iter_info);

        assert_eq!(<()>::stream_next(&mut stream), Some(()));
        assert_eq!(<()>::stream_next(&mut stream), Some(()));
        assert_eq!(<()>::stream_next(&mut stream), Some(()));
        assert_eq!(<()>::stream_next(&mut stream), None);
    }

    #[test]
    fn retrieve_query_iter() {
        let guard_borrow = ();
        let iter_info = SystemParamIterInfo::new_union(vec![(0.into(), 1), (2.into(), 2)]);

        let iter = <()>::query_iter(&guard_borrow, &iter_info);

        assert_iter!(iter, [(), (), ()]);
    }

    #[test]
    fn retrieve_query_iter_mut() {
        let mut guard_borrow = ();
        let iter_info = SystemParamIterInfo::new_union(vec![(0.into(), 1), (2.into(), 2)]);

        let iter = <()>::query_iter_mut(&mut guard_borrow, &iter_info);

        assert_iter!(iter, [(), (), ()]);
    }
}

#[cfg(test)]
mod tuple_with_one_item_system_param_tests {
    use super::*;
    use crate::storages::archetypes::ArchetypeStorage;
    use crate::storages::core::CoreStorage;
    use crate::storages::systems::Access;
    use crate::SystemInfo;

    #[test]
    fn retrieve_properties() {
        let properties = <(&u32,)>::properties();

        assert_eq!(properties.component_types.len(), 1);
        assert_eq!(properties.component_types[0].access, Access::Read);
        assert!(!properties.has_entity_actions);
    }

    #[test]
    fn retrieve_iter_info() {
        let mut core = CoreStorage::default();
        let (_, archetype1_idx) = core.add_component_type::<i64>(ArchetypeStorage::DEFAULT_IDX);
        let (_, archetype2_idx) = core.add_component_type::<u32>(archetype1_idx);
        let info = SystemInfo::with_one_filtered_type::<i64>();

        let iter_info = <(&u32,)>::iter_info(&core.system_data(), &info);

        let expected_iter_info = SystemParamIterInfo::new_intersection(vec![(archetype2_idx, 0)]);
        assert_eq!(iter_info, expected_iter_info);
    }

    #[test]
    fn lock() {
        let mut core = CoreStorage::default();
        let archetype1_idx = ArchetypeStorage::DEFAULT_IDX;
        let (type_idx, archetype2_idx) = core.add_component_type::<u32>(archetype1_idx);
        let location = core.create_entity(archetype2_idx);
        core.add_component(10_u32, type_idx, location);
        let data = core.system_data();

        let mut guard = <(&u32,)>::lock(&data);
        let guard_borrow = <(&u32,)>::borrow_guard(&mut guard);

        assert_eq!(guard_borrow, (&ti_vec![ti_vec![], ti_vec![10_u32]],));
    }

    #[test]
    fn retrieve_stream() {
        let guard = ti_vec![ti_vec![10], ti_vec![20]];
        let mut guard_borrow = (&guard,);
        let iter_info = SystemParamIterInfo::new_intersection(vec![(0.into(), 1), (1.into(), 1)]);

        let mut stream = <(&u32,)>::stream(&mut guard_borrow, &iter_info);

        assert_eq!(<(&u32,)>::stream_next(&mut stream), Some((&10,)));
        assert_eq!(<(&u32,)>::stream_next(&mut stream), Some((&20,)));
        assert_eq!(<(&u32,)>::stream_next(&mut stream), None);
    }

    #[test]
    fn retrieve_query_iter() {
        let guard = ti_vec![ti_vec![10], ti_vec![20]];
        let guard_borrow = (&guard,);
        let iter_info = SystemParamIterInfo::new_intersection(vec![(0.into(), 1), (1.into(), 1)]);

        let iter = <(&u32,)>::query_iter(&guard_borrow, &iter_info);

        assert_iter!(iter, [(&10,), (&20,)]);
    }

    #[test]
    fn retrieve_query_iter_mut() {
        let guard = ti_vec![ti_vec![10], ti_vec![20]];
        let mut guard_borrow = (&guard,);
        let iter_info = SystemParamIterInfo::new_intersection(vec![(0.into(), 1), (1.into(), 1)]);

        let iter = <(&u32,)>::query_iter_mut(&mut guard_borrow, &iter_info);

        assert_iter!(iter, [(&10,), (&20,)]);
    }
}

#[cfg(test)]
mod tuple_with_two_items_system_param_tests {
    use super::*;
    use crate::storages::archetypes::ArchetypeStorage;
    use crate::storages::core::CoreStorage;
    use crate::storages::systems::Access;
    use crate::{SystemInfo, World};

    #[test]
    fn retrieve_properties_with_entity_action() {
        let properties = <(&u32, World<'_>)>::properties();

        assert_eq!(properties.component_types.len(), 1);
        assert_eq!(properties.component_types[0].access, Access::Read);
        assert!(properties.has_entity_actions);
    }

    #[test]
    fn retrieve_properties_without_entity_action() {
        let properties = <(&u32, &mut i64)>::properties();

        assert_eq!(properties.component_types.len(), 2);
        assert_eq!(properties.component_types[0].access, Access::Read);
        assert_eq!(properties.component_types[1].access, Access::Write);
        assert!(!properties.has_entity_actions);
    }

    #[test]
    fn retrieve_iter_info() {
        let mut core = CoreStorage::default();
        let (_, archetype1_idx) = core.add_component_type::<i64>(ArchetypeStorage::DEFAULT_IDX);
        let (_, archetype2_idx) = core.add_component_type::<u32>(archetype1_idx);
        let info = SystemInfo::with_one_filtered_type::<i64>();

        let iter_info = <(&u32, &mut i64)>::iter_info(&core.system_data(), &info);

        let expected_iter_info = SystemParamIterInfo::new_intersection(vec![(archetype2_idx, 0)]);
        assert_eq!(iter_info, expected_iter_info);
    }

    #[test]
    fn lock() {
        let mut core = CoreStorage::default();
        let archetype1_idx = ArchetypeStorage::DEFAULT_IDX;
        let (type1_idx, archetype2_idx) = core.add_component_type::<i64>(archetype1_idx);
        let (type2_idx, archetype3_idx) = core.add_component_type::<u32>(archetype2_idx);
        let location = core.create_entity(archetype3_idx);
        core.add_component(10_i64, type1_idx, location);
        core.add_component(20_u32, type2_idx, location);
        let data = core.system_data();

        let mut guard = <(&u32, &mut i64)>::lock(&data);
        let guard_borrow = <(&u32, &mut i64)>::borrow_guard(&mut guard);

        let expected_guard = ti_vec![ti_vec![], ti_vec![], ti_vec![20_u32]];
        assert_eq!(guard_borrow.0, &expected_guard);
        let expected_guard = ti_vec![ti_vec![], ti_vec![], ti_vec![10_i64]];
        assert_eq!(guard_borrow.1, &expected_guard);
    }

    #[test]
    fn retrieve_stream() {
        let guard1 = ti_vec![ti_vec![10], ti_vec![20]];
        let mut guard2 = ti_vec![ti_vec![30], ti_vec![40]];
        let mut guard_borrow = (&guard1, &mut guard2);
        let iter_info = SystemParamIterInfo::new_intersection(vec![(0.into(), 1), (1.into(), 1)]);

        let mut stream = <(&u32, &mut i64)>::stream(&mut guard_borrow, &iter_info);

        let next = <(&u32, &mut i64)>::stream_next(&mut stream);
        assert_eq!(next, Some((&10, &mut 30)));
        let next = <(&u32, &mut i64)>::stream_next(&mut stream);
        assert_eq!(next, Some((&20, &mut 40)));
        assert_eq!(<(&u32, &mut i64)>::stream_next(&mut stream), None);
    }

    #[test]
    fn retrieve_query_iter() {
        let guard1 = ti_vec![ti_vec![10], ti_vec![20]];
        let mut guard2 = ti_vec![ti_vec![30], ti_vec![40]];
        let guard_borrow = (&guard1, &mut guard2);
        let iter_info = SystemParamIterInfo::new_intersection(vec![(0.into(), 1), (1.into(), 1)]);

        let iter = <(&u32, &mut i64)>::query_iter(&guard_borrow, &iter_info);

        assert_iter!(iter, [(&10, &30), (&20, &40)]);
    }

    #[test]
    fn retrieve_query_iter_mut() {
        let guard1 = ti_vec![ti_vec![10], ti_vec![20]];
        let mut guard2 = ti_vec![ti_vec![30], ti_vec![40]];
        let mut guard_borrow = (&guard1, &mut guard2);
        let iter_info = SystemParamIterInfo::new_intersection(vec![(0.into(), 1), (1.into(), 1)]);

        let iter = <(&u32, &mut i64)>::query_iter_mut(&mut guard_borrow, &iter_info);

        assert_iter!(iter, [(&10, &mut 30), (&20, &mut 40)]);
    }
}

#[cfg(test)]
mod tuple_with_more_than_two_items_system_param_tests {
    use super::*;
    use crate::storages::archetypes::ArchetypeStorage;
    use crate::storages::core::CoreStorage;
    use crate::storages::systems::Access;
    use crate::SystemInfo;

    #[test]
    fn retrieve_properties() {
        let properties = <(&u32, &mut i64, &i16)>::properties();

        assert_eq!(properties.component_types.len(), 3);
        assert_eq!(properties.component_types[0].access, Access::Read);
        assert_eq!(properties.component_types[1].access, Access::Write);
        assert_eq!(properties.component_types[2].access, Access::Read);
        assert!(!properties.has_entity_actions);
    }

    #[test]
    fn retrieve_iter_info() {
        let mut core = CoreStorage::default();
        let (_, archetype1_idx) = core.add_component_type::<i64>(ArchetypeStorage::DEFAULT_IDX);
        let (_, archetype2_idx) = core.add_component_type::<u32>(archetype1_idx);
        let (_, archetype3_idx) = core.add_component_type::<i16>(archetype2_idx);
        let info = SystemInfo::with_one_filtered_type::<i64>();

        let iter_info = <(&u32, &mut i64, &i16)>::iter_info(&core.system_data(), &info);

        let expected_iter_info = SystemParamIterInfo::new_intersection(vec![(archetype3_idx, 0)]);
        assert_eq!(iter_info, expected_iter_info);
    }

    #[test]
    fn lock() {
        let mut core = CoreStorage::default();
        let archetype1_idx = ArchetypeStorage::DEFAULT_IDX;
        let (type1_idx, archetype2_idx) = core.add_component_type::<i64>(archetype1_idx);
        let (type2_idx, archetype3_idx) = core.add_component_type::<u32>(archetype2_idx);
        let (type3_idx, archetype4_idx) = core.add_component_type::<i16>(archetype3_idx);
        let location = core.create_entity(archetype4_idx);
        core.add_component(10_i64, type1_idx, location);
        core.add_component(20_u32, type2_idx, location);
        core.add_component(30_i16, type3_idx, location);
        let data = core.system_data();

        let mut guard = <(&u32, &mut i64, &i16)>::lock(&data);
        let guard_borrow = <(&u32, &mut i64, &i16)>::borrow_guard(&mut guard);

        let expected_guard = ti_vec![ti_vec![], ti_vec![], ti_vec![], ti_vec![20_u32]];
        assert_eq!(guard_borrow.0, &expected_guard);
        let expected_guard = ti_vec![ti_vec![], ti_vec![], ti_vec![], ti_vec![10_i64]];
        assert_eq!(guard_borrow.1, &expected_guard);
        let expected_guard = ti_vec![ti_vec![], ti_vec![], ti_vec![], ti_vec![30_i16]];
        assert_eq!(guard_borrow.2, &expected_guard);
    }

    #[test]
    fn retrieve_stream() {
        let guard1 = ti_vec![ti_vec![10], ti_vec![20]];
        let mut guard2 = ti_vec![ti_vec![30], ti_vec![40]];
        let guard3 = ti_vec![ti_vec![50], ti_vec![60]];
        let mut guard_borrow = (&guard1, &mut guard2, &guard3);
        let iter_info = SystemParamIterInfo::new_intersection(vec![(0.into(), 1), (1.into(), 1)]);

        let mut stream = <(&u32, &mut i64, &i16)>::stream(&mut guard_borrow, &iter_info);

        let next = <(&u32, &mut i64, &i16)>::stream_next(&mut stream);
        assert_eq!(next, Some((&10, &mut 30, &50)));
        let next = <(&u32, &mut i64, &i16)>::stream_next(&mut stream);
        assert_eq!(next, Some((&20, &mut 40, &60)));
        assert_eq!(<(&u32, &mut i64, &i16)>::stream_next(&mut stream), None);
    }

    #[test]
    fn retrieve_query_iter() {
        let guard1 = ti_vec![ti_vec![10], ti_vec![20]];
        let mut guard2 = ti_vec![ti_vec![30], ti_vec![40]];
        let guard3 = ti_vec![ti_vec![50], ti_vec![60]];
        let guard_borrow = (&guard1, &mut guard2, &guard3);
        let iter_info = SystemParamIterInfo::new_intersection(vec![(0.into(), 1), (1.into(), 1)]);

        let iter = <(&u32, &mut i64, &i16)>::query_iter(&guard_borrow, &iter_info);

        assert_iter!(iter, [(&10, &30, &50), (&20, &40, &60)]);
    }

    #[test]
    fn retrieve_query_iter_mut() {
        let guard1 = ti_vec![ti_vec![10], ti_vec![20]];
        let mut guard2 = ti_vec![ti_vec![30], ti_vec![40]];
        let guard3 = ti_vec![ti_vec![50], ti_vec![60]];
        let mut guard_borrow = (&guard1, &mut guard2, &guard3);
        let iter_info = SystemParamIterInfo::new_intersection(vec![(0.into(), 1), (1.into(), 1)]);

        let iter = <(&u32, &mut i64, &i16)>::query_iter_mut(&mut guard_borrow, &iter_info);

        assert_iter!(iter, [(&10, &mut 30, &50), (&20, &mut 40, &60)]);
    }
}
