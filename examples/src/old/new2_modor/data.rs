use crate::new_modor::App;
use fxhash::FxHashMap;
use std::any::Any;
use std::hash::{Hash, Hasher};
use std::iter;
use std::ops::{Deref, DerefMut};
use std::sync::atomic::{AtomicUsize, Ordering};

// TODO: post_update method after drop AccessMut (e.g. to update the associated storage)

pub trait Data {
    type Storage: Storage; // TODO: app mapper callback to access object that implements "RealStorage"
                           // TODO: this will allow to have VecWithContext storages (different data -> different Vec but same storage)

    fn get(
        app: &App,
        key: <Self::Storage as Storage>::Key,
    ) -> Option<<Self::Storage as Storage>::Access<'_>> {
        app.storage::<Self::Storage>()
            .and_then(|storage| storage.get(key))
    }

    fn get_mut(
        app: &mut App,
        key: <Self::Storage as Storage>::Key,
    ) -> DataState<<Self::Storage as Storage>::AccessMut<'_>> {
        app.storage_mut::<Self::Storage>().get_mut(key)
    }

    fn take(
        app: &mut App,
        key: <Self::Storage as Storage>::Key,
        f: impl FnOnce(&mut App, DataState<<Self::Storage as Storage>::AccessMut<'_>>),
    ) {
        app.take_storage::<Self::Storage>(|app, storage| f(app, storage.get_mut(key)));
    }

    #[inline]
    fn take_each(
        app: &mut App,
        mut f: impl FnMut(&mut App, <Self::Storage as Storage>::AccessMut<'_>),
    ) {
        app.take_storage::<Self::Storage>(|app, storage| {
            for data in storage.iter_mut() {
                f(app, data);
            }
        });
    }

    #[inline]
    fn take_scope_each(
        app: &mut App,
        scope: <Self::Storage as Storage>::Scope,
        mut f: impl FnMut(&mut App, <Self::Storage as Storage>::AccessMut<'_>),
    ) {
        app.take_storage::<Self::Storage>(|app, storage| {
            for data in storage.scope_iter_mut(scope) {
                f(app, data);
            }
        });
    }

    fn iter(app: &App) -> impl Iterator<Item = <Self::Storage as Storage>::Access<'_>> {
        app.storage::<Self::Storage>()
            .into_iter()
            .flat_map(Storage::iter)
    }

    fn iter_mut(app: &mut App) -> impl Iterator<Item = <Self::Storage as Storage>::AccessMut<'_>> {
        app.storage_mut::<Self::Storage>().iter_mut()
    }

    fn scope_iter(
        app: &App,
        scope: <Self::Storage as Storage>::Scope,
    ) -> impl Iterator<Item = <Self::Storage as Storage>::Access<'_>> {
        app.storage::<Self::Storage>()
            .map(move |storage| storage.scope_iter(scope))
            .into_iter()
            .flatten()
    }

    fn scope_iter_mut(
        app: &mut App,
        scope: <Self::Storage as Storage>::Scope,
    ) -> impl Iterator<Item = <Self::Storage as Storage>::AccessMut<'_>> {
        app.storage_mut::<Self::Storage>().scope_iter_mut(scope)
    }

    fn scale(app: &mut App, max_key: <Self::Storage as Storage>::Key)
    where
        <Self::Storage as Storage>::Key: PartialEq + Eq,
    {
        app.storage_mut::<Self::Storage>().scale(max_key)
    }

    // TODO: add take + take_each
    // TODO: delete + delete_all
    // TODO: replace iter_mut_or_create by scale ? (no iterator returned)
}

// TODO: create also InnerStorage trait ?
pub trait Storage: Any + Default {
    type Key: Any;
    type Scope; // TODO: replace by actual type + requires that Self::Key: ScopeKey for scope_* methods
    type Access<'a>;
    type AccessMut<'a>;

    fn get(&self, key: Self::Key) -> Option<Self::Access<'_>>;

    fn get_mut(&mut self, key: Self::Key) -> DataState<Self::AccessMut<'_>>;

    fn iter(&self) -> impl Iterator<Item = Self::Access<'_>>;

    fn iter_mut(&mut self) -> impl Iterator<Item = Self::AccessMut<'_>>;

    fn scope_iter(&self, scope: Self::Scope) -> impl Iterator<Item = Self::Access<'_>>;

    fn scope_iter_mut(&mut self, scope: Self::Scope) -> impl Iterator<Item = Self::AccessMut<'_>>;

    fn scale(&mut self, max_key: Self::Key)
    where
        Self::Key: PartialEq + Eq;

    #[allow(unused_variables)]
    fn update(app: &mut App) {}
}

pub enum DataState<T> {
    New(T),
    Existing(T),
}

impl<T> Deref for DataState<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        let (Self::New(value) | Self::Existing(value)) = self;
        value
    }
}

impl<T> DerefMut for DataState<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        let (Self::New(value) | Self::Existing(value)) = self;
        value
    }
}

impl<T> DataState<T> {
    pub fn is_new(&self) -> bool {
        matches!(self, Self::New(_))
    }
}

#[derive(Debug)]
pub struct SingletonStorage<T> {
    inner: T,
    is_new: bool,
}

impl<T> Default for SingletonStorage<T>
where
    T: Default,
{
    fn default() -> Self {
        Self {
            inner: T::default(),
            is_new: true,
        }
    }
}

