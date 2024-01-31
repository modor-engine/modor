use crate::storages::actions::{Action, ActionStorage};
use crate::storages::object_ids::ObjectIdStorage;
use crate::storages::objects::ObjectStorage;
use crate::{DynId, Error, Id, InternalError, Object, ObjectResult, Objects, SingletonObject};
use std::any;
use std::marker::PhantomData;
use std::sync::Arc;

/// A trait implemented for the different types of [`Context`].
pub trait ContextType {}

/// The [`Context`] type for object creation.
pub struct BuildContextType;

impl ContextType for BuildContextType {}

/// The [`Context`] type for object update.
pub struct UpdateContextType;

impl ContextType for UpdateContextType {}

/// The [`Context`] for object creation.
pub type BuildContext<'a> = Context<'a, BuildContextType>;

/// The [`Context`] for object update.
pub type UpdateContext<'a> = Context<'a, UpdateContextType>;

/// The context of an object used to run actions and access any other existing object.
///
/// # Examples
///
/// See [`modor`](crate).
#[derive(Debug)]
pub struct Context<'a, T> {
    pub(crate) objects: &'a mut ObjectStorage,
    pub(crate) object_ids: &'a mut ObjectIdStorage,
    pub(crate) actions: &'a mut ActionStorage,
    pub(crate) self_id: Option<DynId>,
    pub(crate) phantom: PhantomData<fn(T)>,
}

impl<U> Context<'_, U> {
    /// Returns the ID of the current object.
    #[inline]
    pub fn self_id(&self) -> DynId {
        self.self_id.expect("self ID not configured")
    }

    /// Returns an immutable reference to the singleton object of type `T`.
    ///
    /// # Errors
    ///
    /// The following errors can be returned:
    ///
    /// - [`Error::ObjectTypeAlreadyLocked`]
    /// - [`Error::SingletonObjectNotFound`]
    #[inline]
    pub fn singleton<T>(&self) -> crate::Result<&T>
    where
        T: SingletonObject,
    {
        self.objects.get::<T>()?.singleton()
    }

    /// Returns a mutable reference to the singleton object of type `T`.
    ///
    /// # Errors
    ///
    /// The following errors can be returned:
    ///
    /// - [`Error::ObjectTypeAlreadyLocked`]
    /// - [`Error::SingletonObjectNotFound`]
    #[inline]
    pub fn singleton_mut<T>(&mut self) -> crate::Result<&mut T>
    where
        T: SingletonObject,
    {
        self.objects.get_mut::<T>()?.singleton_mut()
    }

    /// Returns an immutable reference to the storage of objects of type `T`.
    ///
    /// # Errors
    ///
    /// The following errors can be returned:
    ///
    /// - [`Error::ObjectTypeAlreadyLocked`]
    #[inline]
    pub fn objects<T>(&self) -> crate::Result<&Objects<T>>
    where
        T: Object,
    {
        self.objects.get()
    }

    /// Returns a mutable reference to the storage of objects of type `T`.
    ///
    /// # Errors
    ///
    /// The following errors can be returned:
    ///
    /// - [`Error::ObjectTypeAlreadyLocked`]
    #[inline]
    pub fn objects_mut<T>(&mut self) -> crate::Result<&mut Objects<T>>
    where
        T: Object,
    {
        self.objects.get_mut()
    }

    /// Locks the storage containing all objects of type `T`.
    ///
    /// This method is particularly useful to get mutable access to multiple object types at the
    /// same time and avoid borrow checker issues.
    ///
    /// # Errors
    ///
    /// The following errors can be returned:
    ///
    /// - [`Error::ObjectTypeAlreadyLocked`]
    pub fn lock_objects<T>(
        &mut self,
        f: impl FnOnce(&mut Context<'_, U>, &mut Objects<T>) -> crate::Result<()>,
    ) -> crate::Result<()>
    where
        T: Object,
    {
        self.objects.lock::<T>(|all_objects, objects| {
            let mut context = Context {
                objects: all_objects,
                object_ids: self.object_ids,
                actions: self.actions,
                self_id: self.self_id,
                phantom: PhantomData,
            };
            f(&mut context, objects)
        })
    }

    /// Creates a new object.
    ///
    /// If the object is a singleton and already exists, or if an error is raised during creation,
    /// then nothing happens.
    ///
    /// The object is actually created after all objects have been updated.
    ///
    /// This object is linked to the current object. It means that when the current object will be
    /// deleted, the created object will also be deleted.
    pub fn create<T, R>(
        &mut self,
        builder: impl FnOnce(&mut BuildContext<'_>) -> R + 'static,
    ) -> Id<T>
    where
        T: Object,
        R: ObjectResult<Object = T>,
    {
        let id = self.object_ids.reserve();
        let parent = self.self_id();
        self.actions.push(Action::Other(Box::new(move |app| {
            app.create_object_or_rollback(id, Some(parent), builder)
                .map_err(|_| {
                    Error::Other(Arc::new(InternalError::ObjectCreationFailed(
                        any::type_name::<T>(),
                    )))
                })
        })));
        id.id()
    }
}

// Object deletion is only performed during update because we cannot rollback it during object
// creation in case of error.
// Also, behavior of `delete_self()` during object creation is uncertain, so it's safer to disallow
// object deletion during object creation.
impl Context<'_, UpdateContextType> {
    /// Delete an object.
    ///
    /// If the object doesn't exist, then nothing happens.
    ///
    /// The object is actually deleted after all objects have been updated.
    ///
    /// All linked objects are also deleted (see [`Context::create`]).
    pub fn delete(&mut self, id: impl Into<DynId>) {
        self.actions.push(Action::ObjectDeletion(id.into()));
    }

    /// Delete the current object.
    ///
    /// The object is actually deleted after all objects have been updated.
    pub fn delete_self(&mut self) {
        self.actions.push(Action::ObjectDeletion(self.self_id()));
    }
}
