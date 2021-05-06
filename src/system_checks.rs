use crate::{
    ComponentSystemParam, ConstSystemParam, NotMandatoryComponentSystemParam, Query, QueryMut,
    System, TupleSystemParam,
};
use std::marker::PhantomData;

// TODO: only use traits defined in system_params module (and create new traits if necessary)

pub trait IncompatibleSystemParam<T, PHANTOM> {}

impl<T, U, PHANTOM> IncompatibleSystemParam<Option<T>, (PHANTOM, (), (), (), ())> for U where
    T: IncompatibleSystemParam<U, PHANTOM>
{
}

impl<T, U, PHANTOM> IncompatibleSystemParam<T, (PHANTOM,)> for Option<U> where
    T: IncompatibleSystemParam<U, PHANTOM>
{
}

impl<T> IncompatibleSystemParam<&T, ()> for &mut T {}

impl<'a, T, U, PHANTOM> IncompatibleSystemParam<&'a T, (PHANTOM,)> for QueryMut<'_, U>
where
    &'a T: IncompatibleSystemParam<U, PHANTOM>,
    U: TupleSystemParam,
{
}

impl<T> IncompatibleSystemParam<&mut T, ()> for &T {}

impl<T> IncompatibleSystemParam<&mut T, ()> for &mut T {}

impl<'a, T, U, PHANTOM> IncompatibleSystemParam<&'a mut T, (PHANTOM,)> for Query<'_, U>
where
    &'a mut T: IncompatibleSystemParam<U, PHANTOM>,
    U: ConstSystemParam + TupleSystemParam,
{
}

impl<'a, T, U, PHANTOM> IncompatibleSystemParam<&'a mut T, (PHANTOM,)> for QueryMut<'_, U>
where
    &'a mut T: IncompatibleSystemParam<U, PHANTOM>,
    U: TupleSystemParam,
{
}

impl<'a, T, U, PHANTOM> IncompatibleSystemParam<Query<'_, T>, (PHANTOM,)> for &'a mut U where
    T: ConstSystemParam + TupleSystemParam + IncompatibleSystemParam<&'a mut U, PHANTOM>
{
}

impl<T, U, PHANTOM> IncompatibleSystemParam<Query<'_, T>, (PHANTOM,)> for QueryMut<'_, U>
where
    T: ConstSystemParam + TupleSystemParam + IncompatibleSystemParam<U, PHANTOM>,
    U: TupleSystemParam,
{
}

impl<'a, T, U, PHANTOM> IncompatibleSystemParam<QueryMut<'_, T>, (PHANTOM,)> for &'a U where
    T: IncompatibleSystemParam<&'a U, PHANTOM> + TupleSystemParam
{
}

impl<'a, T, U, PHANTOM> IncompatibleSystemParam<QueryMut<'_, T>, (PHANTOM,)> for &'a mut U where
    T: IncompatibleSystemParam<&'a mut U, PHANTOM> + TupleSystemParam
{
}

impl<T, U, PHANTOM> IncompatibleSystemParam<QueryMut<'_, T>, (PHANTOM,)> for Query<'_, U>
where
    T: IncompatibleSystemParam<U, PHANTOM> + TupleSystemParam,
    U: ConstSystemParam + TupleSystemParam,
{
}

impl<T, U, PHANTOM> IncompatibleSystemParam<QueryMut<'_, T>, (PHANTOM,)> for QueryMut<'_, U>
where
    T: IncompatibleSystemParam<U, PHANTOM> + TupleSystemParam,
    U: TupleSystemParam,
{
}

macro_rules! impl_incompatible_system_param {
    ($param:ident $(,$params:ident)*) => {
        impl<'a, $param, $($params,)* ITEM, PHANTOM>
            IncompatibleSystemParam<ITEM, (PHANTOM,)>
            for ($param, $($params),*)
        where
            $param: IncompatibleSystemParam<ITEM, PHANTOM>
        {
        }

        impl<'a, $param, $($params,)* ITEM, PHANTOM>
            IncompatibleSystemParam<ITEM, (PHANTOM, ())>
            for ($param, $($params),*)
        where
            ($($params,)*): IncompatibleSystemParam<ITEM, PHANTOM>
        {
        }

        impl<'a, $param, $($params,)* ITEM, PHANTOM>
            IncompatibleSystemParam<($param, $($params),*), (PHANTOM, (), ())>
            for ITEM
        where
            $param: IncompatibleSystemParam<ITEM, PHANTOM>
        {
        }

        impl<'a, $param, $($params,)* ITEM, PHANTOM>
            IncompatibleSystemParam<($param, $($params),*), (PHANTOM, (), (), ())>
            for ITEM
        where
            ($($params,)*): IncompatibleSystemParam<ITEM, PHANTOM>
        {
        }
    };
}

