use crate::new_modor::{App, DataState, Key, Scope, Storage};
use fxhash::FxHashMap;
use std::any::Any;

/*
app.get_mut::<T>()...
app.singleton_mut::<T>()...

 */

trait SingletonData: Any + Default {
    fn get(app: &App) -> Option<&Self> {
        app.singleton::<Self>()
    }

    fn get_mut(app: &mut App) -> DataState<&mut Self> {
        app.singleton_mut::<Self>()
    }

    fn take(app: &mut App, key: Key<usize>, f: impl FnOnce(&mut App, DataState<&mut Self>)) {
        app.take_storage::<Self>(|app, storage| f(app, storage.get_mut(key)));
    }

    #[inline]
    fn take_each(app: &mut App, mut f: impl FnMut(&mut App, &mut Self)) {
        app.take_storage::<Self>(|app, storage| {
            for data in storage.iter_mut() {
                f(app, data);
            }
        });
    }

    #[inline]
    fn take_scope_each(app: &mut App, scope: Scope, mut f: impl FnMut(&mut App, &mut Self)) {
        app.take_storage::<Self>(|app, storage| {
            for data in storage.scope_iter_mut(scope) {
                f(app, data);
            }
        });
    }

    fn iter(app: &App) -> impl Iterator<Item = &Self> {
        app.storage::<Self>().into_iter().flat_map(Storage::iter)
    }

    fn iter_mut(app: &mut App) -> impl Iterator<Item = &mut Self> {
        app.storage_mut::<Self>().iter_mut()
    }

    fn scope_iter(app: &App, scope: Scope) -> impl Iterator<Item = &Self> {
        app.storage::<Self>()
            .map(move |storage| storage.scope_iter(scope))
            .into_iter()
            .flatten()
    }

    fn scope_iter_mut(app: &mut App, scope: Scope) -> impl Iterator<Item = &mut Self> {
        app.storage_mut::<Self>().scope_iter_mut(scope)
    }

    fn scale(app: &mut App, max_key: Key<usize>) {
        app.storage_mut::<Self>().scale(max_key)
    }
}

trait VecData: Any + Default {
    fn get(app: &App, key: Key<usize>) -> Option<&Self> {
        app.storage::<Self>().and_then(|storage| storage.get(key))
    }

    fn get_mut(app: &mut App, key: Key<usize>) -> DataState<&mut Self> {
        app.storage_mut::<Self>().get_mut(key)
    }

    fn take(app: &mut App, key: Key<usize>, f: impl FnOnce(&mut App, DataState<&mut Self>)) {
        app.take_storage::<Self>(|app, storage| f(app, storage.get_mut(key)));
    }

    #[inline]
    fn take_each(app: &mut App, mut f: impl FnMut(&mut App, &mut Self)) {
        app.take_storage::<Self>(|app, storage| {
            for data in storage.iter_mut() {
                f(app, data);
            }
        });
    }

    #[inline]
    fn take_scope_each(app: &mut App, scope: Scope, mut f: impl FnMut(&mut App, &mut Self)) {
        app.take_storage::<Self>(|app, storage| {
            for data in storage.scope_iter_mut(scope) {
                f(app, data);
            }
        });
    }

    fn iter(app: &App) -> impl Iterator<Item = &Self> {
        app.storage::<Self>().into_iter().flat_map(Storage::iter)
    }

    fn iter_mut(app: &mut App) -> impl Iterator<Item = &mut Self> {
        app.storage_mut::<Self>().iter_mut()
    }

    fn scope_iter(app: &App, scope: Scope) -> impl Iterator<Item = &Self> {
        app.storage::<Self>()
            .map(move |storage| storage.scope_iter(scope))
            .into_iter()
            .flatten()
    }

    fn scope_iter_mut(app: &mut App, scope: Scope) -> impl Iterator<Item = &mut Self> {
        app.storage_mut::<Self>().scope_iter_mut(scope)
    }

    fn scale(app: &mut App, max_key: Key<usize>) {
        app.storage_mut::<Self>().scale(max_key)
    }
}

#[derive(Debug, Default)]
pub struct VecStorage<T> {
    items: FxHashMap<Scope, Vec<T>>,
}

impl<T> VecStorage<T>
where
    T: Default + Any,
{
    fn get(&self, key: Key<usize>) -> Option<&T> {
        self.items
            .get(&key.scope)
            .and_then(|items| items.get(key.value))
    }

    fn get_mut(&mut self, key: Key<usize>) -> DataState<&mut T> {
        let items = self.items.entry(key.scope).or_default();
        if key.value < items.len() {
            DataState::Existing(&mut items[key.value])
        } else {
            (items.len()..=key.value).for_each(|_| items.push(T::default()));
            DataState::New(&mut items[key.value])
        }
    }

    fn iter(&self) -> impl Iterator<Item = &T> {
        self.items.values().flatten()
    }

    fn iter_mut(&mut self) -> impl Iterator<Item = &mut T> {
        self.items.values_mut().flatten()
    }

    fn scope_iter(&self, scope: Scope) -> impl Iterator<Item = &T> {
        self.items
            .get(&scope)
            .map(|items| items.iter())
            .into_iter()
            .flatten()
    }

    fn scope_iter_mut(&mut self, scope: Scope) -> impl Iterator<Item = &mut T> {
        self.items.entry(scope).or_default().iter_mut()
    }

    fn scale(&mut self, max_key: Key<usize>) {
        let items = self.items.entry(max_key.scope).or_default();
        (items.len()..=max_key.value).for_each(|_| items.push(T::default()));
    }
}
