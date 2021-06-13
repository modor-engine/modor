use self::internal::Sealed;
use crate::system_params::internal::SealedSystemParam;
use crate::{
    Const, EntityPartSystemParam, MultipleSystemParams, Mut, NotEnoughEntityPartSystemParam,
    NotMandatoryComponentSystemParam, QuerySystemParam, System, SystemParam,
};
use std::any::Any;
use std::marker::PhantomData;

// TODO: move internal pub types in "internal" modules
// TODO: split module

/// Characterise any system.
///
/// See documentation of [`system!`](crate::system!) macro for more information.
pub trait SystemWithParams<S, T>: Sized + Sealed {
    #[doc(hidden)]
    fn check_component_params(self) -> Self {
        self
    }

    #[doc(hidden)]
    fn check_query_component_params(self) -> Self {
        self
    }

    #[doc(hidden)]
    fn check_param_compatibility(self) -> Self {
        self
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[doc(hidden)]
pub struct SystemComponentParamChecker<'a, 'b, S, T>(S, PhantomData<(&'a T, &'b T)>);

impl<'a, 'b, S, T> SystemComponentParamChecker<'a, 'b, S, T>
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

impl<'a, 'b, S, T> Sealed for SystemComponentParamChecker<'a, 'b, S, T> {}

impl<'a, 'b, S, T> SystemWithParams<S, T> for SystemComponentParamChecker<'a, 'b, S, T> where
    S: System<'a, 'b, T>
{
}

/// Characterise a system with a missing component parameter.
///
/// See documentation of [`system!`](crate::system!) macro for more information.
pub trait SystemWithMissingComponentParam<S, Z>: Sized + Sealed {
    #[doc(hidden)]
    fn check_component_params(self) -> Self {
        self
    }
}

macro_rules! impl_only_optional_params_system_check {
    ($param:ident $(,$params:ident)*) => {
        impl<'a, 'b, 'c, S, $param, $($params,)*>
            SystemWithMissingComponentParam<S, ()>
            for SystemComponentParamChecker<'a, 'b, S, ($param, $($params),*)>
        where
            $param: NotEnoughEntityPartSystemParam,
            $($params: NotMandatoryComponentSystemParam,)*
        {
        }

        impl<'a, 'b, 'c, S1, S2, $param, $($params,)* Z>
            SystemWithMissingComponentParam<S1, (Z, S2,)>
            for SystemComponentParamChecker<'a, 'b, S1, ($param, $($params),*)>
        where
            $param: NotMandatoryComponentSystemParam,
            SystemComponentParamChecker<'c, 'c, S2, ($($params,)*)>:
                SystemWithMissingComponentParam<S2, Z>,
            $($params: 'c,)*
        {
        }
    };
}

run_for_tuples!(impl_only_optional_params_system_check);

////////////////////////////////////////////////////////////////////////////////////////////////////

#[doc(hidden)]
pub struct SystemQueryComponentParamChecker<'a, 'b, S, T>(S, PhantomData<(&'a T, &'b T)>);

impl<'a, 'b, S, T> SystemQueryComponentParamChecker<'a, 'b, S, T>
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

impl<'a, 'b, S, T> Sealed for SystemQueryComponentParamChecker<'a, 'b, S, T> {}

impl<'a, 'b, S, T> SystemWithParams<S, T> for SystemQueryComponentParamChecker<'a, 'b, S, T> where
    S: System<'a, 'b, T>
{
}

/// Characterise a system with a query that has a missing component parameter.
///
/// See documentation of [`system!`](crate::system!) macro for more information.
pub trait SystemWithQueryWithMissingComponentParam<S, Z>: Sized + Sealed {
    #[doc(hidden)]
    fn check_query_component_params(self) -> Self {
        self
    }
}

macro_rules! impl_only_optional_params_query_check {
    ($param:ident $(,$params:ident)*) => {
        impl<'a, 'b, 'c, S1, S2, T, $param, $($params,)* Z>
            SystemWithQueryWithMissingComponentParam<S1, ((), Z, S2, T)>
            for SystemQueryComponentParamChecker<'a, 'b, S1, ($param, $($params),*)>
        where
            $param: QuerySystemParam + MultipleSystemParams<TupleSystemParams = T>,
            SystemComponentParamChecker<'c, 'c, S2, T>: SystemWithMissingComponentParam<S2, Z>,
            T: 'c,
        {
        }

        impl<'a, 'b, 'c, S1, S2, T, $param, $($params,)* Z>
            SystemWithQueryWithMissingComponentParam<S1, (((),), Z, S2, T)>
            for SystemQueryComponentParamChecker<'a, 'b, S1, ($param, $($params),*)>
        where
            $param: MultipleSystemParams<TupleSystemParams = T>,
            SystemQueryComponentParamChecker<'c, 'c, S2, T>:
                SystemWithQueryWithMissingComponentParam<S2, Z>,
            T: 'c
        {
        }

        impl<'a, 'b, 'c, S1, S2, $param, $($params,)* Z>
            SystemWithQueryWithMissingComponentParam<S1, ((((),),), Z, S2)>
            for SystemQueryComponentParamChecker<'a, 'b, S1, ($param, $($params),*)>
        where
            SystemQueryComponentParamChecker<'c, 'c, S2, ($($params,)*)>:
                SystemWithQueryWithMissingComponentParam<S2, Z>,
            $($params: 'c,)*
        {
        }
    };
}

run_for_tuples!(impl_only_optional_params_query_check);

////////////////////////////////////////////////////////////////////////////////////////////////////

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

impl<'a, 'b, S, T> Sealed for SystemParamCompatibilityChecker<'a, 'b, S, T> {}

impl<'a, 'b, S, T> SystemWithParams<S, T> for SystemParamCompatibilityChecker<'a, 'b, S, T> where
    S: System<'a, 'b, T>
{
}

/// Characterise a system with incompatible parameters.
///
/// See documentation of [`system!`](crate::system!) macro for more information.
pub trait SystemWithIncompatibleParams<S, Z>: Sized + Sealed {
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

////////////////////////////////////////////////////////////////////////////////////////////////////

mod internal {
    pub trait Sealed {}
}

#[cfg(test)]
mod system_component_param_checker {
    use super::*;

    assert_impl_all!(SystemComponentParamChecker<'_, '_, fn(&u32), (&u32,)>: Sync, Send);
}

#[cfg(test)]
mod system_query_component_param_checker {
    use super::*;

    assert_impl_all!(SystemQueryComponentParamChecker<'_, '_, fn(&u32), (&u32,)>: Sync, Send);
}

#[cfg(test)]
mod system_param_compatibility_checker {
    use super::*;

    assert_impl_all!(SystemParamCompatibilityChecker<'_, '_, fn(&u32), (&u32,)>: Sync, Send);
}
