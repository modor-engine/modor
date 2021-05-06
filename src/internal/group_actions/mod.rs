use crate::internal::group_actions::data::BuildGroupFn;
use crate::internal::group_actions::storages::{
    DeletedGroupsStorage, ModifiedGroupsStorage, ReplacedGroupsStorage,
};
use crate::GroupBuilder;
use std::num::NonZeroUsize;

pub(super) mod data;
mod storages;

// TODO: test facade and storages

#[derive(Default)]
pub(crate) struct GroupActionFacade {
    replaced_groups: ReplacedGroupsStorage,
    deleted_groups: DeletedGroupsStorage,
    modified_groups: ModifiedGroupsStorage,
}

impl GroupActionFacade {
    pub(super) fn replaced_group_idxs(
        &mut self,
    ) -> impl Iterator<Item = (NonZeroUsize, BuildGroupFn)> + '_ {
        let replaced_groups = &mut self.replaced_groups;
        let deleted_groups = &self.deleted_groups;
        self.modified_groups
            .idxs()
            .filter(move |&i| deleted_groups.is_marked_as_deleted(i))
            .filter_map(move |i| replaced_groups.remove(i).map(|f| (i, f)))
    }

    pub(crate) fn mark_group_as_replaced<F>(&mut self, group_idx: NonZeroUsize, build_fn: F)
    where
        F: FnOnce(&mut GroupBuilder<'_>) + Sync + Send + 'static,
    {
        self.replaced_groups.add(group_idx, Box::new(build_fn));
        self.modified_groups.add(group_idx)
    }

    pub(crate) fn mark_group_as_deleted(&mut self, group_idx: NonZeroUsize) {
        self.deleted_groups.add(group_idx);
        self.modified_groups.add(group_idx)
    }

    pub(super) fn deleted_group_idxs(&self) -> impl Iterator<Item = NonZeroUsize> + '_ {
        self.modified_groups
            .idxs()
            .filter(move |&i| self.deleted_groups.is_marked_as_deleted(i))
    }

    pub(super) fn register_group(&mut self, group_idx: NonZeroUsize) {
        self.replaced_groups.register_group(group_idx);
        self.deleted_groups.register(group_idx);
    }

    pub(super) fn reset(&mut self) {
        for group_idx in self.modified_groups.idxs() {
            self.replaced_groups.remove(group_idx);
            self.deleted_groups.delete(group_idx);
        }
        self.modified_groups.reset();
    }
}