run_for_tuples!(impl_incompatible_system_param);

pub struct SystemStaticChecker<'a, 'b, SYS, T>(SYS, PhantomData<(&'a T, &'b T)>);

impl<'a, 'b, SYS, T> SystemStaticChecker<'a, 'b, SYS, T>
where
    SYS: System<'a, 'b, T>,
{
    pub fn new(system: SYS) -> Self {
        Self(system, PhantomData)
    }
}

pub trait SystemStandardCheck<SYS, PHANTOM> {
    fn check_statically(self) -> SYS;
}

impl<'a, 'b, SYS, PHANTOM> SystemStandardCheck<SYS, PHANTOM>
    for SystemStaticChecker<'a, 'b, SYS, PHANTOM>
where
    SYS: System<'a, 'b, PHANTOM>,
{
    fn check_statically(self) -> SYS {
        self.0
    }
}

pub trait OnlyOptionalParamsSystemCheck<SYS, PHANTOM> {
    fn check_statically(self) -> SYS;
}

macro_rules! impl_only_optional_params_system_check {
    ($param:ident $(,$params:ident)*) => {
        impl<'a, 'b, 'c, SYS, SYS2, $param, $($params,)* PHANTOM>
            OnlyOptionalParamsSystemCheck<SYS, (PHANTOM, SYS2)>
            for SystemStaticChecker<'a, 'b, SYS, ($param, $($params),*)>
        where
            $param: NotMandatoryComponentSystemParam + ComponentSystemParam,
            $($params: NotMandatoryComponentSystemParam,)*
        {
            fn check_statically(self) -> SYS {
                self.0
            }
        }

        impl<'a, 'b, 'c, SYS, SYS2, $param, $($params,)* PHANTOM>
            OnlyOptionalParamsSystemCheck<SYS, (PHANTOM, SYS2, ())>
            for SystemStaticChecker<'a, 'b, SYS, ($param, $($params),*)>
        where
            $param: NotMandatoryComponentSystemParam,
            SystemStaticChecker<'c, 'c, SYS2, ($($params,)*)>:
                OnlyOptionalParamsSystemCheck<SYS2, PHANTOM>,
            $($params: 'c,)*
        {
            fn check_statically(self) -> SYS {
                self.0
            }
        }
    };
}

run_for_tuples!(impl_only_optional_params_system_check);

pub trait SystemTypeIncompatibilityCheck<SYS, T, PHANTOM> {
    fn check_statically(self) -> SYS;
}

macro_rules! impl_incompatibility_system_check {
    ($param:ident $(,$params:ident)*) => {
        impl<'a, 'b, SYS, $param, $($params,)* PHANTOM>
            SystemTypeIncompatibilityCheck<SYS, ($param, $($params),*), (PHANTOM,)>
            for SystemStaticChecker<'a, 'b, SYS, ($param, $($params),*)>
        where
            SYS: System<'a, 'b, ($param, $($params),*)>,
            $param: IncompatibleSystemParam<($($params,)*), PHANTOM>,
        {
            fn check_statically(self) -> SYS {
                self.0
            }
        }

        impl<'a, 'b, SYS, $param, $($params,)* PHANTOM>
            SystemTypeIncompatibilityCheck<SYS, ($param, $($params),*), (PHANTOM, ())>
            for SystemStaticChecker<'a, 'b, SYS, ($param, $($params),*)>
        where
            SYS: System<'a, 'b, ($param, $($params),*)>,
            ($($params,)*): IncompatibleSystemParam<$param, PHANTOM>,
        {
            fn check_statically(self) -> SYS {
                self.0
            }
        }

        impl<'a, 'b, 'c, SYS, SYS2, $param, $($params,)* PHANTOM, PHANTOM2>
            SystemTypeIncompatibilityCheck<SYS, ($param, $($params),*), (PHANTOM, PHANTOM2, SYS2)>
            for SystemStaticChecker<'a, 'b, SYS, ($param, $($params),*)>
        where
            SYS: System<'a, 'b, ($param, $($params),*)>,
            SYS2: System<'c, 'c, ($($params,)*)>,
            SystemStaticChecker<'c, 'c, SYS2, ($($params,)*)>:
                SystemTypeIncompatibilityCheck<SYS2, ($($params,)*), PHANTOM2>,
            $($params: 'c,)*
        {
            fn check_statically(self) -> SYS {
                self.0
            }
        }
    };
}

run_for_tuples!(impl_incompatibility_system_check);
