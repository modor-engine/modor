use crate::{
    Const, EntityPartSystemParam, MultipleSystemParams, Mut, NotEnoughEntityPartSystemParam,
    NotMandatoryComponentSystemParam, System, SystemParam,
};
use std::any::Any;
use std::marker::PhantomData;

pub struct SystemStaticChecker<'a, 'b, S, T>(S, PhantomData<(&'a T, &'b T)>);

impl<'a, 'b, S, T> SystemStaticChecker<'a, 'b, S, T>
where
    S: System<'a, 'b, T>,
{
    pub fn new(system: S) -> Self {
        Self(system, PhantomData)
    }
}

pub trait SystemWithCorrectParams<S, Z> {
    fn check_statically(self) -> S;
}

impl<'a, 'b, S, Z> SystemWithCorrectParams<S, Z> for SystemStaticChecker<'a, 'b, S, Z>
where
    S: System<'a, 'b, Z>,
{
    fn check_statically(self) -> S {
        self.0
    }
}

pub trait SystemWithMissingComponentParam<S, Z> {
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

pub trait SystemWithIncompatibleParams<S, Z> {
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
            A: IncompatibleSystemParam<($($params,)*), Z>,
        {
        }

        impl<T, $param, $($params,)* Z>
            IncompatibleMultipleSystemParams<(((),), Z, ($param, $($params),*))>
            for T
        where
            T: MultipleSystemParams<TupleSystemParams = ($param, $($params),*)>,
            ($($params,)*): IncompatibleMultipleSystemParams<Z>,
        {
        }
    };
}

run_for_tuples!(impl_incompatible_multiple_system_params);
