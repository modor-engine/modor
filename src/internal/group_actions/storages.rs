use crate::internal::group_actions::data::BuildGroupFn;
use fxhash::FxHashSet;
use std::num::NonZeroUsize;

#[derive(Default)]
pub(super) struct ReplacedGroupsStorage(Vec<Option<BuildGroupFn>>);

impl ReplacedGroupsStorage {
    pub(super) fn register(&mut self, group_idx: NonZeroUsize) {
        let group_pos = group_idx.get() - 1;
        (self.0.len()..=group_pos).for_each(|_| self.0.push(None));
    }

    pub(super) fn add(&mut self, group_idx: NonZeroUsize, build_fn: BuildGroupFn) {
        let group_pos = group_idx.get() - 1;
        self.0[group_pos] = Some(build_fn);
    }

    pub(super) fn remove(&mut self, group_idx: NonZeroUsize) -> Option<BuildGroupFn> {
        let group_pos = group_idx.get() - 1;
        self.0[group_pos].take()
    }
}

#[derive(Default)]
pub(super) struct DeletedGroupsStorage(Vec<bool>);

impl DeletedGroupsStorage {
    pub(super) fn is_marked_as_deleted(&self, group_idx: NonZeroUsize) -> bool {
        let group_pos = group_idx.get() - 1;
        self.0[group_pos]
    }

    pub(super) fn register(&mut self, group_idx: NonZeroUsize) {
        let group_pos = group_idx.get() - 1;
        (self.0.len()..=group_pos).for_each(|_| self.0.push(false));
    }

    pub(super) fn add(&mut self, group_idx: NonZeroUsize) {
        let group_pos = group_idx.get() - 1;
        self.0[group_pos] = true;
    }

    pub(super) fn delete(&mut self, group_idx: NonZeroUsize) {
        let group_pos = group_idx.get() - 1;
        self.0[group_pos] = false;
    }
}

#[derive(Default)]
pub(super) struct ModifiedGroupsStorage(FxHashSet<NonZeroUsize>);

impl ModifiedGroupsStorage {
    pub(super) fn idxs(&self) -> impl Iterator<Item = NonZeroUsize> + '_ {
        self.0.iter().copied()
    }

    pub(super) fn add(&mut self, group_idx: NonZeroUsize) {
        self.0.insert(group_idx);
    }

    pub(super) fn reset(&mut self) {
        self.0.clear();
    }
}

#[cfg(test)]
mod tests_replaced_groups_storage {
    use super::*;
    use std::convert::TryInto;

    #[test]
    fn register_group() {
        let mut storage = ReplacedGroupsStorage::default();

        storage.register(2.try_into().unwrap());

        assert!(storage.remove(1.try_into().unwrap()).is_none());
        assert!(storage.remove(2.try_into().unwrap()).is_none());
        assert_panics!(storage.remove(3.try_into().unwrap()).is_none());
    }

    #[test]
    fn add_group_builder() {
        let mut storage = ReplacedGroupsStorage::default();
        storage.register(2.try_into().unwrap());

        storage.add(2.try_into().unwrap(), Box::new(|_| ()));

        assert!(storage.remove(1.try_into().unwrap()).is_none());
        assert!(storage.remove(2.try_into().unwrap()).is_some());
    }

    #[test]
    fn remove_group_builder() {
        let mut storage = ReplacedGroupsStorage::default();
        storage.register(2.try_into().unwrap());
        storage.add(2.try_into().unwrap(), Box::new(|_| ()));

        let build_fn = storage.remove(2.try_into().unwrap());

        assert!(build_fn.is_some());
        assert!(storage.remove(2.try_into().unwrap()).is_none());
    }
}

#[cfg(test)]
mod tests_deleted_groups_storage {
    use super::*;
    use std::convert::TryInto;

    #[test]
    fn register_group() {
        let mut storage = DeletedGroupsStorage::default();

        storage.register(2.try_into().unwrap());

        assert!(!storage.is_marked_as_deleted(1.try_into().unwrap()));
        assert!(!storage.is_marked_as_deleted(2.try_into().unwrap()));
        assert_panics!(!storage.is_marked_as_deleted(3.try_into().unwrap()));
    }

    #[test]
    fn add_group() {
        let mut storage = DeletedGroupsStorage::default();
        storage.register(2.try_into().unwrap());

        storage.add(2.try_into().unwrap());

        assert!(!storage.is_marked_as_deleted(1.try_into().unwrap()));
        assert!(storage.is_marked_as_deleted(2.try_into().unwrap()));
    }

    #[test]
    fn delete_group() {
        let mut storage = DeletedGroupsStorage::default();
        storage.register(2.try_into().unwrap());
        storage.add(2.try_into().unwrap());

        storage.delete(2.try_into().unwrap());

        assert!(!storage.is_marked_as_deleted(2.try_into().unwrap()));
    }
}

#[cfg(test)]
mod tests_modified_groups_storage {
    use super::*;
    use std::convert::TryInto;

    #[test]
    fn add_group() {
        let mut storage = ModifiedGroupsStorage::default();

        storage.add(2.try_into().unwrap());

        assert_iter!(storage.idxs(), [2.try_into().unwrap()]);
    }

    #[test]
    fn reset() {
        let mut storage = ModifiedGroupsStorage::default();
        storage.add(2.try_into().unwrap());
        storage.add(4.try_into().unwrap());

        storage.reset();

        assert_eq!(storage.idxs().next(), None);
    }
}
