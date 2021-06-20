use crate::external::systems::checks::internal::SealedChecker;
pub(crate) mod component_params;
pub(crate) mod param_compatibility;
pub(crate) mod query_component_params;

/// Characterise any system.
///
/// See documentation of [`system!`](crate::system!) macro for more information.
pub trait SystemWithParams<S, T>: Sized + SealedChecker {
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

mod internal {
    pub trait SealedChecker {}
}

#[cfg(test)]
mod system_with_params_tests {
    use super::*;

    struct ExampleChecker(u32);

    impl SealedChecker for ExampleChecker {}

    impl SystemWithParams<(), ()> for ExampleChecker {}

    #[test]
    fn check_component_params() {
        let checker = ExampleChecker(42);

        let run_checker = SystemWithParams::check_component_params(checker);

        assert_eq!(run_checker.0, 42);
    }

    #[test]
    fn check_query_component_params() {
        let checker = ExampleChecker(42);

        let run_checker = SystemWithParams::check_query_component_params(checker);

        assert_eq!(run_checker.0, 42);
    }

    #[test]
    fn check_param_compatibility() {
        let checker = ExampleChecker(42);

        let run_checker = SystemWithParams::check_param_compatibility(checker);

        assert_eq!(run_checker.0, 42);
    }
}
