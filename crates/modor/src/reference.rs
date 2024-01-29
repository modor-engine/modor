use crate::{Context, Object};
use derivative::Derivative;
use std::any::TypeId;
use std::fmt::{Display, Formatter};
use std::marker::PhantomData;

/// The unique ID of an object of type `T`.
///
/// # Examples
///
/// See [`modor`](crate).
#[derive(Derivative)]
#[derivative(
    Debug(bound = ""),
    Clone(bound = ""),
    Copy(bound = ""),
    PartialEq(bound = ""),
    Eq(bound = ""),
    Hash(bound = "")
)]
pub struct Id<T> {
    pub(crate) index: usize,
    pub(crate) generation_id: u64,
    phantom: PhantomData<fn(T)>,
}

impl<T> Id<T>
where
    T: Object,
{
    pub(crate) fn new(index: usize, generation_id: u64) -> Self {
        Self {
            index,
            generation_id,
            phantom: PhantomData,
        }
    }

    /// Returns an immutable reference to the object.
    ///
    /// # Errors
    ///
    /// The following errors can be returned:
    ///
    /// - [`Error::ObjectNotFound`](crate::Error::ObjectNotFound)
    /// - [`Error::ObjectTypeAlreadyLocked`](crate::Error::ObjectTypeAlreadyLocked)
    #[inline]
    pub fn get<'a, O>(self, context: &'a Context<'_, O>) -> crate::Result<&'a T>
    where
        O: Object,
    {
        context.objects()?.get(self)
    }

    /// Returns a mutable reference to the object.
    ///
    /// # Errors
    ///
    /// The following errors can be returned:
    ///
    /// - [`Error::ObjectNotFound`](crate::Error::ObjectNotFound)
    /// - [`Error::ObjectTypeAlreadyLocked`](crate::Error::ObjectTypeAlreadyLocked)
    #[inline]
    pub fn get_mut<'a, O>(self, context: &'a mut Context<'_, O>) -> crate::Result<&'a mut T>
    where
        O: Object,
    {
        context.objects_mut()?.get_mut(self)
    }
}

impl<T> Display for Id<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}|{}", self.index, self.generation_id)
    }
}

/// The unique ID of an object of unknown type.
///
/// # Examples
///
/// See [`modor`](crate).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DynId {
    pub(crate) index: usize,
    pub(crate) generation_id: u64,
    pub(crate) object_type_id: TypeId,
}

impl DynId {
    /// Returns the typed version of the ID.
    ///
    /// `None` is returned if the object is not of type `T`.
    pub fn typed<T>(self) -> Option<Id<T>>
    where
        T: Object,
    {
        (self.object_type_id == TypeId::of::<T>()).then_some(Id {
            index: self.index,
            generation_id: self.generation_id,
            phantom: PhantomData,
        })
    }
}

impl<T> From<Id<T>> for DynId
where
    T: Object,
{
    #[inline]
    fn from(value: Id<T>) -> Self {
        Self {
            index: value.index,
            generation_id: value.generation_id,
            object_type_id: TypeId::of::<T>(),
        }
    }
}

impl Display for DynId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}|{}", self.index, self.generation_id)
    }
}
