use std::cmp::Ordering;
use typed_index_collections::TiVec;

macro_rules! ti_vec {
   () => (
        ::typed_index_collections::TiVec::<_, _>::from(vec![])
    );
    ($elem:expr; $n:expr) => (
        ::typed_index_collections::TiVec::<_, _>::from(vec![$elem; $n])
    );
    ($($x:expr),+ $(,)?) => (
        ::typed_index_collections::TiVec::<_, _>::from(vec![$($x),+])
    );
}

macro_rules! idx_type {
    ($visibility:vis $name:ident) => {
        #[derive(Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
        $visibility struct $name($visibility usize);

        impl From<usize> for $name {
            #[inline]
            fn from(idx: usize) -> Self {
                Self(idx)
            }
        }

        impl From<$name> for usize {
            #[inline]
            fn from(idx: $name) -> Self {
                idx.0
            }
        }
    };
}

pub(crate) fn set_value<K, V>(vec: &mut TiVec<K, V>, idx: K, value: V)
where
    usize: From<K>,
    K: From<usize>,
    V: Default,
{
    let idx = usize::from(idx);
    (vec.len()..=idx).for_each(|_| vec.push(V::default()));
    vec[K::from(idx)] = value;
}

pub(crate) fn get_both_mut<K, T>(
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
    } else {
        match key1.cmp(&key2) {
            Ordering::Equal => (data.get_mut(key1), None),
            Ordering::Less => {
                let (left, right) = data.split_at_mut(key2);
                (Some(&mut left[key1]), Some(&mut right[K::from(0)]))
            }
            Ordering::Greater => {
                let (left, right) = data.split_at_mut(key1);
                (Some(&mut right[K::from(0)]), Some(&mut left[key2]))
            }
        }
    }
}

pub(crate) fn merge<T, const N: usize>(vectors: [Vec<T>; N]) -> Vec<T> {
    let mut merged_vec = Vec::new();
    for vec in vectors {
        merged_vec.extend(vec);
    }
    merged_vec
}

macro_rules! run_for_tuples_with_idxs {
    ($macro:ident) => {
        run_for_tuples_with_idxs!(
            @internal
            $macro,
            ((A, 0)),
            ((B, 1), (C, 2), (D, 3), (E, 4), (F, 5), (G, 6), (H, 7), (I, 8), (J, 9))
        );
    };
    (
        @internal
        $macro:ident,
        ($(($generics:ident, $indexes:tt)),+),
        (($next_generic:ident, $next_index:tt) $(,($last_generics:ident, $last_index:tt))*)
    ) => {
        $macro!($(($generics, $indexes)),+);
        run_for_tuples_with_idxs!(
            @internal
            $macro,
            ($(($generics, $indexes)),+, ($next_generic, $next_index)),
            ($(($last_generics, $last_index)),*)
        );
    };
    (@internal $macro:ident, ($(($generics:ident, $indexes:tt)),+), ()) => {
        $macro!($(($generics, $indexes)),+);
    };
}

#[cfg(test)]
#[macro_use]
pub(crate) mod test_utils {
    use std::fmt::Debug;
    macro_rules! assert_panics {
        ($expression:expr) => {{
            assert!(
                std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    let _ = $expression;
                }))
                .is_err(),
                "assertion failed: expression `{}` has not panicked",
                stringify!($expression),
            );
        }};
    }

    pub(crate) fn assert_iter<T, E, I1, I2>(mut actual: I1, expected: E)
    where
        T: PartialEq + Debug,
        I1: Iterator<Item = T> + ExactSizeIterator,
        I2: ExactSizeIterator + Iterator<Item = T>,
        E: IntoIterator<Item = T, IntoIter = I2>,
    {
        let expected_iter = expected.into_iter();
        let expected_len = expected_iter.len();
        for (pos, expected_item) in expected_iter.enumerate() {
            assert_eq!(actual.len(), expected_len - pos);
            assert_eq!(actual.next(), Some(expected_item));
        }
        assert_eq!(actual.len(), 0);
        assert_eq!(actual.next(), None);
    }

    pub(crate) fn assert_dyn_iter<T, E, I1>(mut actual: I1, expected: E)
    where
        T: PartialEq + Debug,
        I1: Iterator<Item = T>,
        E: IntoIterator<Item = T>,
    {
        for expected_item in expected {
            assert_eq!(actual.next(), Some(expected_item));
        }
        assert_eq!(actual.next(), None);
    }

    macro_rules! create_entity_type {
        ($name:ident) => {
            create_entity_type!($name, ());
        };
        ($name:ident, $type:ty) => {
            #[derive(Debug, PartialEq, Clone)]
            struct $name(u32);

            #[allow(unused_qualifications)]
            impl crate::EntityMainComponent for $name {
                type Type = $type;
            }
        };
    }
}

#[cfg(test)]
mod tests {
    use crate::utils::{get_both_mut, merge, set_value};
    use typed_index_collections::TiVec;

    #[test]
    fn set_values() {
        let mut vec = TiVec::<usize, usize>::new();
        set_value(&mut vec, 2, 10);
        set_value(&mut vec, 1, 20);
        assert_eq!(vec, ti_vec![0, 20, 10]);
    }

    #[test]
    fn retrieve_both_mut() {
        let mut vec: TiVec<usize, u32> = ti_vec![10, 20, 30, 40];
        assert_eq!(get_both_mut(&mut vec, 0, 1), (Some(&mut 10), Some(&mut 20)));
        assert_eq!(get_both_mut(&mut vec, 1, 0), (Some(&mut 20), Some(&mut 10)));
        assert_eq!(get_both_mut(&mut vec, 1, 3), (Some(&mut 20), Some(&mut 40)));
        assert_eq!(get_both_mut(&mut vec, 3, 1), (Some(&mut 40), Some(&mut 20)));
        assert_eq!(get_both_mut(&mut vec, 4, 1), (None, Some(&mut 20)));
        assert_eq!(get_both_mut(&mut vec, 0, 4), (Some(&mut 10), None));
        assert_eq!(get_both_mut(&mut vec, 4, 5), (None, None));
        assert_eq!(get_both_mut(&mut vec, 1, 1), (Some(&mut 20), None));
    }

    #[test]
    fn merge_vectors() {
        assert_eq!(merge::<i32, 0>([]), vec![]);
        assert_eq!(merge([vec![1, 2, 3]]), vec![1, 2, 3]);
        assert_eq!(merge([vec![1, 2], vec![3, 4]]), vec![1, 2, 3, 4]);
    }
}
