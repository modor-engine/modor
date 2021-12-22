use crate::system_checks::internal::{SealedChecker, SystemParamWithMutabilityIssue};
use crate::{System, SystemParam};
use std::marker::PhantomData;

#[doc(hidden)]
pub struct SystemParamMutabilityChecker<S, P>(S, PhantomData<P>);

impl<S, P> SystemParamMutabilityChecker<S, P>
where
    S: System<P>,
    P: SystemParam,
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

impl<S, P> SealedChecker for SystemParamMutabilityChecker<S, P> {}

/// A trait implemented for any system.
///
/// This trait is used by the [`system!`](crate::system!) macro to detect invalid systems.
pub trait SystemWithParams<S, P>: Sized + SealedChecker {
    #[doc(hidden)]
    fn check_param_mutability(self) -> Self {
        self
    }
}

impl<S, P> SystemWithParams<S, P> for SystemParamMutabilityChecker<S, P>
where
    S: System<P>,
    P: SystemParam,
{
}

/// A trait implemented for any system with mutability issue.
///
/// There is a mutability issue when two parameters of the system lock the same resource of the
/// engine, and at least one of them locks the resource mutably
/// (e.g. there are two parameters of type `&C` and `&mut C`).
///
/// This trait is used by the [`system!`](crate::system!) macro to detect invalid systems.
pub trait SystemWithParamMutabilityIssue<S, Z>: Sized + SealedChecker {
    #[doc(hidden)]
    fn check_param_mutability(self) -> Self {
        self
    }
}

impl<S, P, Z> SystemWithParamMutabilityIssue<S, Z> for SystemParamMutabilityChecker<S, P>
where
    S: System<P>,
    P: SystemParam + SystemParamWithMutabilityIssue<Z>,
{
}

mod internal {
    use crate::system_params::internal::{Const, LockableSystemParam, Mut};
    use crate::SystemParam;

    pub trait SealedChecker {}

    pub trait SystemParamWithMutabilityIssue<Z> {}

    macro_rules! impl_system_param_with_mutability_issue {
        (($param:ident, $index:tt) $(,($params:ident, $indexes:tt))*) => {
            impl<P, $param, $($params,)* Z>
                SystemParamWithMutabilityIssue<((), Z, ($param, $($params),*))>
                for P
            where
                P: SystemParam<InnerTuple = ($param, $($params),*)>,
                $param: IncompatibleSystemParam<($($params,)*), Z>,
            {
            }

            impl<P, $param, $($params,)* Z>
                SystemParamWithMutabilityIssue<(((),), Z, ($param, $($params),*))>
                for P
            where
                P: SystemParam<InnerTuple = ($param, $($params),*)>,
                $param: SystemParamWithMutabilityIssue<Z>,
            {
            }

            impl<P, $param, $($params,)* Z>
                SystemParamWithMutabilityIssue<((((),),), Z, ($param, $($params),*))>
                for P
            where
                P: SystemParam<InnerTuple = ($param, $($params),*)>,
                ($($params,)*): SystemParamWithMutabilityIssue<Z>,
            {
            }
        };
    }

    run_for_tuples_with_idxs!(impl_system_param_with_mutability_issue);

    pub trait IncompatibleSystemParam<P, Z>: Sized {}

    impl<P1, P2, T> IncompatibleSystemParam<P2, ((), T)> for P1
    where
        P1: LockableSystemParam<LockedType = T, Mutability = Const>,
        P2: LockableSystemParam<LockedType = T, Mutability = Mut>,
    {
    }

    impl<P1, P2, T> IncompatibleSystemParam<P2, ((), T, ())> for P1
    where
        P1: LockableSystemParam<LockedType = T, Mutability = Mut>,
        P2: LockableSystemParam<LockedType = T, Mutability = Const>,
    {
    }

    impl<P1, P2, T> IncompatibleSystemParam<P2, ((), T, ((),))> for P1
    where
        P1: LockableSystemParam<LockedType = T, Mutability = Mut>,
        P2: LockableSystemParam<LockedType = T, Mutability = Mut>,
    {
    }

    macro_rules! impl_incompatible_system_param {
        (($param:ident, $index:tt) $(,($params:ident, $indexes:tt))*) => {
            impl<$param, $($params,)* P1, P2, Z>
                IncompatibleSystemParam<P2, (((),), Z, ($param, $($params),*))>
                for P1
            where
                P1: SystemParam<InnerTuple = ($param, $($params),*)>,
                P2: IncompatibleSystemParam<$param, Z>,
            {
            }

            impl<$param, $($params,)* P1, P2, Z>
                IncompatibleSystemParam<P2, (((),), Z, ($param, $($params),*), ())>
                for P1
            where
                P1: SystemParam<InnerTuple = ($param, $($params),*)>,
                P2: IncompatibleSystemParam<($($params,)*), Z>,
            {
            }

            impl<$param, $($params,)* P1, P2, Z>
                IncompatibleSystemParam<P1, (((),), Z, ($param, $($params),*), ((),))>
                for P2
            where
                P1: SystemParam<InnerTuple = ($param, $($params),*)>,
                P2: IncompatibleSystemParam<$param, Z>,
            {
            }

            impl<$param, $($params,)* P1, P2, Z>
                IncompatibleSystemParam<P1, (((),), Z, ($param, $($params),*), (((),),))>
                for P2
            where
                P1: SystemParam<InnerTuple = ($param, $($params),*)>,
                P2: IncompatibleSystemParam<($($params,)*), Z>,
            {
            }
        };
    }

    run_for_tuples_with_idxs!(impl_incompatible_system_param);
}

#[cfg(test)]
mod system_param_mutability_checker_tests {
    use super::*;

    #[test]
    fn convert_into_inner() {
        let system = || ();
        let checker = SystemParamMutabilityChecker::new(system);

        let inner = checker.into_inner();

        assert_eq!(system as fn(), inner as fn());
    }

    #[test]
    fn check_param_mutability_of_system_with_params() {
        let system = || ();
        let checker = SystemParamMutabilityChecker::new(system);

        let checker = SystemWithParams::check_param_mutability(checker);

        assert_eq!(system as fn(), checker.into_inner() as fn());
    }

    #[test]
    fn check_param_mutability_of_system_with_mutability_issue() {
        let system = |_: &u32, _: &mut u32| ();
        let checker = SystemParamMutabilityChecker::new(system);

        let checker = SystemWithParamMutabilityIssue::check_param_mutability(checker);

        let inner_system = checker.into_inner() as fn(&'static u32, &'static mut u32);
        assert_eq!(system as fn(&'static u32, &'static mut u32), inner_system);
    }
}
