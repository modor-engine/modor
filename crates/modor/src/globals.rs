use crate::{App, FromApp, Node, RootNode, RootNodeHandle, Visit};
use derivative::Derivative;
use log::error;
use std::iter::Flatten;
use std::mem;
use std::ops::Deref;
use std::slice::Iter;
use std::sync::{Arc, Mutex};

/// A globally shared value of type `T`.
///
/// # Examples
///
/// ```
/// # use modor::*;
/// #
/// fn create_glob(app: &mut App) -> Glob<&'static str> {
///     let glob = Glob::from_app(app);
///     assert_eq!(glob.get(app), "");
///     *glob.get_mut(app) = "shared value";
///     assert_eq!(glob.get(app), &"shared value");
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
    globals: RootNodeHandle<Globals<T>>,
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
    T: FromApp,
{
    fn from_app(app: &mut App) -> Self {
        let globals = app.handle::<Globals<T>>();
        let value = T::from_app(app);
        let lifetime = globals.get_mut(app).register(value);
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
    pub fn get<'a>(&self, app: &'a App) -> &'a T {
        &self.globals.get(app)[self.index()]
    }

    /// Returns a mutable reference to the shared value.
    pub fn get_mut<'a>(&self, app: &'a mut App) -> &'a mut T {
        self.globals.get_mut(app).items[self.index()]
            .as_mut()
            .expect("internal error: invalid index")
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

/// An immutable reference with static lifetime to a shared value of type `T`.
///
/// The reference remains valid even after the [`Glob<T>`] is dropped.
///
/// # Examples
///
/// ```
/// # use modor::*;
/// #
/// fn create_glob_ref(app: &mut App) -> GlobRef<&'static str> {
///     let glob = Glob::from_app(app);
///     let ref_ = glob.to_ref();
///     *glob.get_mut(app) = "shared value";
///     assert_eq!(ref_.get(app), &"shared value");
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

impl<T> RootNode for Globals<T>
where
    T: 'static,
{
    fn on_create(_app: &mut App) -> Self {
        Self::default()
    }
}

impl<T> Node for Globals<T> {
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

impl<T> Visit for Globals<T> {
    fn visit(&mut self, _app: &mut App) {}
}

impl<T> Globals<T> {
    /// Returns the indexes and values dropped since last update of the singleton.
    pub fn deleted_items(&self) -> &[(usize, T)] {
        &self.deleted_items
    }

    /// Returns the value corresponding to a given `index` if it exists.
    pub fn get(&self, index: usize) -> Option<&T> {
        self.items.get(index).and_then(|item| item.as_ref())
    }

    /// Returns an iterator on all values.
    pub fn iter(&self) -> Flatten<Iter<'_, Option<T>>> {
        self.items.iter().flatten()
    }

    /// Returns an iterator on all values with their index.
    pub fn iter_enumerated(&self) -> impl Iterator<Item = (usize, &T)> {
        self.items
            .iter()
            .enumerate()
            .filter_map(|(index, item)| item.as_ref().map(|item| (index, item)))
    }

    fn register(&mut self, item: T) -> GlobLifetime {
        let lifetime = GlobLifetime {
            index: self.available_indexes.pop().unwrap_or_else(|| {
                let index = self.next_index + 1;
                self.next_index += 1;
                index
            }),
            deleted_indexes: self.deleted_indexes.clone(),
        };
        for _ in self.items.len()..=lifetime.index {
            self.items.push(None);
        }
        self.items[lifetime.index] = Some(item);
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

impl<T> std::ops::Index<usize> for Globals<T> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        self.items[index].as_ref().expect("invalid index")
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
