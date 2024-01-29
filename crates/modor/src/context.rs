use crate::storages::actions::{Action, ActionStorage};
use crate::storages::object_ids::ObjectIdStorage;
use crate::storages::objects::ObjectStorage;
use crate::{DynId, Error, Id, InternalError, IntoResult, Object, Objects, SingletonObject};
use std::any;
use std::sync::Arc;

/// The context of an object used to run actions and access any other existing object.
///
/// # Examples
///
/// See [`modor`](crate).
pub struct Context<'a, S> {
    pub(crate) objects: &'a mut ObjectStorage,
    pub(crate) object_ids: &'a ObjectIdStorage,
    pub(crate) actions: &'a ActionStorage,
    pub(crate) self_id: Option<Id<S>>,
}

impl<S> Context<'_, S>
where
    S: Object,
{
    /// Returns the ID of the current object.
    #[inline]
    pub fn self_id(&self) -> Id<S> {
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
        f: impl FnOnce(&mut Context<'_, S>, &mut Objects<T>) -> crate::Result<()>,
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
    pub fn create<T, R>(&self, builder: impl FnOnce(&mut Context<'_, T>) -> R + 'static) -> Id<T>
    where
        T: Object,
        R: IntoResult<T>,
    {
        let id = self.object_ids.reserve();
        let parent = self.self_id().into();
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

    /// Delete an object.
    ///
    /// If the object doesn't exist, then nothing happens.
    ///
    /// The object is actually deleted after all objects have been updated.
    ///
    /// All linked objects are also deleted (see [`Context::create`]).
    pub fn delete(&self, id: impl Into<DynId>) {
        self.actions.push(Action::ObjectDeletion(id.into()));
    }

    /// Delete the current object.
    ///
    /// The object is actually deleted after all objects have been updated.
    pub fn delete_self(&self) {
        let id = self.self_id().into();
        self.actions.push(Action::ObjectDeletion(id));
    }
}
