use self::internal::SealedBool;
use crate::{Action, FinishedSystemRunner, SystemRunner, VariableSend, VariableSync};
use std::any::Any;

/// A trait for defining a component.
///
/// **Do not implement manually this trait.**<br>
/// Instead, you can use:
/// - [`Component`](macro@crate::Component) for a non-singleton component.
/// - [`SingletonComponent`](macro@crate::SingletonComponent) for a singleton component.
pub trait Component: Sized + Any + VariableSync + VariableSend {
    /// Whether the component is a singleton, i.e. there is a maximum of one instance at a time.
    type IsSingleton: Bool;
}

/// A trait for defining systems of a component.
///
/// **Do not implement manually this trait.**<br>
/// Instead, you can use one of these macros:
/// - [`systems`](macro@crate::systems) proc macro to define systems of a component.
/// - [`NoSystem`](macro@crate::NoSystem) derive macro to define a component without systems.
pub trait ComponentSystems: Component {
    /// The type of the action associated to the component type.
    ///
    /// The action is considered as done when all systems of the component have been run.
    type Action: Action;

    #[doc(hidden)]
    fn on_update(runner: SystemRunner<'_>) -> FinishedSystemRunner;
}

#[doc(hidden)]
pub trait Bool: Any + SealedBool {}

#[doc(hidden)]
pub struct False;

impl SealedBool for False {}

impl Bool for False {}

#[doc(hidden)]
pub struct True;

impl SealedBool for True {}

impl Bool for True {}

mod internal {
    pub trait SealedBool {}
}
