use crate::{Role, UpdateContext};
use std::any::Any;

/// A trait for defining an object.
///
/// Objects are the main resources stored in the [`App`](crate::App). They can store data of the
/// application and execute logic on it.
///
/// Note that in case the object does not need to be updated, you can quickly implement this trait
/// with the [`Object`](macro@crate::Object) derive macro.
///
/// # Examples
///
/// See [`modor`](crate).
pub trait Object: Sized + Any {
    /// Whether [`Object::update`] is run during [`App::update`](crate::App::update).
    ///
    /// Note that in case the value is `false`, you can quickly implement the
    /// [`Object`] trait with the [`Object`](macro@crate::Object) derive macro.
    const IS_UPDATE_ENABLED: bool = true;
    /// Whether the object is a singleton.
    ///
    /// Instead of configuring manually this constant, it is recommended to implement
    /// [`SingletonObject`].
    const IS_SINGLETON: bool = false;

    /// The role of the object.
    type Role: Role;

    /// Updates the object.
    ///
    /// This method is run once per instance during [`App::update`](crate::App::update).
    ///
    /// As the update locks objects of type `Self`, `ctx` cannot access to these objects.
    ///
    /// Note that in case no logic is performed in this method, you can quickly implement the
    /// [`Object`] trait with the [`Object`](macro@crate::Object) derive macro.
    ///
    /// # Errors
    ///
    /// Any error not explicitly handled in this method can be returned to be logged.
    /// Note that this doesn't stop the execution of the application.
    fn update(&mut self, ctx: &mut UpdateContext<'_>) -> crate::Result<()>;
}

/// A trait for defining a singleton object.
///
/// This trait automatically implements [`Object`], and indicates to the engine that a maximum
/// of one instance of the object type must exist. Instance of a singleton object type
/// can be more easily accessed than for standard object types.
///
/// Note that in case the object does not need to be updated, you can quickly implement this trait
/// with the [`SingletonObject`](macro@crate::SingletonObject) derive macro.
///
/// # Examples
///
/// See [`modor`](crate).
pub trait SingletonObject: Object {
    /// Whether [`SingletonObject::update`] is run during [`App::update`](crate::App::update).
    ///
    /// Note that in case the value is `false`, you can quickly implement the
    /// [`SingletonObject`] trait with the [`SingletonObject`](macro@crate::SingletonObject)
    /// derive macro.
    const IS_UPDATE_ENABLED: bool = true;

    /// The role of the object.
    type Role: Role;

    /// Updates the object.
    ///
    /// This method is run once per instance during [`App::update`](crate::App::update).
    ///
    /// As the update locks objects of type `Self`, `ctx` cannot access to these objects.
    ///
    /// Note that in case no logic is performed in this method, you can quickly implement the
    /// [`Object`] trait with the [`Object`](macro@crate::Object) derive macro.
    ///
    /// # Errors
    ///
    /// Any error not explicitly handled in this method can be returned to be logged.
    /// Note that this doesn't stop the execution of the application.
    fn update(&mut self, ctx: &mut UpdateContext<'_>) -> crate::Result<()>;
}

impl<T> Object for T
where
    T: SingletonObject,
{
    const IS_UPDATE_ENABLED: bool = <Self as SingletonObject>::IS_UPDATE_ENABLED;
    const IS_SINGLETON: bool = true;

    type Role = <Self as SingletonObject>::Role;

    fn update(&mut self, ctx: &mut UpdateContext<'_>) -> crate::Result<()> {
        SingletonObject::update(self, ctx)
    }
}
