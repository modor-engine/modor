use crate::{App, FromApp, State, StateHandle};
use derivative::Derivative;
use log::error;
use std::iter::Flatten;
use std::mem;
use std::ops::Deref;
use std::slice::{Iter, IterMut};
use std::sync::{Arc, Mutex};

/// A trait for defining a shared value.
pub trait Global: FromApp {
    /// Initializes the shared value.
    ///
    /// `index` is the unique index of the shared value.
    ///
    /// This method is called just after [`FromApp::from_app`].
    #[allow(unused_variables)]
    fn init(&mut self, app: &mut App, index: usize) {}
}

/// A globally shared value of type `T`.
///
/// # Examples
///
/// ```
/// # use modor::*;
/// #
/// #[derive(FromApp, Global)]
/// struct SharedValue(usize);
///
/// fn create_glob(app: &mut App) -> Glob<SharedValue> {
///     let glob = Glob::<SharedValue>::from_app(app);
///     assert_eq!(glob.get(app).0, 0);
///     glob.get_mut(app).0 = 42;
///     assert_eq!(glob.get(app).0, 42);
///     glob
/// }
/// ```
#[derive(Derivative)]
#[derivative(
    Debug(bound = ""),
    Hash(bound = ""),
    PartialEq(bound = ""),
    Eq(bound = ""),
    PartialOrd(bound = ""),
    Ord(bound = "")
)]
pub struct Glob<T> {
    index: usize,
    #[derivative(
        Hash = "ignore",
        PartialEq = "ignore",
        PartialOrd = "ignore",
        Ord = "ignore"
    )]
    globals: StateHandle<Globals<T>>,
    #[derivative(
        Hash = "ignore",
        PartialEq = "ignore",
        PartialOrd = "ignore",
        Ord = "ignore"
    )]
    lifetime: Arc<GlobLifetime>,
}

impl<T> FromApp for Glob<T>
where
    T: Global,
{
    fn from_app(app: &mut App) -> Self {
        let globals = app.handle::<Globals<T>>();
        let index = globals.get_mut(app).next_index();
        let value = T::from_app_with(app, |value, app| value.init(app, index));
        let lifetime = globals.get_mut(app).register(index, value);
        Self {
            index: lifetime.index,
            globals,
            lifetime: Arc::from(lifetime),
        }
    }
}

impl<T> Glob<T>
where
    T: 'static,
{
    /// Returns the unique index of the shared value.
    ///
    /// Note that in case the [`Glob<T>`] and all associated [`GlobRef<T>`]s are dropped, this index
    /// can be reused for a new [`Glob<T>`].
    #[inline]
    pub fn index(&self) -> usize {
        self.index
    }

    /// Returns an immutable reference to the shared value.
    ///
    /// # Panics
    ///
    /// This will panic if a shared value of type `T` is already mutably borrowed.
    pub fn get<'a>(&self, app: &'a App) -> &'a T {
        &self.globals.get(app)[self]
    }

    /// Returns a mutable reference to the shared value.
    ///
    /// # Panics
    ///
    /// This will panic if a shared value of type `T` is already mutably borrowed.
    pub fn get_mut<'a>(&self, app: &'a mut App) -> &'a mut T {
        &mut self.globals.get_mut(app)[self]
    }

    /// Borrows the shared value without borrowing the app.
    ///
    /// The method returns the output of `f`.
    ///
    /// # Panics
    ///
    /// This will panic if a shared value of type `T` is already mutably borrowed.
    pub fn take<O>(&self, app: &mut App, f: impl FnOnce(&mut T, &mut App) -> O) -> O {
        self.globals.take(app, |globals, app| {
            let value = globals.items[self.index()]
                .as_mut()
                .expect("internal error: invalid index");
            f(value, app)
        })
    }

    /// Returns an immutable reference with static lifetime to the shared value.
    pub fn to_ref(&self) -> GlobRef<T> {
        GlobRef(Self {
            index: self.index,
            globals: self.globals,
            lifetime: self.lifetime.clone(),
        })
    }
}

/// A reference with static lifetime to a shared value of type `T`.
///
/// The reference remains valid even after the original [`Glob<T>`] is dropped.
///
/// # Examples
///
/// ```
/// # use modor::*;
/// #
/// #[derive(FromApp, Global)]
/// struct SharedValue(usize);
///
/// fn create_glob_ref(app: &mut App) -> GlobRef<SharedValue> {
///     let glob = Glob::<SharedValue>::from_app(app);
///     let ref_ = glob.to_ref();
///     glob.get_mut(app).0 = 42;
///     assert_eq!(ref_.get(app).0, 42);
///     ref_
/// }
/// ```
#[derive(Derivative)]
#[derivative(
    Debug(bound = ""),
    Hash(bound = ""),
    PartialEq(bound = ""),
    Eq(bound = ""),
    PartialOrd(bound = ""),
    Ord(bound = "")
)]
pub struct GlobRef<T>(Glob<T>);

