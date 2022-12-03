use self::internal::SealedEntityType;
use crate::{Action, FinishedSystemRunner, SystemRunner};
use std::any::Any;

/// A trait for defining the main component of an entity type.
///
/// This trait shouldn't be directly implemented.<br>
/// Instead, you can use [`entity`](macro@crate::entity) and [`singleton`](macro@crate::singleton)
/// proc macros.
pub trait EntityMainComponent: Sized + Any + Sync + Send + Action {
    #[doc(hidden)]
    type Type: EntityType;

    #[doc(hidden)]
    fn on_update(runner: SystemRunner<'_>) -> FinishedSystemRunner;
}

#[doc(hidden)]
pub trait EntityType: Any + SealedEntityType {}

#[doc(hidden)]
pub struct NotSingleton;

impl SealedEntityType for NotSingleton {}

impl EntityType for NotSingleton {}

#[doc(hidden)]
pub struct Singleton;

impl SealedEntityType for Singleton {}

impl EntityType for Singleton {}

#[doc(hidden)]
pub trait Inheritable<E> {}

impl<E, T> Inheritable<E> for T
where
    T: EntityMainComponent,
    (T, T::Type): InheritableInner<E>,
{
}

#[doc(hidden)]
pub trait InheritableInner<E> {}

impl<T, E> InheritableInner<E> for (T, Singleton)
where
    T: EntityMainComponent<Type = Singleton>,
    E: EntityMainComponent<Type = Singleton>,
{
}

impl<T, E> InheritableInner<E> for (T, NotSingleton)
where
    T: EntityMainComponent<Type = NotSingleton>,
    E: EntityMainComponent,
{
}

mod internal {
    pub trait SealedEntityType {}
}
