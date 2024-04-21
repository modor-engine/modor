#![allow(clippy::non_canonical_partial_ord_impl)] // warnings caused by Derivative

use crate::{Context, Node, RootNode, RootNodeHandle, Visit};
use derivative::Derivative;
use log::error;
use std::iter::Flatten;
use std::marker::PhantomData;
use std::mem;
use std::slice::Iter;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

/// A globally shared value of type `T`.
///
/// # Examples
///
/// ```
/// # use modor::*;
/// #
/// fn create_glob(ctx: &mut Context<'_>) -> Glob<&'static str> {
///     let glob = Glob::new(ctx, "shared value");
///     assert_eq!(glob.get(ctx), &"shared value");
///     glob
/// }
/// ```
#[derive(Debug, Derivative)]
#[derivative(
    Hash(bound = ""),
    PartialEq(bound = ""),
    Eq(bound = ""),
    PartialOrd(bound = ""),
    Ord(bound = "")
)]
pub struct Glob<T> {
    ref_: GlobRef<T>,
    phantom: PhantomData<fn(T)>,
}

impl<T> Glob<T>
where
    T: 'static,
{
    /// Creates a new shared `value`.
    pub fn new(ctx: &mut Context<'_>, value: T) -> Self {
        let globals = ctx.root::<Globals<T>>();
        Self {
            ref_: GlobRef {
                index: globals.get_mut(ctx).register(value).into(),
                globals,
                phantom: PhantomData,
            },
            phantom: PhantomData,
        }
    }

    /// Returns the unique index of the shared value.
    ///
    /// Note that in case the [`Glob<T>`] and all associated [`GlobRef<T>`]s are dropped, this index
    /// can be reused for a new [`Glob<T>`].
    #[inline]
    pub fn index(&self) -> usize {
        self.as_ref().index()
    }

    /// Returns an immutable reference to the shared value.
    pub fn get<'a>(&self, ctx: &'a Context<'_>) -> &'a T {
        &self.ref_.globals.get(ctx)[self.index()]
    }

    /// Returns a mutable reference to the shared value.
    pub fn get_mut<'a>(&self, ctx: &'a mut Context<'_>) -> &'a mut T {
        self.ref_.globals.get_mut(ctx).items[self.index()]
            .as_mut()
            .expect("internal error: invalid index")
    }
}

impl<T> AsRef<GlobRef<T>> for Glob<T> {
    #[inline]
    fn as_ref(&self) -> &GlobRef<T> {
        &self.ref_
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
/// fn create_glob_ref(ctx: &mut Context<'_>) -> GlobRef<&'static str> {
///     let glob = Glob::new(ctx, "shared value");
///     let ref_ = glob.as_ref().clone();
///     assert_eq!(ref_.get(ctx), &"shared value");
///     ref_
/// }
/// ```
#[derive(Debug, Derivative)]
#[derivative(
    Clone(bound = ""),
    Hash(bound = ""),
    PartialEq(bound = ""),
    Eq(bound = ""),
    PartialOrd(bound = ""),
    Ord(bound = "")
)]
pub struct GlobRef<T> {
    index: Arc<Index>,
    globals: RootNodeHandle<Globals<T>>,
    phantom: PhantomData<fn(T)>,
}

impl<T> GlobRef<T>
where
    T: 'static,
{
    /// Returns the unique index of the shared value.
    ///
    /// Note that in case the [`Glob<T>`] and all associated [`GlobRef<T>`]s are dropped, this index
    /// can be reused for a new [`Glob<T>`].
    #[inline]
    pub fn index(&self) -> usize {
        self.index.index
    }

    /// Returns an immutable reference to the shared value.
    pub fn get<'a>(&self, ctx: &'a Context<'_>) -> &'a T {
        &self.globals.get(ctx)[self.index.index]
    }
}

/// A container that stores all shared values of type `T`.
///
/// # Examples
///
/// ```
/// # use modor::*;
/// #
/// fn access_glob(ctx: &mut Context<'_>, index: usize) -> &'static str {
///     ctx.root::<Globals<&'static str>>().get(ctx)[index]
/// }
/// ```
#[derive(Debug)]
pub struct Globals<T> {
    indexes: Arc<IndexPool>,
    items: Vec<Option<T>>,
    deleted_items: Vec<(usize, T)>,
}

impl<T> RootNode for Globals<T>
where
    T: 'static,
{
    fn on_create(_ctx: &mut Context<'_>) -> Self {
        Self {
            indexes: Arc::default(),
            items: vec![],
            deleted_items: vec![],
        }
    }
}

impl<T> Node for Globals<T> {
    fn on_enter(&mut self, _ctx: &mut Context<'_>) {
        self.deleted_items.clear();
        for &index in &self.indexes.take_deleted_indexes() {
            self.deleted_items.push((
                index,
                self.items[index]
                    .take()
                    .expect("internal error: missing item in arena"),
            ));
        }
    }
}

impl<T> Visit for Globals<T> {
    fn visit(&mut self, _ctx: &mut Context<'_>) {}
}

impl<T> Globals<T> {
    /// Returns the indexes and values dropped since last update of the node.
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

    fn register(&mut self, item: T) -> Index {
        let index = self.indexes.generate();
        for _ in self.items.len()..=index.index {
            self.items.push(None);
        }
        self.items[index.index] = Some(item);
        index
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

#[derive(Debug, Default)]
struct IndexPool {
    deleted_indexes: Mutex<Vec<usize>>,
    available_indexes: Mutex<Vec<usize>>,
    next_index: AtomicUsize,
}

impl IndexPool {
    const ERROR: &'static str = "cannot access index pool";

    fn generate(self: &Arc<Self>) -> Index {
        Index {
            index: if let Some(index) = self.available_indexes.lock().expect(Self::ERROR).pop() {
                index
            } else {
                self.next_index.fetch_add(1, Ordering::Relaxed)
            },
            pool: self.clone(),
        }
    }

    fn take_deleted_indexes(&self) -> Vec<usize> {
        let indexes = mem::take(&mut *self.deleted_indexes.lock().expect(Self::ERROR));
        self.available_indexes
            .lock()
            .expect(Self::ERROR)
            .extend_from_slice(&indexes);
        indexes
    }
}

#[derive(Debug, Derivative)]
#[derivative(Hash, PartialEq, Eq, PartialOrd, Ord)]
struct Index {
    index: usize,
    #[derivative(
        Hash = "ignore",
        PartialEq = "ignore",
        PartialOrd = "ignore",
        Ord = "ignore"
    )]
    pool: Arc<IndexPool>,
}

impl Drop for Index {
    fn drop(&mut self) {
        match self.pool.deleted_indexes.lock() {
            Ok(mut indexes) => indexes.push(self.index),
            Err(err) => error!("error: {err}"), // no-coverage (difficult to test poisoning)
        }
    }
}
