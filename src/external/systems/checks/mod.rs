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
