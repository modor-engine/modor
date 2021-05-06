use crate::internal::group_actions::data::BuildGroupFn;
use fxhash::FxHashSet;
use std::num::NonZeroUsize;

#[derive(Default)]
pub(super) struct ReplacedGroupsStorage(Vec<Option<BuildGroupFn>>);

impl ReplacedGroupsStorage {
    pub(super) fn register_group(&mut self, group_idx: NonZeroUsize) {
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
