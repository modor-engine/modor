use crate::external::systems::checks::internal::SealedChecker;
use crate::external::systems::checks::param_compatibility::internal::{
    IncompatibleMultipleSystemParams, IncompatibleSystemParam,
};
use crate::{System, SystemWithParams};
use std::marker::PhantomData;

#[doc(hidden)]
pub struct SystemParamCompatibilityChecker<'a, 'b, S, T>(S, PhantomData<(&'a T, &'b T)>);

impl<'a, 'b, S, T> SystemParamCompatibilityChecker<'a, 'b, S, T>
where
    S: System<'a, 'b, T>,
{
    #[doc(hidden)]
    pub fn new(system: S) -> Self {
        Self(system, PhantomData)
    }

    #[doc(hidden)]
    pub fn into_inner(self) -> S {
        self.0
    }
}

impl<'a, 'b, S, T> SealedChecker for SystemParamCompatibilityChecker<'a, 'b, S, T> {}

impl<'a, 'b, S, T> SystemWithParams<S, T> for SystemParamCompatibilityChecker<'a, 'b, S, T> where
    S: System<'a, 'b, T>
{
}

/// Characterise a system with incompatible parameters.
///
/// See documentation of [`system!`](crate::system!) macro for more information.
pub trait SystemWithIncompatibleParams<S, Z>: Sized + SealedChecker {
    #[doc(hidden)]
    fn check_param_compatibility(self) -> Self {
        self
    }
}

