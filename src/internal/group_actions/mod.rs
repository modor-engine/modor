use crate::internal::group_actions::data::BuildGroupFn;
use crate::internal::group_actions::storages::{
    DeletedGroupsStorage, ModifiedGroupsStorage, ReplacedGroupsStorage,
};
use crate::GroupBuilder;
use std::num::NonZeroUsize;

pub(super) mod data;
mod storages;

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
            .filter(move |&i| !deleted_groups.is_marked_as_deleted(i))
            .filter_map(move |i| replaced_groups.remove(i).map(|f| (i, f)))
    }

    pub(crate) fn mark_group_as_replaced<F>(&mut self, group_idx: NonZeroUsize, build_fn: F)
    where
        F: FnOnce(&mut GroupBuilder<'_>) + Sync + Send + 'static,
    {
        self.replaced_groups.add(group_idx, Box::new(build_fn));
        self.modified_groups.add(group_idx)
    }

    pub(super) fn deleted_group_idxs(&self) -> impl Iterator<Item = NonZeroUsize> + '_ {
        self.modified_groups
            .idxs()
            .filter(move |&i| self.deleted_groups.is_marked_as_deleted(i))
    }

    pub(crate) fn mark_group_as_deleted(&mut self, group_idx: NonZeroUsize) {
        self.deleted_groups.add(group_idx);
        self.modified_groups.add(group_idx)
    }

    pub(super) fn register_group(&mut self, group_idx: NonZeroUsize) {
        self.replaced_groups.register(group_idx);
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

#[cfg(test)]
mod tests_group_action_facade {
    use super::*;
    use std::convert::TryInto;

    #[test]
    fn register_group() {
        let mut facade = GroupActionFacade::default();
        let group_idx = 2.try_into().unwrap();

        facade.register_group(group_idx);

        assert!(facade.replaced_groups.remove(group_idx).is_none());
        assert!(!facade.deleted_groups.is_marked_as_deleted(group_idx));
    }

    #[test]
    fn mark_group_as_replaced() {
        let mut facade = GroupActionFacade::default();
        let group_idx = 2.try_into().unwrap();
        facade.register_group(group_idx);

        facade.mark_group_as_replaced(group_idx, |_| ());

        assert!(facade.replaced_groups.remove(group_idx).is_some());
        assert_iter!(facade.modified_groups.idxs(), [group_idx]);
    }

    #[test]
    fn mark_group_as_deleted() {
        let mut facade = GroupActionFacade::default();
        let group_idx = 2.try_into().unwrap();
        facade.register_group(group_idx);

        facade.mark_group_as_deleted(group_idx);

        assert!(facade.deleted_groups.is_marked_as_deleted(group_idx));
        assert_iter!(facade.modified_groups.idxs(), [group_idx]);
    }

    #[test]
    fn retrieve_replaced_group_idxs() {
        let mut facade = GroupActionFacade::default();
        let group1_idx = 1.try_into().unwrap();
        let group2_idx = 2.try_into().unwrap();
        facade.register_group(group1_idx);
        facade.register_group(group2_idx);
        facade.mark_group_as_deleted(group1_idx);
        facade.mark_group_as_replaced(group1_idx, |_| ());
        facade.mark_group_as_replaced(group2_idx, |_| ());

        let replaced_group_idxs: Vec<_> = facade.replaced_group_idxs().collect();

        assert_eq!(replaced_group_idxs.len(), 1);
        assert_eq!(replaced_group_idxs[0].0, group2_idx);
        assert!(facade.replaced_groups.remove(group2_idx).is_none());
    }

    #[test]
    fn retrieve_deleted_group_idxs() {
        let mut facade = GroupActionFacade::default();
        let group1_idx = 1.try_into().unwrap();
        let group2_idx = 2.try_into().unwrap();
        facade.register_group(group1_idx);
        facade.register_group(group2_idx);
        facade.mark_group_as_replaced(group1_idx, |_| ());
        facade.mark_group_as_deleted(group2_idx);
        facade.mark_group_as_replaced(group2_idx, |_| ());

        let deleted_group_idxs = facade.deleted_group_idxs();

        assert_iter!(deleted_group_idxs, [group2_idx]);
    }

    #[test]
    fn reset() {
        let mut facade = GroupActionFacade::default();
        let group1_idx = 1.try_into().unwrap();
        let group2_idx = 2.try_into().unwrap();
        facade.register_group(group1_idx);
        facade.register_group(group2_idx);
        facade.mark_group_as_replaced(group1_idx, |_| ());
        facade.mark_group_as_replaced(group2_idx, |_| ());
        facade.mark_group_as_deleted(group1_idx);
        facade.mark_group_as_deleted(group2_idx);

        facade.reset();

        assert!(facade.replaced_groups.remove(group1_idx).is_none());
        assert!(facade.replaced_groups.remove(group2_idx).is_none());
        assert!(!facade.deleted_groups.is_marked_as_deleted(group1_idx));
        assert!(!facade.deleted_groups.is_marked_as_deleted(group2_idx));
        assert!(facade.modified_groups.idxs().next().is_none())
    }
}
