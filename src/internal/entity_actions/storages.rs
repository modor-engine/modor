use crate::internal::entity_actions::data::AddComponentFn;
use fxhash::FxHashSet;
use std::any::{Any, TypeId};
use std::mem;

#[derive(Default)]
pub(super) struct DeletedEntitiesStorage(Vec<bool>);

impl DeletedEntitiesStorage {
    pub(super) fn is_marked_as_deleted(&self, entity_idx: usize) -> bool {
        self.0.get(entity_idx).copied().unwrap_or(false)
    }

    pub(super) fn add(&mut self, entity_idx: usize) {
        (self.0.len()..=entity_idx).for_each(|_| self.0.push(false));
        self.0[entity_idx] = true;
    }

    pub(super) fn delete(&mut self, entity_idx: usize) {
        (self.0.len()..=entity_idx).for_each(|_| self.0.push(false));
        self.0[entity_idx] = false;
    }
}

#[derive(Default)]
pub(super) struct AddedComponentsStorage(Vec<Vec<AddComponentFn>>);

impl AddedComponentsStorage {
    pub(super) fn add(&mut self, entity_idx: usize, add_component_fn: AddComponentFn) {
        (self.0.len()..=entity_idx).for_each(|_| self.0.push(Vec::new()));
        self.0[entity_idx].push(add_component_fn);
    }

    pub(super) fn remove(&mut self, entity_idx: usize) -> Vec<AddComponentFn> {
        (self.0.len()..=entity_idx).for_each(|_| self.0.push(Vec::new()));
        mem::take(&mut self.0[entity_idx])
    }

    pub(super) fn reset(&mut self, entity_idx: usize) {
        (self.0.len()..=entity_idx).for_each(|_| self.0.push(Vec::new()));
        self.0[entity_idx] = Vec::new();
    }
}

#[derive(Default)]
pub(super) struct DeletedComponentsStorage(Vec<FxHashSet<TypeId>>);

impl DeletedComponentsStorage {
    pub(super) fn add<C>(&mut self, entity_idx: usize)
    where
        C: Any,
    {
        (self.0.len()..=entity_idx).for_each(|_| self.0.push(FxHashSet::default()));
        self.0[entity_idx].insert(TypeId::of::<C>());
    }

    pub(super) fn remove(&mut self, entity_idx: usize) -> FxHashSet<TypeId> {
        (self.0.len()..=entity_idx).for_each(|_| self.0.push(FxHashSet::default()));
        mem::take(&mut self.0[entity_idx])
    }

    pub(super) fn reset(&mut self, entity_idx: usize) {
        (self.0.len()..=entity_idx).for_each(|_| self.0.push(FxHashSet::default()));
        self.0[entity_idx] = FxHashSet::default();
    }
}

#[derive(Default)]
pub(super) struct ModifiedEntitiesStorage(FxHashSet<usize>);

impl ModifiedEntitiesStorage {
    pub(super) fn idxs(&self) -> impl Iterator<Item = usize> + '_ {
        self.0.iter().copied()
    }

    pub(super) fn add(&mut self, entity_idx: usize) {
        self.0.insert(entity_idx);
    }

    pub(super) fn reset(&mut self) {
        self.0.clear();
    }
}

#[cfg(test)]
mod tests_deleted_entities_storage {
    use super::*;

    #[test]
    fn add_entity() {
        let mut storage = DeletedEntitiesStorage::default();

        storage.add(1);

        assert!(!storage.is_marked_as_deleted(0));
        assert!(storage.is_marked_as_deleted(1));
        assert!(!storage.is_marked_as_deleted(2));
    }

    #[test]
    fn delete_nonexisting_entity() {
        let mut storage = DeletedEntitiesStorage::default();

        storage.delete(1);

        assert!(!storage.is_marked_as_deleted(0));
        assert!(!storage.is_marked_as_deleted(1));
    }

    #[test]
    fn delete_existing_entity() {
        let mut storage = DeletedEntitiesStorage::default();
        storage.add(1);

        storage.delete(1);

        assert!(!storage.is_marked_as_deleted(0));
        assert!(!storage.is_marked_as_deleted(1));
    }
}

#[cfg(test)]
mod tests_added_components_storage {
    use super::*;

    #[test]
    fn add_components_for_entity() {
        let mut storage = AddedComponentsStorage::default();

        storage.add(1, Box::new(|_| ()));
        storage.add(1, Box::new(|_| ()));

        assert_eq!(storage.remove(0).len(), 0);
        assert_eq!(storage.remove(1).len(), 2);
        assert_eq!(storage.remove(2).len(), 0);
    }

    #[test]
    fn remove_components_for_entity() {
        let mut storage = AddedComponentsStorage::default();
        storage.add(1, Box::new(|_| ()));

        let component_adders = storage.remove(1);

        assert_eq!(component_adders.len(), 1);
        assert_eq!(storage.remove(1).len(), 0);
    }

    #[test]
    fn reset_entity() {
        let mut storage = AddedComponentsStorage::default();
        storage.add(1, Box::new(|_| ()));
        storage.add(2, Box::new(|_| ()));

        storage.reset(1);

        assert_eq!(storage.remove(1).len(), 0);
        assert_eq!(storage.remove(2).len(), 1);
    }
}

#[cfg(test)]
mod tests_deleted_components_storage {
    use super::*;
    use std::iter;

    #[test]
    fn add_components_for_entity() {
        let mut storage = DeletedComponentsStorage::default();

        storage.add::<u32>(1);
        storage.add::<i64>(1);

        assert_eq!(
            storage.remove(1),
            [TypeId::of::<u32>(), TypeId::of::<i64>()]
                .iter()
                .copied()
                .collect()
        )
    }

    #[test]
    fn remove_components_for_entity() {
        let mut storage = DeletedComponentsStorage::default();
        storage.add::<u32>(1);
        storage.add::<i64>(1);

        let deleted_component_types = storage.remove(1);

        assert_eq!(
            deleted_component_types,
            [TypeId::of::<u32>(), TypeId::of::<i64>()]
                .iter()
                .copied()
                .collect()
        );
        assert_eq!(storage.remove(1), FxHashSet::default());
    }

    #[test]
    fn reset_entity() {
        let mut storage = DeletedComponentsStorage::default();
        storage.add::<u32>(1);
        storage.add::<i64>(2);

        storage.reset(1);

        assert_eq!(storage.remove(1), FxHashSet::default());
        assert_eq!(storage.remove(2), iter::once(TypeId::of::<i64>()).collect());
    }
}

#[cfg(test)]
mod tests_modified_entities_storage {
    use super::*;

    #[test]
    fn add_entity() {
        let mut storage = ModifiedEntitiesStorage::default();

        storage.add(1);

        assert_iter!(storage.idxs(), [1]);
    }

    #[test]
    fn reset() {
        let mut storage = ModifiedEntitiesStorage::default();
        storage.add(1);
        storage.add(3);

        storage.reset();

        assert_eq!(storage.idxs().next(), None);
    }
}
