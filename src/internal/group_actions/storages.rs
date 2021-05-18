use crate::internal::group_actions::data::{BuildGroupFn, CreateEntityFn, GroupBuilderState};
use fxhash::FxHashSet;
use std::mem;
use std::num::NonZeroUsize;

#[derive(Default)]
pub(super) struct ReplacedGroupsStorage(Vec<GroupBuilderState>);

impl ReplacedGroupsStorage {
    pub(super) fn is_marked_as_replaced(&self, group_idx: NonZeroUsize) -> bool {
        let group_pos = group_idx.get() - 1;
        self.0.get(group_pos).map_or(false, |b| {
            matches!(b, GroupBuilderState::Some(_) | GroupBuilderState::Removed)
        })
    }

    pub(super) fn add(&mut self, group_idx: NonZeroUsize, build_fn: BuildGroupFn) {
        let group_pos = group_idx.get() - 1;
        (self.0.len()..=group_pos).for_each(|_| self.0.push(GroupBuilderState::None));
        self.0[group_pos] = GroupBuilderState::Some(build_fn);
    }

    pub(super) fn remove(&mut self, group_idx: NonZeroUsize) -> Option<BuildGroupFn> {
        let group_pos = group_idx.get() - 1;
        self.0.get_mut(group_pos).and_then(|b| match b {
            GroupBuilderState::Some(_) => {
                let old = mem::replace(b, GroupBuilderState::Removed);
                if let GroupBuilderState::Some(build_fn) = old {
                    Some(build_fn)
                } else {
                    None
                }
            }
            GroupBuilderState::Removed | GroupBuilderState::None => None,
        })
    }

    pub(super) fn reset(&mut self, group_idx: NonZeroUsize) {
        let group_pos = group_idx.get() - 1;
        if let Some(builder) = self.0.get_mut(group_pos) {
            *builder = GroupBuilderState::None;
        }
    }
}

#[derive(Default)]
pub(super) struct DeletedGroupsStorage(Vec<bool>);

impl DeletedGroupsStorage {
    pub(super) fn is_marked_as_deleted(&self, group_idx: NonZeroUsize) -> bool {
        let group_pos = group_idx.get() - 1;
        self.0.get(group_pos).copied().unwrap_or(false)
    }

    pub(super) fn add(&mut self, group_idx: NonZeroUsize) {
        let group_pos = group_idx.get() - 1;
        (self.0.len()..=group_pos).for_each(|_| self.0.push(false));
        self.0[group_pos] = true;
    }

    pub(super) fn delete(&mut self, group_idx: NonZeroUsize) {
        let group_pos = group_idx.get() - 1;
        if let Some(deleted) = self.0.get_mut(group_pos) {
            *deleted = false;
        }
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

#[derive(Default)]
pub(super) struct CreatedEntitiesStorage(Vec<Vec<CreateEntityFn>>);

impl CreatedEntitiesStorage {
    pub(super) fn add(&mut self, group_idx: NonZeroUsize, create_fn: CreateEntityFn) {
        let group_pos = group_idx.get() - 1;
        (self.0.len()..=group_pos).for_each(|_| self.0.push(Vec::new()));
        self.0[group_pos].push(create_fn);
    }

    pub(super) fn remove(&mut self, group_idx: NonZeroUsize) -> Vec<CreateEntityFn> {
        let group_pos = group_idx.get() - 1;
        self.0.get_mut(group_pos).map_or_else(Vec::new, mem::take)
    }
}

#[cfg(test)]
mod tests_replaced_groups_storage {
    use super::*;
    use std::convert::TryInto;

    #[test]
    fn add_group_builder() {
        let mut storage = ReplacedGroupsStorage::default();

        storage.add(2.try_into().unwrap(), Box::new(|_| ()));

        assert!(!storage.is_marked_as_replaced(1.try_into().unwrap()));
        assert!(storage.is_marked_as_replaced(2.try_into().unwrap()));
        assert!(!storage.is_marked_as_replaced(3.try_into().unwrap()));
    }

    #[test]
    fn remove_missing_group_builder() {
        let mut storage = ReplacedGroupsStorage::default();

        let build_fn = storage.remove(2.try_into().unwrap());

        assert!(build_fn.is_none());
    }

    #[test]
    fn remove_existing_group_builder() {
        let mut storage = ReplacedGroupsStorage::default();
        storage.add(2.try_into().unwrap(), Box::new(|_| ()));

        let build_fn = storage.remove(2.try_into().unwrap());

        assert!(build_fn.is_some());
        assert!(storage.is_marked_as_replaced(2.try_into().unwrap()));
        assert!(storage.remove(2.try_into().unwrap()).is_none());
    }

    #[test]
    fn reset_existing_group_builder() {
        let mut storage = ReplacedGroupsStorage::default();
        storage.add(2.try_into().unwrap(), Box::new(|_| ()));

        storage.reset(2.try_into().unwrap());

        assert!(!storage.is_marked_as_replaced(2.try_into().unwrap()));
    }

    #[test]
    fn reset_missing_group_builder() {
        let mut storage = ReplacedGroupsStorage::default();

        storage.reset(2.try_into().unwrap());

        assert!(!storage.is_marked_as_replaced(2.try_into().unwrap()));
    }
}

#[cfg(test)]
mod tests_deleted_groups_storage {
    use super::*;
    use std::convert::TryInto;

    #[test]
    fn add_group() {
        let mut storage = DeletedGroupsStorage::default();

        storage.add(2.try_into().unwrap());

        assert!(!storage.is_marked_as_deleted(1.try_into().unwrap()));
        assert!(storage.is_marked_as_deleted(2.try_into().unwrap()));
        assert!(!storage.is_marked_as_deleted(3.try_into().unwrap()));
    }

    #[test]
    fn delete_missing_group() {
        let mut storage = DeletedGroupsStorage::default();

        storage.delete(2.try_into().unwrap());

        assert!(!storage.is_marked_as_deleted(2.try_into().unwrap()));
    }

    #[test]
    fn delete_existing_group() {
        let mut storage = DeletedGroupsStorage::default();
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

#[cfg(test)]
mod tests_created_entities_storage {
    use super::*;
    use std::convert::TryInto;

    #[test]
    fn add_entity_builders() {
        let mut storage = CreatedEntitiesStorage::default();

        storage.add(2.try_into().unwrap(), Box::new(|_| ()));
        storage.add(2.try_into().unwrap(), Box::new(|_| ()));
        storage.add(2.try_into().unwrap(), Box::new(|_| ()));

        assert_eq!(storage.remove(1.try_into().unwrap()).len(), 0);
        assert_eq!(storage.remove(2.try_into().unwrap()).len(), 3);
        assert_eq!(storage.remove(3.try_into().unwrap()).len(), 0);
    }

    #[test]
    fn remove_missing_entity_builders() {
        let mut storage = CreatedEntitiesStorage::default();

        let builders = storage.remove(2.try_into().unwrap());

        assert_eq!(builders.len(), 0);
    }

    #[test]
    fn remove_existing_entity_builders() {
        let mut storage = CreatedEntitiesStorage::default();
        storage.add(2.try_into().unwrap(), Box::new(|_| ()));
        storage.add(2.try_into().unwrap(), Box::new(|_| ()));
        storage.add(2.try_into().unwrap(), Box::new(|_| ()));

        let builders = storage.remove(2.try_into().unwrap());

        assert_eq!(builders.len(), 3);
        assert_eq!(storage.remove(2.try_into().unwrap()).len(), 0);
    }
}
