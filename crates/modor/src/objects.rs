use crate::{Error, Id, Object, SingletonObject, UpdateContext};
use fxhash::FxHashSet;
use log::error;
use rayon::iter::{
    IndexedParallelIterator, IntoParallelRefIterator, IntoParallelRefMutIterator, ParallelIterator,
};
use std::iter::{Copied, Enumerate, FilterMap, Flatten, Zip};
use std::slice::{Iter, IterMut};
use std::{any, mem};

/// An iterator over immutable reference to objects of type `T` and their associated [`Id<T>`].
pub type ObjectsIterEnumerated<'a, T> = FilterMap<
    Zip<Enumerate<Iter<'a, Option<T>>>, Copied<Iter<'a, u64>>>,
    fn(((usize, &Option<T>), u64)) -> Option<(Id<T>, &T)>,
>;

///  An iterator over mutable reference to objects of type `T` and their associated [`Id<T>`].
pub type ObjectsIterMutEnumerated<'a, T> = FilterMap<
    Zip<Enumerate<IterMut<'a, Option<T>>>, Copied<Iter<'a, u64>>>,
    fn(((usize, &mut Option<T>), u64)) -> Option<(Id<T>, &mut T)>,
>;

/// A parallel iterator over immutable reference to objects of type `T` and their associated
/// [`Id<T>`].
pub type ObjectsParIterEnumerated<'a, T> = rayon::iter::FilterMap<
    rayon::iter::Zip<
        rayon::iter::Enumerate<rayon::slice::Iter<'a, Option<T>>>,
        rayon::iter::Copied<rayon::slice::Iter<'a, u64>>,
    >,
    fn(((usize, &Option<T>), u64)) -> Option<(Id<T>, &T)>,
>;

/// A parallel iterator over mutable reference to objects of type `T` and their associated
/// [`Id<T>`].
pub type ObjectsParIterMutEnumerated<'a, T> = rayon::iter::FilterMap<
    rayon::iter::Zip<
        rayon::iter::Enumerate<rayon::slice::IterMut<'a, Option<T>>>,
        rayon::iter::Copied<rayon::slice::Iter<'a, u64>>,
    >,
    fn(((usize, &mut Option<T>), u64)) -> Option<(Id<T>, &mut T)>,
>;

/// A storage containing all objects of type `T`.
///
/// # Parallelism
///
/// Some methods allow to iterate on objects in parallel.
///
/// As parallelism is handled by [`rayon`], you can for example configure the number of threads
/// by settings the `RAYON_NUM_THREADS` environment variable before creating the
/// [`App`](crate::App). For example:
///
/// ```rust
/// # use std::env;
/// # use modor::*;
///
/// fn main() {
///     env::set_var("RAYON_NUM_THREADS", "8");
///     App::new()
///     // ...
/// # ;
/// }
/// ```
///
/// # Examples
///
/// See [`modor`](crate).
#[derive(Debug)]
pub struct Objects<T> {
    is_locked: bool,
    objects: Vec<Option<T>>,
    generation_ids: Vec<u64>,
    logged_errors: Option<FxHashSet<String>>,
}

impl<T> Default for Objects<T> {
    fn default() -> Self {
        Self {
            is_locked: false,
            objects: vec![],
            generation_ids: vec![],
            logged_errors: Some(FxHashSet::default()),
        }
    }
}

