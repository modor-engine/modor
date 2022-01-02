use crate::storages::archetypes::ArchetypeFilter;
use crate::storages::core::CoreStorage;
use crate::storages::systems::SystemProperties;
use crate::system_params::internal::{QuerySystemParamWithLifetime, SystemParamWithLifetime};
use crate::system_params::tuples::internal::{EmptyTupleGuard, EmptyTupleIter};
use crate::tuples::internal::EmptyTupleGuardBorrow;
use crate::{QuerySystemParam, SystemData, SystemInfo, SystemParam};
use std::iter;
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
            has_entity_actions: false,
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
                SystemProperties {
                    component_types:
                        iter::empty() $(.chain(properties.$indexes.component_types))+.collect(),
                    has_entity_actions:
                        [$(properties.$indexes.has_entity_actions),+].into_iter().any(|b| b),
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
mod empty_tuple_system_param_tests {
    use super::*;
    use crate::storages::archetypes::ArchetypeStorage;
    use crate::storages::core::CoreStorage;
    use crate::SystemInfo;

    #[test]
    fn retrieve_properties() {
        let mut core = CoreStorage::default();

        let properties = <()>::properties(&mut core);

        assert_eq!(properties.component_types.len(), 0);
        assert!(!properties.has_entity_actions);
        assert_eq!(properties.archetype_filter, ArchetypeFilter::None);
    }

    #[test]
    fn lock() {
        let mut core = CoreStorage::default();
        let archetype1_idx = ArchetypeStorage::DEFAULT_IDX;
        let (type_idx, archetype2_idx) = core.add_component_type::<u32>(archetype1_idx);
        let location = core.create_entity(archetype2_idx);
        core.add_component(10_u32, type_idx, location);
        let data = core.system_data();
        let info = SystemInfo {
            filtered_component_type_idxs: &[0.into()],
            archetype_filter: &ArchetypeFilter::All,
            item_count: 1,
        };

        let mut guard = <()>::lock(data, info);
        let guard_borrow = <()>::borrow_guard(&mut guard);

        assert_eq!(guard_borrow.item_count, 1);
    }

    #[test]
    fn retrieve_stream() {
        let mut guard_borrow = EmptyTupleGuardBorrow { item_count: 3 };

        let mut stream = <()>::stream(&mut guard_borrow);

        assert_eq!(<()>::stream_next(&mut stream), Some(()));
        assert_eq!(<()>::stream_next(&mut stream), Some(()));
        assert_eq!(<()>::stream_next(&mut stream), Some(()));
        assert_eq!(<()>::stream_next(&mut stream), None);
    }

    #[test]
    fn retrieve_query_iter() {
        let guard_borrow = EmptyTupleGuardBorrow { item_count: 3 };

        let mut iter = <()>::query_iter(&guard_borrow);

        assert_eq!(iter.len(), 3);
        assert_eq!(iter.next(), Some(()));
        assert_eq!(iter.len(), 2);
        assert_eq!(iter.next(), Some(()));
        assert_eq!(iter.len(), 1);
        assert_eq!(iter.next(), Some(()));
        assert_eq!(iter.len(), 0);
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn retrieve_reversed_query_iter() {
        let guard_borrow = EmptyTupleGuardBorrow { item_count: 3 };

        let mut iter = <()>::query_iter(&guard_borrow).rev();

        assert_eq!(iter.len(), 3);
        assert_eq!(iter.next(), Some(()));
        assert_eq!(iter.len(), 2);
        assert_eq!(iter.next(), Some(()));
        assert_eq!(iter.len(), 1);
        assert_eq!(iter.next(), Some(()));
        assert_eq!(iter.len(), 0);
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn retrieve_query_iter_mut() {
        let mut guard_borrow = EmptyTupleGuardBorrow { item_count: 3 };

        let mut iter = <()>::query_iter_mut(&mut guard_borrow);

        assert_eq!(iter.len(), 3);
        assert_eq!(iter.next(), Some(()));
        assert_eq!(iter.len(), 2);
        assert_eq!(iter.next(), Some(()));
        assert_eq!(iter.len(), 1);
        assert_eq!(iter.next(), Some(()));
        assert_eq!(iter.len(), 0);
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn retrieve_reversed_query_iter_mut() {
        let mut guard_borrow = EmptyTupleGuardBorrow { item_count: 3 };

        let mut iter = <()>::query_iter_mut(&mut guard_borrow).rev();

        assert_eq!(iter.len(), 3);
        assert_eq!(iter.next(), Some(()));
        assert_eq!(iter.len(), 2);
        assert_eq!(iter.next(), Some(()));
        assert_eq!(iter.len(), 1);
        assert_eq!(iter.next(), Some(()));
        assert_eq!(iter.len(), 0);
        assert_eq!(iter.next(), None);
    }
}

#[cfg(test)]
mod tuple_with_one_item_system_param_tests {
    use super::*;
    use crate::components::internal::ComponentGuardBorrow;
    use crate::storages::archetypes::{ArchetypeStorage, FilteredArchetypeIdxIter};
    use crate::storages::core::CoreStorage;
    use crate::storages::systems::Access;
    use crate::SystemInfo;

    #[test]
    fn retrieve_properties() {
        let mut core = CoreStorage::default();

        let properties = <(&u32,)>::properties(&mut core);

        assert_eq!(properties.component_types.len(), 1);
        assert_eq!(properties.component_types[0].access, Access::Read);
        assert_eq!(properties.component_types[0].type_idx, 0.into());
        assert!(!properties.has_entity_actions);
        let archetype_filter = ArchetypeFilter::Intersection(ne_vec![0.into()]);
        assert_eq!(properties.archetype_filter, archetype_filter);
    }

    #[test]
    fn lock() {
        let mut core = CoreStorage::default();
        let archetype1_idx = ArchetypeStorage::DEFAULT_IDX;
        let (type_idx, archetype2_idx) = core.add_component_type::<u32>(archetype1_idx);
        let location = core.create_entity(archetype2_idx);
        core.add_component(10_u32, type_idx, location);
        let data = core.system_data();
        let info = SystemInfo {
            filtered_component_type_idxs: &[0.into()],
            archetype_filter: &ArchetypeFilter::All,
            item_count: 1,
        };

        let mut guard = <(&u32,)>::lock(data, info);
        let guard_borrow = <(&u32,)>::borrow_guard(&mut guard);

        let components = guard_borrow.0.components;
        assert_eq!(components, &ti_vec![ti_vec![], ti_vec![10_u32]]);
    }

    #[test]
    fn retrieve_stream() {
        let components = ti_vec![ti_vec![10], ti_vec![20]];
        let archetype_idxs = [0.into(), 1.into()];
        let archetype_type_idxs = ti_vec![vec![0.into()]; 2];
        let mut guard_borrow = (ComponentGuardBorrow {
            components: &components,
            item_count: 3,
            sorted_archetype_idxs: FilteredArchetypeIdxIter::new(
                &archetype_idxs,
                &archetype_type_idxs,
            ),
        },);

        let mut stream = <(&u32,)>::stream(&mut guard_borrow);

        assert_eq!(<(&u32,)>::stream_next(&mut stream), Some((&10,)));
        assert_eq!(<(&u32,)>::stream_next(&mut stream), Some((&20,)));
        assert_eq!(<(&u32,)>::stream_next(&mut stream), None);
    }

    #[test]
    fn retrieve_query_iter() {
        let components = ti_vec![ti_vec![10], ti_vec![20]];
        let archetype_idxs = [0.into(), 1.into()];
        let archetype_type_idxs = ti_vec![vec![0.into()]; 2];
        let guard_borrow = (ComponentGuardBorrow {
            components: &components,
            item_count: 3,
            sorted_archetype_idxs: FilteredArchetypeIdxIter::new(
                &archetype_idxs,
                &archetype_type_idxs,
            ),
        },);

        let mut iter = <(&u32,)>::query_iter(&guard_borrow);

        assert_eq!(iter.next(), Some((&10,)));
        assert_eq!(iter.next(), Some((&20,)));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn retrieve_query_iter_mut() {
        let components = ti_vec![ti_vec![10], ti_vec![20]];
        let archetype_idxs = [0.into(), 1.into()];
        let archetype_type_idxs = ti_vec![vec![0.into()]; 2];
        let mut guard_borrow = (ComponentGuardBorrow {
            components: &components,
            item_count: 2,
            sorted_archetype_idxs: FilteredArchetypeIdxIter::new(
                &archetype_idxs,
                &archetype_type_idxs,
            ),
        },);

        let mut iter = <(&u32,)>::query_iter_mut(&mut guard_borrow);

        assert_eq!(iter.next(), Some((&10,)));
        assert_eq!(iter.next(), Some((&20,)));
        assert_eq!(iter.next(), None);
    }
}

#[cfg(test)]
mod tuple_with_two_items_system_param_tests {
    use super::*;
    use crate::components::internal::ComponentGuardBorrow;
    use crate::components_mut::internal::ComponentMutGuardBorrow;
    use crate::storages::archetypes::{ArchetypeStorage, FilteredArchetypeIdxIter};
    use crate::storages::core::CoreStorage;
    use crate::storages::systems::Access;
    use crate::{SystemInfo, World};

    #[test]
    fn retrieve_properties_with_entity_action() {
        let mut core = CoreStorage::default();

        let properties = <(&u32, World<'_>)>::properties(&mut core);

        assert_eq!(properties.component_types.len(), 1);
        assert_eq!(properties.component_types[0].access, Access::Read);
        assert_eq!(properties.component_types[0].type_idx, 0.into());
        assert!(properties.has_entity_actions);
        let archetype_filter = ArchetypeFilter::Intersection(ne_vec![0.into()]);
        assert_eq!(properties.archetype_filter, archetype_filter);
    }

    #[test]
    fn retrieve_properties_without_entity_action() {
        let mut core = CoreStorage::default();

        let properties = <(&u32, &mut i64)>::properties(&mut core);

        assert_eq!(properties.component_types.len(), 2);
        assert_eq!(properties.component_types[0].access, Access::Read);
        assert_eq!(properties.component_types[0].type_idx, 0.into());
        assert_eq!(properties.component_types[1].access, Access::Write);
        assert_eq!(properties.component_types[1].type_idx, 1.into());
        assert!(!properties.has_entity_actions);
        let archetype_filter = ArchetypeFilter::Intersection(ne_vec![0.into(), 1.into()]);
        assert_eq!(properties.archetype_filter, archetype_filter);
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
        let info = SystemInfo {
            filtered_component_type_idxs: &[0.into()],
            archetype_filter: &ArchetypeFilter::All,
            item_count: 1,
        };

        let mut guard = <(&u32, &mut i64)>::lock(data, info);
        let guard_borrow = <(&u32, &mut i64)>::borrow_guard(&mut guard);

        let expected_guard = ti_vec![ti_vec![], ti_vec![], ti_vec![20_u32]];
        assert_eq!(guard_borrow.0.components, &expected_guard);
        let expected_guard = ti_vec![ti_vec![], ti_vec![], ti_vec![10_i64]];
        assert_eq!(guard_borrow.1.components, &expected_guard);
    }

    #[test]
    fn retrieve_stream() {
        let components1 = ti_vec![ti_vec![10], ti_vec![20]];
        let mut components2 = ti_vec![ti_vec![30], ti_vec![40]];
        let archetype_idxs = [0.into(), 1.into()];
        let archetype_type_idxs = ti_vec![vec![0.into()]; 2];
        let sorted_archetype_idxs =
            FilteredArchetypeIdxIter::new(&archetype_idxs, &archetype_type_idxs);
        let mut guard_borrow = (
            ComponentGuardBorrow {
                components: &components1,
                item_count: 2,
                sorted_archetype_idxs: sorted_archetype_idxs.clone(),
            },
            ComponentMutGuardBorrow {
                components: &mut components2,
                item_count: 2,
                sorted_archetype_idxs,
            },
        );

        let mut stream = <(&u32, &mut i64)>::stream(&mut guard_borrow);

        let next = <(&u32, &mut i64)>::stream_next(&mut stream);
        assert_eq!(next, Some((&10, &mut 30)));
        let next = <(&u32, &mut i64)>::stream_next(&mut stream);
        assert_eq!(next, Some((&20, &mut 40)));
        assert_eq!(<(&u32, &mut i64)>::stream_next(&mut stream), None);
    }

    #[test]
    fn retrieve_query_iter() {
        let components1 = ti_vec![ti_vec![10], ti_vec![20]];
        let mut components2 = ti_vec![ti_vec![30], ti_vec![40]];
        let archetype_idxs = [0.into(), 1.into()];
        let archetype_type_idxs = ti_vec![vec![0.into()]; 2];
        let sorted_archetype_idxs =
            FilteredArchetypeIdxIter::new(&archetype_idxs, &archetype_type_idxs);
        let guard_borrow = (
            ComponentGuardBorrow {
                components: &components1,
                item_count: 2,
                sorted_archetype_idxs: sorted_archetype_idxs.clone(),
            },
            ComponentMutGuardBorrow {
                components: &mut components2,
                item_count: 2,
                sorted_archetype_idxs,
            },
        );

        let mut iter = <(&u32, &mut i64)>::query_iter(&guard_borrow);

        assert_eq!(iter.next(), Some((&10, &30)));
        assert_eq!(iter.next(), Some((&20, &40)));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn retrieve_query_iter_mut() {
        let components1 = ti_vec![ti_vec![10], ti_vec![20]];
        let mut components2 = ti_vec![ti_vec![30], ti_vec![40]];
        let archetype_idxs = [0.into(), 1.into()];
        let archetype_type_idxs = ti_vec![vec![0.into()]; 2];
        let sorted_archetype_idxs =
            FilteredArchetypeIdxIter::new(&archetype_idxs, &archetype_type_idxs);
        let mut guard_borrow = (
            ComponentGuardBorrow {
                components: &components1,
                item_count: 2,
                sorted_archetype_idxs: sorted_archetype_idxs.clone(),
            },
            ComponentMutGuardBorrow {
                components: &mut components2,
                item_count: 2,
                sorted_archetype_idxs,
            },
        );

        let mut iter = <(&u32, &mut i64)>::query_iter_mut(&mut guard_borrow);

        assert_eq!(iter.next(), Some((&10, &mut 30)));
        assert_eq!(iter.next(), Some((&20, &mut 40)));
        assert_eq!(iter.next(), None);
    }
}

#[cfg(test)]
mod tuple_with_more_than_two_items_system_param_tests {
    use super::*;
    use crate::components::internal::ComponentGuardBorrow;
    use crate::components_mut::internal::ComponentMutGuardBorrow;
    use crate::storages::archetypes::{ArchetypeStorage, FilteredArchetypeIdxIter};
    use crate::storages::core::CoreStorage;
    use crate::storages::systems::Access;
    use crate::SystemInfo;

    #[test]
    fn retrieve_properties() {
        let mut core = CoreStorage::default();

        let properties = <(&u32, &mut i64, &i16)>::properties(&mut core);

        assert_eq!(properties.component_types.len(), 3);
        assert_eq!(properties.component_types[0].access, Access::Read);
        assert_eq!(properties.component_types[0].type_idx, 0.into());
        assert_eq!(properties.component_types[1].access, Access::Write);
        assert_eq!(properties.component_types[1].type_idx, 1.into());
        assert_eq!(properties.component_types[2].access, Access::Read);
        assert_eq!(properties.component_types[2].type_idx, 2.into());
        assert!(!properties.has_entity_actions);
        let archetype_filter = ArchetypeFilter::Intersection(ne_vec![0.into(), 1.into(), 2.into()]);
        assert_eq!(properties.archetype_filter, archetype_filter);
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
        let info = SystemInfo {
            filtered_component_type_idxs: &[0.into()],
            archetype_filter: &ArchetypeFilter::All,
            item_count: 1,
        };

        let mut guard = <(&u32, &mut i64, &i16)>::lock(data, info);
        let guard_borrow = <(&u32, &mut i64, &i16)>::borrow_guard(&mut guard);

        let expected_guard = ti_vec![ti_vec![], ti_vec![], ti_vec![], ti_vec![20_u32]];
        assert_eq!(guard_borrow.0.components, &expected_guard);
        let expected_guard = ti_vec![ti_vec![], ti_vec![], ti_vec![], ti_vec![10_i64]];
        assert_eq!(guard_borrow.1.components, &expected_guard);
        let expected_guard = ti_vec![ti_vec![], ti_vec![], ti_vec![], ti_vec![30_i16]];
        assert_eq!(guard_borrow.2.components, &expected_guard);
    }

    #[test]
    fn retrieve_stream() {
        let components1 = ti_vec![ti_vec![10], ti_vec![20]];
        let mut components2 = ti_vec![ti_vec![30], ti_vec![40]];
        let components3 = ti_vec![ti_vec![50], ti_vec![60]];
        let archetype_idxs = [0.into(), 1.into()];
        let archetype_type_idxs = ti_vec![vec![0.into()]; 2];
        let sorted_archetype_idxs =
            FilteredArchetypeIdxIter::new(&archetype_idxs, &archetype_type_idxs);
        let mut guard_borrow = (
            ComponentGuardBorrow {
                components: &components1,
                item_count: 2,
                sorted_archetype_idxs: sorted_archetype_idxs.clone(),
            },
            ComponentMutGuardBorrow {
                components: &mut components2,
                item_count: 2,
                sorted_archetype_idxs: sorted_archetype_idxs.clone(),
            },
            ComponentGuardBorrow {
                components: &components3,
                item_count: 2,
                sorted_archetype_idxs,
            },
        );

        let mut stream = <(&u32, &mut i64, &i16)>::stream(&mut guard_borrow);

        let next = <(&u32, &mut i64, &i16)>::stream_next(&mut stream);
        assert_eq!(next, Some((&10, &mut 30, &50)));
        let next = <(&u32, &mut i64, &i16)>::stream_next(&mut stream);
        assert_eq!(next, Some((&20, &mut 40, &60)));
        assert_eq!(<(&u32, &mut i64, &i16)>::stream_next(&mut stream), None);
    }

    #[test]
    fn retrieve_query_iter() {
        let components1 = ti_vec![ti_vec![10], ti_vec![20]];
        let mut components2 = ti_vec![ti_vec![30], ti_vec![40]];
        let components3 = ti_vec![ti_vec![50], ti_vec![60]];
        let archetype_idxs = [0.into(), 1.into()];
        let archetype_type_idxs = ti_vec![vec![0.into()]; 2];
        let sorted_archetype_idxs =
            FilteredArchetypeIdxIter::new(&archetype_idxs, &archetype_type_idxs);
        let guard_borrow = (
            ComponentGuardBorrow {
                components: &components1,
                item_count: 2,
                sorted_archetype_idxs: sorted_archetype_idxs.clone(),
            },
            ComponentMutGuardBorrow {
                components: &mut components2,
                item_count: 2,
                sorted_archetype_idxs: sorted_archetype_idxs.clone(),
            },
            ComponentGuardBorrow {
                components: &components3,
                item_count: 2,
                sorted_archetype_idxs,
            },
        );

        let mut iter = <(&u32, &mut i64, &i16)>::query_iter(&guard_borrow);

        assert_eq!(iter.next(), Some((&10, &30, &50)));
        assert_eq!(iter.next(), Some((&20, &40, &60)));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn retrieve_query_iter_mut() {
        let components1 = ti_vec![ti_vec![10], ti_vec![20]];
        let mut components2 = ti_vec![ti_vec![30], ti_vec![40]];
        let components3 = ti_vec![ti_vec![50], ti_vec![60]];
        let archetype_idxs = [0.into(), 1.into()];
        let archetype_type_idxs = ti_vec![vec![0.into()]; 2];
        let sorted_archetype_idxs =
            FilteredArchetypeIdxIter::new(&archetype_idxs, &archetype_type_idxs);
        let mut guard_borrow = (
            ComponentGuardBorrow {
                components: &components1,
                item_count: 2,
                sorted_archetype_idxs: sorted_archetype_idxs.clone(),
            },
            ComponentMutGuardBorrow {
                components: &mut components2,
                item_count: 2,
                sorted_archetype_idxs: sorted_archetype_idxs.clone(),
            },
            ComponentGuardBorrow {
                components: &components3,
                item_count: 2,
                sorted_archetype_idxs,
            },
        );

        let mut iter = <(&u32, &mut i64, &i16)>::query_iter_mut(&mut guard_borrow);

        assert_eq!(iter.next(), Some((&10, &mut 30, &50)));
        assert_eq!(iter.next(), Some((&20, &mut 40, &60)));
        assert_eq!(iter.next(), None);
    }
}
