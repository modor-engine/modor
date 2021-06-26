use crate::external::systems::checks::internal::SealedChecker;
use crate::external::systems::param_traits::internal::{MultipleSystemParams, QuerySystemParam};
use crate::{
    System, SystemComponentParamChecker, SystemWithMissingComponentParam, SystemWithParams,
};
use std::marker::PhantomData;

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

impl<'a, 'b, S, T> SealedChecker for SystemQueryComponentParamChecker<'a, 'b, S, T> {}

impl<'a, 'b, S, T> SystemWithParams<S, T> for SystemQueryComponentParamChecker<'a, 'b, S, T> where
    S: System<'a, 'b, T>
{
}

/// Characterise a system with a query that has a missing component parameter.
///
/// See documentation of [`system!`](crate::system!) macro for more information.
pub trait SystemWithQueryWithMissingComponentParam<S, Z>: Sized + SealedChecker {
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

#[cfg(test)]
mod system_query_component_param_checker_tests {
    use super::*;

    assert_impl_all!(SystemQueryComponentParamChecker<'_, '_, fn(&u32), (&u32,)>: Sync, Send);

    fn system_example() {}

    #[test]
    fn into_inner() {
        let checker = SystemQueryComponentParamChecker::new(system_example);

        let system = checker.into_inner();

        assert_eq!(system as usize, system_example as usize);
    }
}

#[cfg(test)]
mod system_with_query_with_missing_component_param_tests {
    use super::*;

    struct ExampleChecker(u32);

    impl SealedChecker for ExampleChecker {}

    impl SystemWithQueryWithMissingComponentParam<(), ()> for ExampleChecker {}

    #[test]
    fn check_query_component_params() {
        let checker = ExampleChecker(42);

        let run_checker =
            SystemWithQueryWithMissingComponentParam::check_query_component_params(checker);

        assert_eq!(run_checker.0, 42);
    }
}
