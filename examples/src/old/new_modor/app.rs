use crate::new_modor::{Data, Scope, Singleton, State, Storage};
use fxhash::{FxHashMap, FxHashSet};
use modor::log::Level;
use std::any::{Any, TypeId};
use std::ops::Range;
use std::{any, iter};

pub struct App {
    singletons: FxHashMap<TypeId, Option<Box<dyn Any>>>,
    storages: FxHashMap<Scope, FxHashMap<TypeId, Box<dyn Any>>>,
    update_fns: FxHashSet<fn(&mut Self)>,
    deleted_scopes: Vec<Scope>,
}

impl App {
    pub fn new(_level: Level, update_fn: fn(&mut Self)) -> Self {
        Self {
            singletons: FxHashMap::default(),
            storages: FxHashMap::default(),
            update_fns: iter::once(update_fn).collect(),
            deleted_scopes: vec![],
        }
    }

    pub fn update(&mut self) {
        for update_fn in self.update_fns.clone() {
            update_fn(self);
            for scope in self.deleted_scopes.drain(..) {
                self.storages.remove(&scope);
            }
        }
    }

    pub fn scope(&mut self, scope: Scope) -> ScopedApp<'_> {
        let storages = self.storages.entry(scope).or_default();
        ScopedApp {
            storage: storages,
            singletons: &mut self.singletons,
            update_fns: &mut self.update_fns,
            deleted_scopes: &mut self.deleted_scopes,
        }
    }

    pub fn for_each_scope(&mut self, mut f: impl FnMut(ScopedApp<'_>)) {
        for storage in self.storages.values_mut() {
            f(ScopedApp {
                storage,
                singletons: &mut self.singletons,
                update_fns: &mut self.update_fns,
                deleted_scopes: &mut self.deleted_scopes,
            });
        }
    }

    pub fn delete_scope(&mut self, scope: Scope) {
        self.storages.remove(&scope);
    }

    pub fn single_mut<T>(&mut self) -> &mut T
    where
        T: Singleton,
    {
        self.singletons
            .entry(TypeId::of::<T>())
            .or_insert_with(|| {
                self.update_fns.insert(T::update);
                Some(Box::new(T::default()))
            })
            .as_mut()
            .unwrap_or_else(|| panic!("singleton type `{}` already taken", any::type_name::<T>()))
            .downcast_mut::<T>()
            .expect("internal error: incorrect singleton type")
    }
}

pub struct ScopedApp<'a> {
    storage: &'a mut FxHashMap<TypeId, Box<dyn Any>>,
    singletons: &'a mut FxHashMap<TypeId, Option<Box<dyn Any>>>,
    update_fns: &'a mut FxHashSet<fn(&mut App)>,
    deleted_scopes: &'a mut Vec<Scope>,
}

impl ScopedApp<'_> {
    pub fn delete_scope(&mut self, scope: Scope) {
        self.deleted_scopes.push(scope);
    }

    pub fn single_mut<T>(&mut self) -> &mut T
    where
        T: Singleton,
    {
        self.singletons
            .entry(TypeId::of::<T>())
            .or_insert_with(|| {
                self.update_fns.insert(T::update);
                Some(Box::new(T::default()))
            })
            .as_mut()
            .unwrap_or_else(|| panic!("singleton type `{}` already taken", any::type_name::<T>()))
            .downcast_mut::<T>()
            .expect("internal error: incorrect singleton type")
    }

    pub fn get_mut<T>(&mut self, index: usize) -> &mut T
    where
        T: Data,
    {
        self.storage().get_mut(index)
    }

    pub fn iter_mut<T>(&mut self) -> impl Iterator<Item = &mut T>
    where
        T: Data,
    {
        self.storage().iter_mut()
    }

    pub fn range_iter_mut<T>(&mut self, range: Range<usize>) -> impl Iterator<Item = &mut T>
    where
        T: Data,
    {
        self.storage::<T>().range_iter_mut(range)
    }

    fn storage<T>(&mut self) -> &mut Storage<T>
    where
        T: Data,
    {
        self.storage
            .entry(TypeId::of::<T>())
            .or_insert_with(|| {
                self.update_fns.insert(T::update);
                Box::new(Storage::<T>::default())
            })
            .as_mut()
            .downcast_mut::<Storage<T>>()
            .expect("internal error: incorrect data type")
    }
}
