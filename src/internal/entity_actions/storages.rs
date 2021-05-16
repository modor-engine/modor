use fxhash::FxHashSet;

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