impl<T> Storage for SingletonStorage<T>
where
    T: Default + Any,
{
    type Key = ();
    type Scope = ();
    type Access<'a> = &'a T;
    type AccessMut<'a> = &'a mut T;

    fn get(&self, (): Self::Key) -> Option<Self::Access<'_>> {
        Some(&self.inner)
    }

    fn get_mut(&mut self, (): Self::Key) -> DataState<Self::AccessMut<'_>> {
        if self.is_new {
            DataState::New(&mut self.inner)
        } else {
            DataState::Existing(&mut self.inner)
        }
    }

    fn iter(&self) -> impl Iterator<Item = Self::Access<'_>> {
        iter::once(&self.inner)
    }

    fn iter_mut(&mut self) -> impl Iterator<Item = Self::AccessMut<'_>> {
        iter::once(&mut self.inner)
    }

    fn scope_iter(&self, (): Self::Scope) -> impl Iterator<Item = Self::Access<'_>> {
        self.iter()
    }

    fn scope_iter_mut(&mut self, (): Self::Scope) -> impl Iterator<Item = Self::AccessMut<'_>> {
        self.iter_mut()
    }

    fn scale(&mut self, (): Self::Key) {
        // do nothing
    }

    fn update(app: &mut App) {
        app.storage_mut::<Self>().is_new = false;
    }
}

#[derive(Copy, Clone, Debug, Eq)]
pub struct Scope {
    id: ScopeId,
    hash: u64,
}

impl PartialEq for Scope {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Hash for Scope {
    #[inline]
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write_u64(self.hash);
    }
}

impl Scope {
    pub const fn new(id: &'static str) -> Self {
        Self {
            id: ScopeId::Str(id),
            hash: Self::calculate_hash(id),
        }
    }

    pub fn unique() -> Self {
        static NEXT_ID: AtomicUsize = AtomicUsize::new(0);
        let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
        Self {
            id: ScopeId::Usize(id),
            hash: id as u64,
        }
    }

    pub const fn key<T>(self, value: T) -> Key<T> {
        Key { scope: self, value }
    }

    const fn calculate_hash(id: &'static str) -> u64 {
        let bytes = id.as_bytes();
        let mut hash = if bytes.is_empty() { 0 } else { bytes[0] as u64 };
        if bytes.len() > 1 {
            hash |= (bytes[1] as u64) << 8;
        }
        if bytes.len() > 2 {
            hash |= (bytes[2] as u64) << 16;
        }
        if bytes.len() > 3 {
            hash |= (bytes[3] as u64) << 24;
        }
        if bytes.len() > 4 {
            hash |= (bytes[bytes.len() - 1] as u64) << 32;
        }
        if bytes.len() > 5 {
            hash |= (bytes[bytes.len() - 2] as u64) << 40;
        }
        if bytes.len() > 6 {
            hash |= (bytes[bytes.len() - 3] as u64) << 48;
        }
        if bytes.len() > 7 {
            hash |= (bytes[bytes.len() - 4] as u64) << 56;
        }
        hash
    }
}

#[derive(Copy, Clone, Debug, Eq)]
enum ScopeId {
    Str(&'static str),
    Usize(usize),
}

impl PartialEq for ScopeId {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Usize(index1), Self::Usize(index2)) => index1 == index2,
            (Self::Str(id1), Self::Str(id2)) => id1.as_ptr() == id2.as_ptr() || id1 == id2,
            (Self::Usize(_), Self::Str(_)) | (Self::Str(_), Self::Usize(_)) => false,
        }
    }
}

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub struct Key<T> {
    pub(crate) scope: Scope,
    pub(crate) value: T,
}

#[derive(Debug, Default)]
pub struct VecStorage<T> {
    items: FxHashMap<Scope, Vec<T>>,
}

impl<T> Storage for VecStorage<T>
where
    T: Default + Any,
{
    type Key = Key<usize>;
    type Scope = Scope;
    type Access<'a> = &'a T;
    type AccessMut<'a> = &'a mut T;

    fn get(&self, key: Self::Key) -> Option<Self::Access<'_>> {
        self.items
            .get(&key.scope)
            .and_then(|items| items.get(key.value))
    }

    fn get_mut(&mut self, key: Self::Key) -> DataState<Self::AccessMut<'_>> {
        let items = self.items.entry(key.scope).or_default();
        if key.value < items.len() {
            DataState::Existing(&mut items[key.value])
        } else {
            (items.len()..=key.value).for_each(|_| items.push(T::default()));
            DataState::New(&mut items[key.value])
        }
    }

    fn iter(&self) -> impl Iterator<Item = Self::Access<'_>> {
        self.items.values().flatten()
    }

    fn iter_mut(&mut self) -> impl Iterator<Item = Self::AccessMut<'_>> {
        self.items.values_mut().flatten()
    }

    fn scope_iter(&self, scope: Self::Scope) -> impl Iterator<Item = Self::Access<'_>> {
        self.items
            .get(&scope)
            .map(|items| items.iter())
            .into_iter()
            .flatten()
    }

    fn scope_iter_mut(&mut self, scope: Self::Scope) -> impl Iterator<Item = Self::AccessMut<'_>> {
        self.items.entry(scope).or_default().iter_mut()
    }

    fn scale(&mut self, max_key: Self::Key) {
        let items = self.items.entry(max_key.scope).or_default();
        (items.len()..=max_key.value).for_each(|_| items.push(T::default()));
    }
}
