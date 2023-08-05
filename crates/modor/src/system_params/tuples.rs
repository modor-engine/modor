use crate::storages::archetypes::EntityLocation;
use crate::storages::core::CoreStorage;
use crate::storages::systems::SystemProperties;
use crate::system_params::tuples::internal::{EmptyTupleGuard, EmptyTupleIter};
use crate::systems::context::SystemContext;
use crate::tuples::internal::EmptyTupleGuardBorrow;
use crate::utils;
use crate::{QuerySystemParam, QuerySystemParamWithLifetime, SystemParam, SystemParamWithLifetime};
use std::iter::{Map, Zip};

impl<'a> SystemParamWithLifetime<'a> for () {
    type Param = ();
    type Guard = EmptyTupleGuard;
    type GuardBorrow = EmptyTupleGuardBorrow;
    type Stream = EmptyTupleIter;
}

impl SystemParam for () {
    type Filter = ();
    type InnerTuple = Self;

    fn properties(_core: &mut CoreStorage) -> SystemProperties {
        SystemProperties {
            component_types: vec![],
            can_update: false,
            mutation_component_type_idxs: vec![],
        }
    }

    fn lock(context: SystemContext<'_>) -> <Self as SystemParamWithLifetime<'_>>::Guard {
        EmptyTupleGuard::new(context)
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
            type Filter = ($($params::Filter,)+);
            type InnerTuple = Self;

            fn properties(core: &mut CoreStorage) -> SystemProperties {
                let properties = ($($params::properties(core),)+);
                SystemProperties {
                    component_types: utils::merge([$(properties.$indexes.component_types),+]),
                    can_update: [$(properties.$indexes.can_update),+].into_iter().any(|b| b),
                    mutation_component_type_idxs:
                        utils::merge([$(properties.$indexes.mutation_component_type_idxs),+]),
                }
            }

            fn lock(context: SystemContext<'_>) -> <Self as SystemParamWithLifetime<'_>>::Guard {
                ($($params::lock(context),)+)
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
    use crate::systems::context::SystemContext;
    use std::ops::Range;

    pub struct EmptyTupleGuard {
        item_count: usize,
    }

    impl EmptyTupleGuard {
        pub(crate) fn new(context: SystemContext<'_>) -> Self {
            Self {
                item_count: context.item_count,
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
