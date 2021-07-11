use crate::external::systems::definition::internal::ArchetypeInfo;
use crate::external::systems::params::{SystemParam, SystemParamData, SystemParamDataMut};
use crate::SystemData;
use std::any::TypeId;
use std::iter;
use std::iter::{Map, Repeat, Take, Zip};

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

macro_rules! tuple_iter_type {
    () => {
        EmptyTupleIter
    };
    ($(($params:ident, $indexes:tt)),*) => {
        Map<
            nested_zip!($($params::Iter),*),
            fn(nested_tuple_type!($($params::Const),*)) -> ($($params::Const,)*)
        >
    };
}

macro_rules! tuple_iter_mut_type {
    () => {
        EmptyTupleIter
    };
    ($(($params:ident, $indexes:tt)),*) => {
        Map<
            nested_zip!($($params::IterMut),*),
            fn(nested_tuple_type!($($params),*)) -> ($($params,)*)
        >
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
}

macro_rules! nested_zip {
    ($($params:path),+) => {
        nested_zip!(@internal (), ($($params),+))
    };
    (@internal ($($lefts:path),*), ($right:path $(,$rights:path)+)) => {
        nested_zip!(@internal ($($lefts,)* $right), ($($rights),+))
    };
    (@internal ($($lefts:path),+), ($right:path)) => {
        Zip<nested_zip!(@internal (), ($($lefts),+)), $right>
    };
    (@internal (), ($right:path)) => {
        $right
    };
}

macro_rules! tuple_iter {
    ($data:ident) => {
        EmptyTupleIter::new($data.item_count)
    };
    ($data:ident, ($param:ident, $index:tt) $(,($params:ident, $indexes:tt))*) => {
        $param::iter(map_system_param_data!($data, $index))
            $(.zip($params::iter(map_system_param_data!($data, $indexes))))*
            .map(|nested_tuple!($param $(, $params)*)| ($param, $($params),*))
    };
}

macro_rules! tuple_iter_mut {
    ($data:ident) => {
        EmptyTupleIter::new($data.item_count)
    };
    ($data:ident, ($param:ident, $index:tt) $(,($params:ident, $indexes:tt))*) => {
        $param::iter_mut(map_system_param_data_mut!($data, $index))
            $(.zip($params::iter_mut(map_system_param_data_mut!($data, $indexes))))*
            .map(|nested_tuple!($param $(, $params)*)| ($param, $($params),*))
    };
}

macro_rules! impl_system_param_for_tuple {
    ($(($params:ident, $indexes:tt)),*) => {
        impl<'a, 'b $(, $params)*> SystemParam<'a, 'b> for ($($params,)*)
        where
            'a: 'b,
            $($params: SystemParam<'a, 'b>,)*
        {
            type Guard = ($($params::Guard,)*);
            type Const = ($($params::Const,)*);
            type Iter = tuple_iter_type!($(($params, $indexes)),*);
            type IterMut = tuple_iter_mut_type!($(($params, $indexes)),*);

            #[allow(unused_mut)]
            fn mandatory_component_types() -> Vec<TypeId> {
                let mut types = Vec::new();
                $(types.extend($params::mandatory_component_types());)*
                types
            }

            #[allow(unused_variables)]
            fn lock(data: &'a SystemData<'_>) -> Self::Guard {
                ($($params::lock(data),)*)
            }

            #[allow(unused_variables)]
            fn item_count(guard: &Self::Guard, archetypes: &[ArchetypeInfo]) -> usize {
                0$(.max($params::item_count(&guard.$indexes, archetypes)))*
            }

            #[allow(non_snake_case)]
            fn iter(data: SystemParamData<'b, Self::Guard>) -> Self::Iter {
                tuple_iter!(data $(, ($params, $indexes))*)
            }

            #[allow(non_snake_case)]
            fn iter_mut(data: SystemParamDataMut<'b, Self::Guard>) -> Self::IterMut {
                tuple_iter_mut!(data $(, ($params, $indexes))*)
            }
        }
    };
}

impl_system_param_for_tuple!();
run_for_tuples_with_idxs!(impl_system_param_for_tuple);

pub struct EmptyTupleIter {
    iter: Take<Repeat<()>>,
    len: usize,
}

impl EmptyTupleIter {
    fn new(len: usize) -> Self {
        Self {
            iter: iter::repeat(()).take(len),
            len,
        }
    }
}

impl Iterator for EmptyTupleIter {
    type Item = ();

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }
}

impl DoubleEndedIterator for EmptyTupleIter {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        self.next()
    }
}

impl ExactSizeIterator for EmptyTupleIter {
    fn len(&self) -> usize {
        self.len
    }
}
