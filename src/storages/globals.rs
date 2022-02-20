use crate::utils;
use fxhash::FxHashMap;
use std::any::{Any, TypeId};
use std::sync::{RwLock, RwLockReadGuard, RwLockWriteGuard};
use typed_index_collections::TiVec;

#[derive(Default)]
pub(crate) struct GlobalStorage {
    idxs: FxHashMap<TypeId, GlobalIdx>,
    globals: TiVec<GlobalIdx, Option<Box<dyn Any + Sync + Send>>>,
    has_been_created: TiVec<GlobalIdx, bool>,
}

impl GlobalStorage {
    pub(crate) fn has_been_created(&self, idx: GlobalIdx) -> bool {
        self.has_been_created[idx]
    }

    pub(crate) fn exists(&self, idx: GlobalIdx) -> bool {
        self.globals.get(idx).map_or(false, Option::is_some)
    }

    pub(crate) fn read<G>(&self) -> Option<RwLockReadGuard<'_, G>>
    where
        G: Any,
    {
        self.idxs
            .get(&TypeId::of::<G>())
            .and_then(|&g| self.globals[g].as_ref())
            .map(|g| {
                g.downcast_ref::<RwLock<G>>()
                    .expect("internal error: wrong type used to read global")
                    .try_read()
                    .expect("internal error: cannot read global")
            })
    }

    pub(crate) fn write<G>(&self) -> Option<RwLockWriteGuard<'_, G>>
    where
        G: Any,
    {
        self.idxs
            .get(&TypeId::of::<G>())
            .and_then(|&g| self.globals[g].as_ref())
            .map(|g| {
                g.downcast_ref::<RwLock<G>>()
                    .expect("internal error: wrong type used to write global")
                    .try_write()
                    .expect("internal error: cannot write global")
            })
    }

    pub(crate) fn idx_or_register(&mut self, global_type: TypeId) -> GlobalIdx {
        *self.idxs.entry(global_type).or_insert_with(|| {
            let idx = self.globals.next_key();
            self.globals.push(None);
            self.has_been_created.push(false);
            idx
        })
    }

    pub(super) fn replace_or_add<G>(&mut self, global: G)
    where
        G: Any + Sync + Send,
    {
        let idx = self.idx_or_register(TypeId::of::<G>());
        let boxed_global = Box::new(RwLock::new(global));
        utils::set_value(&mut self.globals, idx, Some(boxed_global));
        self.has_been_created[idx] = true;
    }
}

idx_type!(pub(crate) GlobalIdx);

#[cfg(test)]
mod global_storage_tests {
    use crate::storages::globals::GlobalStorage;
    use std::any::TypeId;

    #[test]
    fn configure_globals() {
        let mut storage = GlobalStorage::default();
        let global1_idx = storage.idx_or_register(TypeId::of::<u32>());
        let global2_idx = storage.idx_or_register(TypeId::of::<u32>());
        let global3_idx = storage.idx_or_register(TypeId::of::<i64>());
        storage.replace_or_add(10_u32);
        storage.replace_or_add(20_u32);
        storage.replace_or_add(30_i8);
        assert_eq!([global1_idx, global2_idx], [0.into(); 2]);
        assert_eq!(global3_idx, 1.into());
        assert!(storage.has_been_created(global1_idx));
        assert!(!storage.has_been_created(global3_idx));
        assert!(storage.has_been_created(2.into()));
        assert!(storage.exists(global1_idx));
        assert!(!storage.exists(global3_idx));
        assert!(storage.exists(2.into()));
        assert_eq!(storage.read::<u32>().as_deref(), Some(&20_u32));
        assert_eq!(storage.read::<i64>().as_deref(), None);
        assert_eq!(storage.read::<i8>().as_deref(), Some(&30_i8));
        assert_eq!(storage.read::<u16>().as_deref(), None);
        assert_eq!(storage.write::<u32>().as_deref_mut(), Some(&mut 20_u32));
        assert_eq!(storage.write::<i64>().as_deref_mut(), None);
        assert_eq!(storage.write::<i8>().as_deref_mut(), Some(&mut 30_i8));
        assert_eq!(storage.write::<u16>().as_deref_mut(), None);
    }
}
