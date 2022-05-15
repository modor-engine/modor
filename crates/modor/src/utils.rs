use std::cmp::Ordering;
use typed_index_collections::TiVec;

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
