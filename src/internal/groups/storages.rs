use fxhash::FxHashSet;
use std::{convert::TryInto, num::NonZeroUsize};

pub(super) struct GroupStorage {
    next_idx: NonZeroUsize,
    deleted_idxs: Vec<NonZeroUsize>,
}

impl Default for GroupStorage {
    fn default() -> Self {
        Self {
            next_idx: 1.try_into().unwrap(),
            deleted_idxs: Vec::new(),
        }
    }
}

impl GroupStorage {
    pub(super) fn create(&mut self) -> NonZeroUsize {
        self.deleted_idxs.pop().unwrap_or_else(|| {
            let idx = self.next_idx;
            self.next_idx = (self.next_idx.get() + 1).try_into().unwrap();
            idx
        })
    }

    pub(super) fn delete(&mut self, group_idx: NonZeroUsize) {
        if group_idx.get() >= self.next_idx.get() {
            panic!("internal error: cannot delete nonexisting group");
        }
        self.deleted_idxs.push(group_idx);
    }
}

#[derive(Default)]
pub(super) struct EntityStorage(Vec<FxHashSet<usize>>);

impl EntityStorage {
    pub(super) fn idxs(&self, group_idx: NonZeroUsize) -> impl Iterator<Item = usize> + '_ {
        let group_pos = group_idx.get() - 1;
        self.0[group_pos].iter().copied()
    }

    pub(super) fn add(&mut self, entity_idx: usize, group_idx: NonZeroUsize) {
        let group_pos = group_idx.get() - 1;
        (self.0.len()..=group_pos).for_each(|_| self.0.push(FxHashSet::default()));
        self.0[group_pos].insert(entity_idx);
    }

    pub(super) fn delete(&mut self, entity_idx: usize, group_idx: NonZeroUsize) {
        let group_pos = group_idx.get() - 1;
        self.0[group_pos].remove(&entity_idx);
    }

    pub(super) fn delete_group(&mut self, group_idx: NonZeroUsize) {
        let group_pos = group_idx.get() - 1;
        self.0[group_pos] = FxHashSet::default();
    }
}

#[derive(Default)]
pub(super) struct EntityGroupStorage(Vec<Option<NonZeroUsize>>);

impl EntityGroupStorage {
    pub(super) fn idx(&self, entity_idx: usize) -> NonZeroUsize {
        self.0[entity_idx].unwrap()
    }

    pub(super) fn set(&mut self, entity_idx: usize, group_idx: NonZeroUsize) {
        (self.0.len()..=entity_idx).for_each(|_| self.0.push(None));
        self.0[entity_idx] = Some(group_idx);
    }

    pub(super) fn delete(&mut self, entity_idx: usize) {
        self.0[entity_idx] = None;
    }
}

// coverage=off
#[cfg(test)]
mod tests_group_storage {
    use super::*;

    #[test]
    fn create_first_group() {
        let mut storage = GroupStorage::default();

        let group_idx = storage.create();

        assert_eq!(group_idx, 1.try_into().unwrap());
    }

    #[test]
    fn create_other_group() {
        let mut storage = GroupStorage::default();
        storage.create();

        let group_idx = storage.create();

        assert_eq!(group_idx, 2.try_into().unwrap());
    }

    #[test]
    #[should_panic]
    fn delete_nonexisting_group() {
        let mut storage = GroupStorage::default();
        storage.create();
        storage.create();

        storage.delete(3.try_into().unwrap());
    }

    #[test]
    fn delete_exiting_group() {
        let mut storage = GroupStorage::default();
        storage.create();
        storage.create();
        storage.create();

        storage.delete(2.try_into().unwrap());

        assert_eq!(storage.create(), 2.try_into().unwrap());
        assert_eq!(storage.create(), 4.try_into().unwrap());
    }
}

#[cfg(test)]
mod tests_entity_storage {
    use super::*;

    #[test]
    fn add_entities() {
        let mut storage = EntityStorage::default();

        storage.add(1, 2.try_into().unwrap());
        storage.add(4, 2.try_into().unwrap());
        storage.add(5, 3.try_into().unwrap());

        assert_eq!(storage.idxs(1.try_into().unwrap()).next(), None);
        assert_iter!(storage.idxs(2.try_into().unwrap()), [4, 1]);
        assert_iter!(storage.idxs(3.try_into().unwrap()), [5]);
        assert_panics!(storage.idxs(4.try_into().unwrap()));
    }

    #[test]
    #[should_panic]
    fn test_entity_from_nonexisting_group() {
        let mut storage = EntityStorage::default();

        storage.delete(2, 1.try_into().unwrap());
    }

    #[test]
    fn delete_entity_from_existing_group() {
        let mut storage = EntityStorage::default();
        storage.add(1, 2.try_into().unwrap());
        storage.add(4, 2.try_into().unwrap());

        storage.delete(4, 2.try_into().unwrap());

        assert_iter!(storage.idxs(2.try_into().unwrap()), [1]);
    }

    #[test]
    #[should_panic]
    fn delete_nonexiting_group() {
        let mut storage = EntityStorage::default();

        storage.delete_group(1.try_into().unwrap());
    }

    #[test]
    fn delete_existing_group() {
        let mut storage = EntityStorage::default();
        storage.add(4, 2.try_into().unwrap());
        storage.add(5, 3.try_into().unwrap());

        storage.delete_group(2.try_into().unwrap());

        assert_eq!(storage.idxs(2.try_into().unwrap()).next(), None);
        assert_iter!(storage.idxs(3.try_into().unwrap()), [5]);
    }
}

#[cfg(test)]
mod tests_entity_group_storage {
    use super::*;

    #[test]
    fn set_entity_groups() {
        let mut storage = EntityGroupStorage::default();

        storage.set(2, 5.try_into().unwrap());
        storage.set(1, 4.try_into().unwrap());

        assert_panics!(storage.idx(0));
        assert_eq!(storage.idx(1), 4.try_into().unwrap());
        assert_eq!(storage.idx(2), 5.try_into().unwrap());
        assert_panics!(storage.idx(3));
    }

    #[test]
    #[should_panic]
    fn delete_nonexisting_entity() {
        let mut storage = EntityGroupStorage::default();

        storage.delete(0);
    }

    #[test]
    fn delete_existing_entity() {
        let mut storage = EntityGroupStorage::default();
        storage.set(2, 5.try_into().unwrap());
        storage.set(1, 4.try_into().unwrap());

        storage.delete(1);

        assert_panics!(storage.idx(0));
        assert_panics!(storage.idx(1));
        assert_eq!(storage.idx(2), 5.try_into().unwrap());
        assert_panics!(storage.idx(3));
    }
}
