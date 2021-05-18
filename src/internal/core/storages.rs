use fxhash::{FxHashMap, FxHashSet};
use std::any::TypeId;

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
pub(super) struct EntityMainComponentTypeStorage(FxHashSet<TypeId>);

impl EntityMainComponentTypeStorage {
    /// Return whether the type is new for the group.
    pub(super) fn add(&mut self, entity_type: TypeId) -> bool {
        self.0.insert(entity_type)
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

    #[test]
    fn add_first_type() {
        let mut storage = EntityMainComponentTypeStorage::default();

        let is_new = storage.add(TypeId::of::<usize>());

        assert!(is_new);
    }

    #[test]
    fn add_different_type() {
        let mut storage = EntityMainComponentTypeStorage::default();
        storage.add(TypeId::of::<u32>());

        let is_new = storage.add(TypeId::of::<i64>());

        assert!(is_new);
    }

    #[test]
    fn add_same_type() {
        let mut storage = EntityMainComponentTypeStorage::default();
        storage.add(TypeId::of::<u32>());

        let is_new = storage.add(TypeId::of::<u32>());

        assert!(!is_new);
    }
}
