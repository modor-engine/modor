use self::internal::Sealed;
use crate::system_params::internal::SealedSystemParam;
use crate::{
    Const, EntityPartSystemParam, MultipleSystemParams, Mut, NotEnoughEntityPartSystemParam,
    NotMandatoryComponentSystemParam, System, SystemParam,
};
use std::any::Any;
use std::marker::PhantomData;

// TODO: remove this type
#[doc(hidden)]
pub struct SystemStaticChecker<'a, 'b, S, T>(S, PhantomData<(&'a T, &'b T)>);

impl<'a, 'b, S, T> SystemStaticChecker<'a, 'b, S, T>
where
    S: System<'a, 'b, T>,
{
    #[doc(hidden)]
    pub fn new(system: S) -> Self {
        Self(system, PhantomData)
    }
}

impl<'a, 'b, S, T> Sealed for SystemStaticChecker<'a, 'b, S, T> {}

// TODO: remove this trait
/// Characterise any system.
///
/// See documentation of [`system!`](crate::system!) macro for more information.
pub trait SystemWithParams<S, T>: Sealed {
    #[doc(hidden)]
    fn check_statically(self) -> S;
}

impl<'a, 'b, S, T> SystemWithParams<S, T> for SystemStaticChecker<'a, 'b, S, T>
where
    S: System<'a, 'b, T>,
{
    fn check_statically(self) -> S {
        self.0
    }
}

/// Characterise a system with a missing component parameter.
///
/// See documentation of [`system!`](crate::system!) macro for more information.
pub trait SystemWithMissingComponentParam<S, Z>: Sealed {
    #[doc(hidden)]
    fn check_statically(self) -> S;
}

macro_rules! impl_only_optional_params_system_check {
    ($param:ident $(,$params:ident)*) => {
        impl<'a, 'b, 'c, S, $param, $($params,)* Z>
            SystemWithMissingComponentParam<S, (Z, ())>
            for SystemStaticChecker<'a, 'b, S, ($param, $($params),*)>
        where
            $param: NotEnoughEntityPartSystemParam,
            $($params: NotMandatoryComponentSystemParam,)*
        {
            fn check_statically(self) -> S {
                self.0
            }
        }

        impl<'a, 'b, 'c, S1, S2, $param, $($params,)* Z>
            SystemWithMissingComponentParam<S1, (Z, S2, ())>
            for SystemStaticChecker<'a, 'b, S1, ($param, $($params),*)>
        where
            $param: NotMandatoryComponentSystemParam,
            SystemStaticChecker<'c, 'c, S2, ($($params,)*)>: SystemWithMissingComponentParam<S2, Z>,
            $($params: 'c,)*
        {
            fn check_statically(self) -> S1 {
                self.0
            }
        }
    };
}

run_for_tuples!(impl_only_optional_params_system_check);

/// Characterise a system with incompatible parameters.
///
/// See documentation of [`system!`](crate::system!) macro for more information.
pub trait SystemWithIncompatibleParams<S, Z>: Sealed {
    #[doc(hidden)]
    fn check_statically(self) -> S;
}

macro_rules! impl_incompatibility_system_check {
    ($param:ident $(,$params:ident)*) => {
        impl<'a, 'b, S, $param, $($params,)* Z>
            SystemWithIncompatibleParams<S, ((), Z, ($param, $($params),*))>
            for SystemStaticChecker<'a, 'b, S, ($param, $($params),*)>
        where
            S: System<'a, 'b, ($param, $($params),*)>,
            $param: IncompatibleSystemParam<($($params,)*), Z>,
        {
            fn check_statically(self) -> S {
                self.0
            }
        }

        impl<'a, 'b, S, $param, $($params,)* Z>
            SystemWithIncompatibleParams<S, (((),), Z, ($param, $($params),*))>
            for SystemStaticChecker<'a, 'b, S, ($param, $($params),*)>
        where
            S: System<'a, 'b, ($param, $($params),*)>,
            $param: IncompatibleMultipleSystemParams<Z>,
        {
            fn check_statically(self) -> S {
                self.0
            }
        }

        impl<'a, 'b, 'c, S1, S2, $param, $($params,)* Z>
            SystemWithIncompatibleParams<S1, ((), Z, ($param, $($params),*), S2)>
            for SystemStaticChecker<'a, 'b, S1, ($param, $($params),*)>
        where
            S1: System<'a, 'b, ($param, $($params),*)>,
            S2: System<'c, 'c, ($($params,)*)>,
            SystemStaticChecker<'c, 'c, S2, ($($params,)*)>: SystemWithIncompatibleParams<S2, Z>,
            $($params: 'c,)*
        {
            fn check_statically(self) -> S1 {
                self.0
            }
        }
    };
}

run_for_tuples!(impl_incompatibility_system_check);

#[doc(hidden)]
pub trait IncompatibleSystemParam<T, Z>: SealedSystemParam {}

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

#[doc(hidden)]
pub trait IncompatibleMultipleSystemParams<Z>: SealedSystemParam {}

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

mod internal {
    pub trait Sealed {}
}

#[cfg(test)]
mod system_static_checker_tests {
    use super::*;

    assert_impl_all!(SystemStaticChecker<'_, '_, fn(&u32), (&u32,)>: Sync, Send);
}