impl<T> Objects<T>
where
    T: Object,
{
    pub(crate) const DEFAULT: &'static Self = &Self {
        is_locked: false,
        objects: vec![],
        generation_ids: vec![],
        logged_errors: None,
    };

    // TODO: add exists(Id<T>) method (is getting list of deleted items still needed ?)

    /// Returns an immutable reference to the object with a given `id`.
    ///
    /// # Errors
    ///
    /// The following errors can be returned:
    ///
    /// - [`Error::ObjectNotFound`]
    pub fn get(&self, id: Id<T>) -> crate::Result<&T> {
        self.check_id(id)?;
        self.objects
            .get(id.index)
            .and_then(|object| object.as_ref())
            .ok_or_else(|| Error::ObjectNotFound(any::type_name::<T>()))
    }

    /// Returns a mutable reference to the object with a given `id`.
    ///
    /// # Errors
    ///
    /// The following errors can be returned:
    ///
    /// - [`Error::ObjectNotFound`]
    pub fn get_mut(&mut self, id: Id<T>) -> crate::Result<&mut T> {
        self.check_id(id)?;
        self.objects
            .get_mut(id.index)
            .and_then(|object| object.as_mut())
            .ok_or_else(|| Error::ObjectNotFound(any::type_name::<T>()))
    }

    /// Returns an immutable reference to the singleton object.
    ///
    /// # Errors
    ///
    /// The following errors can be returned:
    ///
    /// - [`Error::SingletonObjectNotFound`]
    pub fn singleton(&self) -> crate::Result<&T>
    where
        T: SingletonObject,
    {
        self.objects
            .first()
            .and_then(|object| object.as_ref())
            .ok_or_else(|| Error::SingletonObjectNotFound(any::type_name::<T>()))
    }

    /// Returns a mutable reference to the singleton object.
    ///
    /// # Errors
    ///
    /// The following errors can be returned:
    ///
    /// - [`Error::SingletonObjectNotFound`]
    pub fn singleton_mut(&mut self) -> crate::Result<&mut T>
    where
        T: SingletonObject,
    {
        self.objects
            .first_mut()
            .and_then(|object| object.as_mut())
            .ok_or_else(|| Error::SingletonObjectNotFound(any::type_name::<T>()))
    }

    /// Returns an iterator over immutable references to all objects.
    pub fn iter(&self) -> Flatten<Iter<'_, Option<T>>> {
        self.objects.iter().flatten()
    }

    /// Returns an iterator over mutable references to all objects.
    pub fn iter_mut(&mut self) -> Flatten<IterMut<'_, Option<T>>> {
        self.objects.iter_mut().flatten()
    }

    /// Returns a parallel iterator over immutable references to all objects.
    pub fn par_iter(&self) -> rayon::iter::Flatten<rayon::slice::Iter<'_, Option<T>>>
    where
        T: Sync,
    {
        ParallelIterator::flatten(self.objects.par_iter())
    }

    /// Returns a parallel iterator over mutable references to all objects.
    pub fn par_iter_mut(&mut self) -> rayon::iter::Flatten<rayon::slice::IterMut<'_, Option<T>>>
    where
        T: Send,
    {
        ParallelIterator::flatten(self.objects.par_iter_mut())
    }

    /// Returns an iterator over immutable references to all objects and their [`Id<T>`].
    pub fn iter_enumerated(&self) -> ObjectsIterEnumerated<'_, T> {
        self.objects
            .iter()
            .enumerate()
            .zip(self.generation_ids.iter().copied())
            .filter_map(|((index, object), generation_id)| {
                object
                    .as_ref()
                    .map(|object| (Id::<T>::new(index, generation_id), object))
            })
    }

    /// Returns an iterator over mutable references to all objects and their [`Id<T>`].
    pub fn iter_mut_enumerated(&mut self) -> ObjectsIterMutEnumerated<'_, T> {
        self.objects
            .iter_mut()
            .enumerate()
            .zip(self.generation_ids.iter().copied())
            .filter_map(|((index, object), generation_id)| {
                object
                    .as_mut()
                    .map(|object| (Id::<T>::new(index, generation_id), object))
            })
    }

    /// Returns a parallel iterator over immutable references to all objects and their [`Id<T>`].
    pub fn par_iter_enumerated(&self) -> ObjectsParIterEnumerated<'_, T>
    where
        T: Sync,
    {
        IndexedParallelIterator::enumerate(self.objects.par_iter())
            .zip(self.generation_ids.par_iter().copied())
            .filter_map(|((index, object), generation_id)| {
                object
                    .as_ref()
                    .map(|object| (Id::<T>::new(index, generation_id), object))
            })
    }

    /// Returns a parallel iterator over mutable references to all objects and their [`Id<T>`].
    pub fn par_iter_mut_enumerated(&mut self) -> ObjectsParIterMutEnumerated<'_, T>
    where
        T: Send,
    {
        IndexedParallelIterator::enumerate(self.objects.par_iter_mut())
            .zip(self.generation_ids.par_iter().copied())
            .filter_map(|((index, object), generation_id)| {
                object
                    .as_mut()
                    .map(|object| (Id::<T>::new(index, generation_id), object))
            })
    }

    pub(crate) fn add(&mut self, object: T, id: Id<T>) {
        if let Some(current_object) = self.objects.get_mut(id.index) {
            *current_object = Some(object);
            self.generation_ids[id.index] = id.generation_id;
        } else {
            self.objects.push(Some(object));
            self.generation_ids.push(id.generation_id);
        }
    }

    pub(crate) fn delete(&mut self, id: Id<T>) {
        if self.generation_ids.get(id.index) == Some(&id.generation_id) {
            if let Some(object) = self.objects.get_mut(id.index) {
                *object = None;
            }
        }
    }

    pub(crate) fn update(&mut self, context: &mut UpdateContext<'_>) {
        self.reduce_object_vec_size();
        let logged_errors = self
            .logged_errors
            .as_mut()
            .expect("cannot update locked object");
        for (index, generation_id, object) in self
            .objects
            .iter_mut()
            .enumerate()
            .zip(&self.generation_ids)
            .filter_map(|((i, o), &g)| o.as_mut().map(|o| (i, g, o)))
        {
            context.self_id = Some(Id::<T>::new(index, generation_id).into());
            if let Err(err) = object.update(context) {
                if logged_errors.insert(format!("{err}")) {
                    error!(
                        "error when updating object of type `{}`: {err}",
                        any::type_name::<T>(),
                    );
                }
            }
        }
    }

    pub(crate) fn lock(&mut self) -> crate::Result<Self> {
        let objects = self.checked_mut()?;
        objects.is_locked = true;
        Ok(Self {
            is_locked: false,
            objects: mem::take(&mut objects.objects),
            generation_ids: mem::take(&mut objects.generation_ids),
            logged_errors: objects.logged_errors.take(),
        })
    }

    pub(crate) fn unlock(&mut self, objects: &mut Self) {
        self.is_locked = false;
        self.objects = mem::take(&mut objects.objects);
        self.generation_ids = mem::take(&mut objects.generation_ids);
        self.logged_errors = objects.logged_errors.take();
    }

    pub(crate) fn checked(&self) -> crate::Result<&Self> {
        if self.is_locked {
            Err(Error::ObjectTypeAlreadyLocked(any::type_name::<T>()))
        } else {
            Ok(self)
        }
    }

    pub(crate) fn checked_mut(&mut self) -> crate::Result<&mut Self> {
        if self.is_locked {
            Err(Error::ObjectTypeAlreadyLocked(any::type_name::<T>()))
        } else {
            Ok(self)
        }
    }

    fn check_id(&self, id: Id<T>) -> crate::Result<()> {
        if self.generation_ids[id.index] == id.generation_id {
            Ok(())
        } else {
            Err(Error::ObjectNotFound(any::type_name::<T>()))
        }
    }

    fn reduce_object_vec_size(&mut self) {
        let removed_count = self
            .objects
            .iter()
            .map_while(|o| o.is_none().then_some(()))
            .count();
        self.objects.truncate(self.objects.len() - removed_count);
    }
}

impl<'a, T> IntoIterator for &'a Objects<T>
where
    T: Object,
{
    type Item = &'a T;
    type IntoIter = Flatten<Iter<'a, Option<T>>>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, T> IntoIterator for &'a mut Objects<T>
where
    T: Object,
{
    type Item = &'a mut T;
    type IntoIter = Flatten<IterMut<'a, Option<T>>>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}