impl<T> Clone for GlobRef<T>
where
    T: 'static,
{
    fn clone(&self) -> Self {
        self.0.to_ref()
    }
}

impl<T> Deref for GlobRef<T> {
    type Target = Glob<T>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// A container that stores all shared values of type `T`.
///
/// # Examples
///
/// ```
/// # use modor::*;
/// #
/// fn print_all_strings(app: &mut App) {
///     for string in app.get_mut::<Globals<&'static str>>().iter() {
///         println!("{}", string);
///     }
/// }
/// ```
#[derive(Debug, Derivative)]
#[derivative(Default(bound = ""))]
pub struct Globals<T> {
    items: Vec<Option<T>>,
    deleted_items: Vec<(usize, T)>,
    deleted_indexes: Arc<Mutex<Vec<usize>>>,
    available_indexes: Vec<usize>,
    next_index: usize,
}

impl<T> State for Globals<T>
where
    T: 'static,
{
    fn update(&mut self, _app: &mut App) {
        self.available_indexes
            .extend(self.deleted_items.drain(..).map(|(index, _)| index));
        let deleted_indexes = mem::take(
            &mut *self
                .deleted_indexes
                .lock()
                .expect("cannot lock deleted glob indexes"),
        );
        for index in deleted_indexes {
            self.deleted_items.push((
                index,
                self.items[index]
                    .take()
                    .expect("internal error: missing glob"),
            ));
        }
    }
}

impl<T> Globals<T> {
    /// Returns the indexes and values dropped since last update.
    pub fn deleted_items(&self) -> &[(usize, T)] {
        &self.deleted_items
    }

    /// Returns an immutable reference to the value corresponding to a given `index` if it exists.
    pub fn get(&self, index: usize) -> Option<&T> {
        self.items.get(index).and_then(|item| item.as_ref())
    }

    /// Returns a mutable reference to the value corresponding to a given `index` if it exists.
    pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        self.items.get_mut(index).and_then(|item| item.as_mut())
    }

    /// Returns an iterator on immutable references to all values.
    pub fn iter(&self) -> Flatten<Iter<'_, Option<T>>> {
        self.items.iter().flatten()
    }

    /// Returns an iterator on mutable references to all values.
    pub fn iter_mut(&mut self) -> Flatten<IterMut<'_, Option<T>>> {
        self.items.iter_mut().flatten()
    }

    /// Returns an iterator on immutable references to all values with their index.
    pub fn iter_enumerated(&self) -> impl Iterator<Item = (usize, &T)> {
        self.items
            .iter()
            .enumerate()
            .filter_map(|(index, item)| item.as_ref().map(|item| (index, item)))
    }

    /// Returns an iterator on mutable references to all values with their index.
    pub fn iter_mut_enumerated(&mut self) -> impl Iterator<Item = (usize, &mut T)> {
        self.items
            .iter_mut()
            .enumerate()
            .filter_map(|(index, item)| item.as_mut().map(|item| (index, item)))
    }

    fn next_index(&mut self) -> usize {
        self.available_indexes.pop().unwrap_or_else(|| {
            let index = self.next_index;
            self.next_index += 1;
            index
        })
    }

    fn register(&mut self, index: usize, item: T) -> GlobLifetime {
        let lifetime = GlobLifetime {
            index,
            deleted_indexes: self.deleted_indexes.clone(),
        };
        for _ in self.items.len()..=index {
            self.items.push(None);
        }
        self.items[index] = Some(item);
        lifetime
    }
}

impl<'a, T> IntoIterator for &'a Globals<T> {
    type Item = &'a T;
    type IntoIter = Flatten<Iter<'a, Option<T>>>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, T> IntoIterator for &'a mut Globals<T> {
    type Item = &'a mut T;
    type IntoIter = Flatten<IterMut<'a, Option<T>>>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

impl<T> std::ops::Index<&Glob<T>> for Globals<T> {
    type Output = T;

    fn index(&self, glob: &Glob<T>) -> &Self::Output {
        self.items[glob.index]
            .as_ref()
            .expect("internal error: invalid index")
    }
}

impl<T> std::ops::IndexMut<&Glob<T>> for Globals<T> {
    fn index_mut(&mut self, glob: &Glob<T>) -> &mut Self::Output {
        self.items[glob.index]
            .as_mut()
            .expect("internal error: invalid index")
    }
}

#[derive(Debug)]
struct GlobLifetime {
    index: usize,
    deleted_indexes: Arc<Mutex<Vec<usize>>>,
}

impl Drop for GlobLifetime {
    fn drop(&mut self) {
        match self.deleted_indexes.lock() {
            Ok(mut indexes) => indexes.push(self.index),
            Err(err) => error!("Error: {err}"), // no-coverage (difficult to test poisoning)
        }
    }
}
