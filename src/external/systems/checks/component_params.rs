use crate::external::systems::checks::internal::SealedChecker;
use crate::external::systems::params::internal::{
    NotEnoughEntityPartSystemParam, NotMandatoryComponentSystemParam,
};
use crate::{System, SystemWithParams};
use std::marker::PhantomData;

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

impl<'a, 'b, S, T> SealedChecker for SystemComponentParamChecker<'a, 'b, S, T> {}

impl<'a, 'b, S, T> SystemWithParams<S, T> for SystemComponentParamChecker<'a, 'b, S, T> where
    S: System<'a, 'b, T>
{
}

/// Characterise a system with a missing component parameter.
///
/// See documentation of [`system!`](crate::system!) macro for more information.
pub trait SystemWithMissingComponentParam<S, Z>: Sized + SealedChecker {
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

#[cfg(test)]
mod system_component_param_checker_tests {
    use super::*;

    assert_impl_all!(SystemComponentParamChecker<'_, '_, fn(&u32), (&u32,)>: Sync, Send);

    fn system_example() {}

    #[test]
    fn into_inner() {
        let checker = SystemComponentParamChecker::new(system_example);

        let system = checker.into_inner();

        assert_eq!(system as usize, system_example as usize);
    }
}

#[cfg(test)]
mod system_with_missing_component_param_tests {
    use super::*;

    struct ExampleChecker(u32);

    impl SealedChecker for ExampleChecker {}

    impl SystemWithMissingComponentParam<(), ()> for ExampleChecker {}

    #[test]
    fn check_component_params() {
        let checker = ExampleChecker(42);

        let run_checker = SystemWithMissingComponentParam::check_component_params(checker);

        assert_eq!(run_checker.0, 42);
    }
}
