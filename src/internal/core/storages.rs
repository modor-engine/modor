use fxhash::{FxHashMap, FxHashSet};
use std::{any::TypeId, num::NonZeroUsize};

#[derive(Default)]
pub(super) struct ComponentTypeStorage(FxHashMap<TypeId, usize>);

impl ComponentTypeStorage {
    pub(super) fn idx(&self, type_id: TypeId) -> Option<usize> {
        self.0.get(&type_id).copied()
    }

    pub(super) fn add(&mut self, type_id: TypeId) -> usize {
        self.0.get(&type_id).copied().unwrap_or_else(|| {
            let type_idx = self.0.len();
            self.0.insert(type_id, type_idx);
            type_idx
        })
    }
}

#[derive(Default)]
pub(super) struct EntityTypeStorage(Vec<FxHashSet<TypeId>>);

impl EntityTypeStorage {
    /// Return whether the type is new for the group.
    pub(super) fn add(&mut self, group_idx: NonZeroUsize, entity_type: TypeId) -> bool {
        let group_idx = group_idx.get() - 1;
        (self.0.len()..=group_idx).for_each(|_| self.0.push(FxHashSet::default()));
        self.0[group_idx].insert(entity_type)
    }

    pub(super) fn delete(&mut self, group_idx: NonZeroUsize) {
        let group_idx = group_idx.get() - 1;
        (self.0.len()..=group_idx).for_each(|_| self.0.push(FxHashSet::default()));
        self.0[group_idx] = FxHashSet::default();
    }
}

#[cfg(test)]
mod tests_component_type_storage {
    use super::*;

    #[test]
    fn add_first_type() {
        let mut storage = ComponentTypeStorage::default();

        let type_idx = storage.add(TypeId::of::<u32>());

        assert_eq!(type_idx, 0);
        assert_eq!(storage.idx(TypeId::of::<u32>()), Some(0));
        assert_eq!(storage.idx(TypeId::of::<i64>()), None);
    }

    #[test]
    fn add_different_nonexisting_type() {
        let mut storage = ComponentTypeStorage::default();
        storage.add(TypeId::of::<u32>());

        let type_idx = storage.add(TypeId::of::<i64>());

        assert_eq!(type_idx, 1);
        assert_eq!(storage.idx(TypeId::of::<u32>()), Some(0));
        assert_eq!(storage.idx(TypeId::of::<i64>()), Some(1));
        assert_eq!(storage.idx(TypeId::of::<String>()), None);
    }

    #[test]
    fn add_existing_type() {
        let mut storage = ComponentTypeStorage::default();
        storage.add(TypeId::of::<u32>());

        let type_idx = storage.add(TypeId::of::<u32>());

        assert_eq!(type_idx, 0);
        assert_eq!(storage.idx(TypeId::of::<u32>()), Some(0));
        assert_eq!(storage.idx(TypeId::of::<i64>()), None);
    }
}

#[cfg(test)]
mod tests_entity_type_storage {
    use super::*;
    use std::convert::TryInto;

    #[test]
    fn add_first_type() {
        let mut storage = EntityTypeStorage::default();

        let is_new = storage.add(1.try_into().unwrap(), TypeId::of::<usize>());

        assert!(is_new);
    }

    #[test]
    fn add_different_type_with_same_group() {
        let mut storage = EntityTypeStorage::default();
        storage.add(1.try_into().unwrap(), TypeId::of::<u32>());

        let is_new = storage.add(1.try_into().unwrap(), TypeId::of::<i64>());

        assert!(is_new);
    }

    #[test]
    fn add_same_type_with_different_group() {
        let mut storage = EntityTypeStorage::default();
        storage.add(1.try_into().unwrap(), TypeId::of::<u32>());

        let is_new = storage.add(2.try_into().unwrap(), TypeId::of::<u32>());

        assert!(is_new);
    }

    #[test]
    fn add_same_type_with_same_group() {
        let mut storage = EntityTypeStorage::default();
        storage.add(1.try_into().unwrap(), TypeId::of::<u32>());

        let is_new = storage.add(1.try_into().unwrap(), TypeId::of::<u32>());

        assert!(!is_new);
    }

    #[test]
    fn delete_nonexisting_group() {
        let mut storage = EntityTypeStorage::default();

        storage.delete(2.try_into().unwrap());

        assert!(storage.add(2.try_into().unwrap(), TypeId::of::<u32>()));
    }

    #[test]
    fn delete_existing_group() {
        let mut storage = EntityTypeStorage::default();
        storage.add(2.try_into().unwrap(), TypeId::of::<u32>());

        storage.delete(2.try_into().unwrap());

        assert!(storage.add(2.try_into().unwrap(), TypeId::of::<u32>()));
    }
}