macro_rules! impl_incompatibility_system_check {
    ($param:ident $(,$params:ident)*) => {
        impl<'a, 'b, S, $param, $($params,)* Z>
            SystemWithIncompatibleParams<S, ((), Z, ($param, $($params),*))>
            for SystemParamCompatibilityChecker<'a, 'b, S, ($param, $($params),*)>
        where
            S: System<'a, 'b, ($param, $($params),*)>,
            $param: IncompatibleSystemParam<($($params,)*), Z>,
        {
        }

        impl<'a, 'b, S, $param, $($params,)* Z>
            SystemWithIncompatibleParams<S, (((),), Z, ($param, $($params),*))>
            for SystemParamCompatibilityChecker<'a, 'b, S, ($param, $($params),*)>
        where
            S: System<'a, 'b, ($param, $($params),*)>,
            $param: IncompatibleMultipleSystemParams<Z>,
        {
        }

        impl<'a, 'b, 'c, S1, S2, $param, $($params,)* Z>
            SystemWithIncompatibleParams<S1, ((), Z, ($param, $($params),*), S2)>
            for SystemParamCompatibilityChecker<'a, 'b, S1, ($param, $($params),*)>
        where
            S1: System<'a, 'b, ($param, $($params),*)>,
            S2: System<'c, 'c, ($($params,)*)>,
            SystemParamCompatibilityChecker<'c, 'c, S2, ($($params,)*)>:
                SystemWithIncompatibleParams<S2, Z>,
            $($params: 'c,)*
        {
        }
    };
}

run_for_tuples!(impl_incompatibility_system_check);

mod internal {
    use crate::external::systems::params::internal::{
        Const, EntityPartSystemParam, MultipleSystemParams, Mut,
    };
    use crate::SystemParam;
    use std::any::Any;

    pub trait IncompatibleSystemParam<T, Z> {}

    impl<T, U, C> IncompatibleSystemParam<U, ((), C)> for T
    where
        T: EntityPartSystemParam<Resource = C, Mutability = Const>,
        U: EntityPartSystemParam<Resource = C, Mutability = Mut>,
        C: Any,
    {
    }

    impl<T, U, C> IncompatibleSystemParam<U, ((), C, ())> for T
    where
        T: EntityPartSystemParam<Resource = C, Mutability = Mut>,
        U: EntityPartSystemParam<Resource = C, Mutability = Const>,
        C: Any,
    {
    }

    impl<T, U, C> IncompatibleSystemParam<U, ((), C, ((),))> for T
    where
        T: EntityPartSystemParam<Resource = C, Mutability = Mut>,
        U: EntityPartSystemParam<Resource = C, Mutability = Mut>,
        C: Any,
    {
    }

    macro_rules! impl_incompatible_system_param {
        ($param:ident $(,$params:ident)*) => {
            impl<'a, 'b, $param, $($params,)* T, U, Z>
                IncompatibleSystemParam<U, (((),), Z, ($param, $($params),*))>
                for T
            where
                $param: SystemParam<'a, 'b>,
                $($params: SystemParam<'a, 'b>,)*
                T: MultipleSystemParams<TupleSystemParams = ($param, $($params),*)>,
                U: IncompatibleSystemParam<$param, Z>,
            {
            }

            impl<'a, 'b, $param, $($params,)* T, U, Z>
                IncompatibleSystemParam<U, (((),), Z, ($param, $($params),*), ())>
                for T
            where
                $param: SystemParam<'a, 'b>,
                $($params: SystemParam<'a, 'b>,)*
                T: MultipleSystemParams<TupleSystemParams = ($param, $($params),*)>,
                U: IncompatibleSystemParam<($($params,)*), Z>,
            {
            }

            impl<'a, 'b, $param, $($params,)* T, U, Z>
                IncompatibleSystemParam<T, (((),), Z, ($param, $($params),*), ((),))>
                for U
            where
                $param: SystemParam<'a, 'b>,
                $($params: SystemParam<'a, 'b>,)*
                T: MultipleSystemParams<TupleSystemParams = ($param, $($params),*)>,
                U: IncompatibleSystemParam<$param, Z>,
            {
            }

            impl<'a, 'b, $param, $($params,)* T, U, Z>
                IncompatibleSystemParam<T, (((),), Z, ($param, $($params),*), (((),),))>
                for U
            where
                $param: SystemParam<'a, 'b>,
                $($params: SystemParam<'a, 'b>,)*
                T: MultipleSystemParams<TupleSystemParams = ($param, $($params),*)>,
                U: IncompatibleSystemParam<($($params,)*), Z>,
            {
            }
        };
    }

    run_for_tuples!(impl_incompatible_system_param);

    pub trait IncompatibleMultipleSystemParams<Z> {}

    macro_rules! impl_incompatible_multiple_system_params {
        ($param:ident $(,$params:ident)*) => {
            impl<T, $param, $($params,)* Z>
                IncompatibleMultipleSystemParams<((), Z, ($param, $($params),*))>
                for T
            where
                T: MultipleSystemParams<TupleSystemParams = ($param, $($params),*)>,
                $param: IncompatibleSystemParam<($($params,)*), Z>,
            {
            }

            impl<T, $param, $($params,)* Z>
                IncompatibleMultipleSystemParams<(((),), Z, ($param, $($params),*))>
                for T
            where
                T: MultipleSystemParams<TupleSystemParams = ($param, $($params),*)>,
                $param: IncompatibleMultipleSystemParams<Z>,
            {
            }

            impl<T, $param, $($params,)* Z>
                IncompatibleMultipleSystemParams<((((),),), Z, ($param, $($params),*))>
                for T
            where
                T: MultipleSystemParams<TupleSystemParams = ($param, $($params),*)>,
                ($($params,)*): IncompatibleMultipleSystemParams<Z>,
            {
            }
        };
    }

    run_for_tuples!(impl_incompatible_multiple_system_params);
}

#[cfg(test)]
mod system_param_compatibility_checker_tests {
    use super::*;

    assert_impl_all!(SystemParamCompatibilityChecker<'_, '_, fn(&u32), (&u32,)>: Sync, Send);

    fn system_example() {}

    #[test]
    fn into_inner() {
        let checker = SystemParamCompatibilityChecker::new(system_example);

        let system = checker.into_inner();

        assert_eq!(system as usize, system_example as usize);
    }
}

#[cfg(test)]
mod system_with_incompatible_params_tests {
    use super::*;

    struct ExampleChecker(u32);

    impl SealedChecker for ExampleChecker {}

    impl SystemWithIncompatibleParams<(), ()> for ExampleChecker {}

    #[test]
    fn check_param_compatibility() {
        let checker = ExampleChecker(42);

        let run_checker = SystemWithIncompatibleParams::check_param_compatibility(checker);

        assert_eq!(run_checker.0, 42);
    }
}
