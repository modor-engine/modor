use crate::new_modor::Storage;
use fxhash::FxHashMap;
use modor::log::Level;
use std::any;
use std::any::{Any, TypeId};

pub struct App {
    update_fn: fn(&mut Self),
    storages: FxHashMap<TypeId, Option<Box<dyn Any>>>,
    storage_update_fns: Vec<fn(&mut Self)>,
}

impl App {
    pub fn new(_level: Level, update_fn: fn(&mut Self)) -> Self {
        Self {
            update_fn,
            storages: FxHashMap::default(),
            storage_update_fns: vec![],
        }
    }

    pub fn update(&mut self) {
        (self.update_fn)(self);
        for update_fn in self.storage_update_fns.clone() {
            update_fn(self);
        }
    }

    // TODO: for next methods, maybe use T: Data instead of T: Storage + improve panic messages
    pub fn storage<T>(&self) -> Option<&T>
    where
        T: Storage,
    {
        self.storages.get(&TypeId::of::<T>()).map(|storage| {
            storage
                .as_ref()
                .unwrap_or_else(|| panic!("storage type `{}` already taken", any::type_name::<T>()))
                .downcast_ref::<T>()
                .expect("internal error: incorrect storage type")
        })
    }

    pub fn storage_mut<T>(&mut self) -> &mut T
    where
        T: Storage,
    {
        self.storages
            .entry(TypeId::of::<T>())
            .or_insert_with(|| {
                self.storage_update_fns.push(T::update);
                Some(Box::new(T::default()))
            })
            .as_mut()
            .unwrap_or_else(|| panic!("storage type `{}` already taken", any::type_name::<T>()))
            .downcast_mut::<T>()
            .expect("internal error: incorrect storage type")
    }

    pub fn take_storage<T>(&mut self, f: impl FnOnce(&mut Self, &mut T))
    where
        T: Storage,
    {
        let mut storage = self
            .storages
            .entry(TypeId::of::<T>())
            .or_insert_with(|| Some(Box::new(T::default())))
            .take()
            .unwrap_or_else(|| panic!("storage type `{}` already taken", any::type_name::<T>()));
        f(
            self,
            storage
                .downcast_mut::<T>()
                .expect("internal error: incorrect storage type"),
        );
        self.storages.insert(TypeId::of::<T>(), Some(storage));
    }
}
