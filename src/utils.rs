macro_rules! run_for_tuples {
    ($macro:ident) => {
        run_for_tuples!(@internal $macro, (A), (B, C, D, E, F, G, H, I, J));
    };
    ($macro:ident, $generic:ident $(,$generics:ident)*) => {
        run_for_tuples!(@internal $macro, ($generic), ($($generics),*));
    };
    (
        @internal
        $macro:ident,
        ($($generics:ident),+),
        ($next_generic:ident $(,$last_generics:ident)*)
    ) => {
        $macro!($($generics),+);
        run_for_tuples!(@internal $macro, ($($generics),+, $next_generic), ($($last_generics),*));
    };
    (@internal $macro:ident, ($($generics:ident),+), ()) => {
        $macro!($($generics),+);
    };
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

    macro_rules! assert_option_iter {
        ($actual_iter:expr, $expected_option_slice:expr) => {{
            assert_eq!(
                $actual_iter.map(Iterator::collect::<Vec<_>>),
                $expected_option_slice
            );
        }};
    }

    macro_rules! assert_iter {
        ($actual_iter:expr, $expected_slice:expr) => {{
            assert_eq!($actual_iter.collect::<Vec<_>>(), $expected_slice);
        }};
    }
}
