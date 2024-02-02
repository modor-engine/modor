use crate::{Context, Error, Object};
use derivative::Derivative;
use std::any;
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
    /// - [`Error::ObjectNotFound`]
    /// - [`Error::ObjectTypeAlreadyLocked`]
    #[inline]
    pub fn get<'a, U>(self, context: &'a Context<'_, U>) -> crate::Result<&'a T> {
        context.objects()?.get(self)
    }

    /// Returns a mutable reference to the object.
    ///
    /// # Errors
    ///
    /// The following errors can be returned:
    ///
    /// - [`Error::ObjectNotFound`]
    /// - [`Error::ObjectTypeAlreadyLocked`]
    #[inline]
    pub fn get_mut<'a, U>(self, context: &'a mut Context<'_, U>) -> crate::Result<&'a mut T> {
        context.objects_mut()?.get_mut(self)
    }
}

impl<T> Display for Id<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}|{}", self.index, self.generation_id)
    }
}

impl<T> From<Id<T>> for usize {
    #[inline]
    fn from(value: Id<T>) -> Self {
        value.index
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
    /// # Errors
    ///
    /// The following errors can be returned:
    ///
    /// - [`Error::InvalidIdType`]
    pub fn typed<T>(self) -> crate::Result<Id<T>>
    where
        T: Object,
    {
        if self.object_type_id == TypeId::of::<T>() {
            Ok(Id {
                index: self.index,
                generation_id: self.generation_id,
                phantom: PhantomData,
            })
        } else {
            Err(Error::InvalidIdType(any::type_name::<T>()))
        }
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

impl From<DynId> for usize {
    #[inline]
    fn from(value: DynId) -> Self {
        value.index
    }
}
