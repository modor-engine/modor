use typed_index_collections::TiVec;

macro_rules! ti_vec {
   () => (
        ::typed_index_collections::TiVec::<_, _>::from(vec![])
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
mod test_utils {
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn set_missing_value_in_ti_vec() {
        let mut vec = TiVec::<usize, usize>::new();

        set_value(&mut vec, 2, 10);

        assert_eq!(vec, ti_vec![0, 0, 10]);
    }

    #[test]
    fn set_existing_value_in_ti_vec() {
        let mut vec: TiVec<usize, usize> = ti_vec![0, 0, 10];

        set_value(&mut vec, 1, 20);

        assert_eq!(vec, ti_vec![0, 20, 10]);
    }
}
