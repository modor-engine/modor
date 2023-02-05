use self::internal::SealedBool;
use crate::{Action, FinishedSystemRunner, SystemRunner};
use std::any::Any;

/// A trait for defining a component.
///
/// **Do not implement manually this trait.**<br>
/// Instead, you can use:
/// - [`entity`](macro@crate::entity) proc macro to define an entity main component
/// - [`singleton`](macro@crate::singleton) proc macro to define a singleton entity main component
/// - [`Component`](macro@crate::Component) derive macro to define a simple component
pub trait Component: Sized + Any + Sync + Send {
    /// Whether the component type is the main component of an entity.
    type IsEntityMainComponent: Bool;
    /// The type of the action associated to the component type.
    ///
    /// The action is considered as done when all systems of the component have been run.
    type Action: Action;

    #[doc(hidden)]
    fn on_update(runner: SystemRunner<'_>) -> FinishedSystemRunner;
}

/// A trait for defining the main component of an entity type.
///
/// **Do not implement manually this trait.**<br>
/// Instead, you can use [`entity`](macro@crate::entity) and [`singleton`](macro@crate::singleton)
/// proc macros.
pub trait EntityMainComponent: Component<IsEntityMainComponent = True> {
    /// Whether the entity is a singleton.
    type IsSingleton: Bool;
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

#[doc(hidden)]
pub trait Inheritable<E> {}

impl<E, T> Inheritable<E> for T
where
    T: EntityMainComponent,
    (T, T::IsSingleton): InheritableWithIsSingleton<E>,
{
}

#[doc(hidden)]
pub trait InheritableWithIsSingleton<E> {}

impl<T, E> InheritableWithIsSingleton<E> for (T, True)
where
    T: EntityMainComponent<IsSingleton = True>,
    E: EntityMainComponent<IsSingleton = True>,
{
}

impl<T, E> InheritableWithIsSingleton<E> for (T, False)
where
    T: EntityMainComponent<IsSingleton = False>,
    E: EntityMainComponent,
{
}

mod internal {
    pub trait SealedBool {}
}
