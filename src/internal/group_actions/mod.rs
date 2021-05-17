use crate::internal::group_actions::data::{BuildGroupFn, CreateEntityFn};
use crate::internal::group_actions::storages::{
    CreatedEntitiesStorage, DeletedGroupsStorage, ModifiedGroupsStorage, ReplacedGroupsStorage,
};
use std::num::NonZeroUsize;

pub(super) mod data;
mod storages;

#[derive(Default)]
pub(super) struct GroupActionFacade {
    replaced_groups: ReplacedGroupsStorage,
    deleted_groups: DeletedGroupsStorage,
    modified_groups: ModifiedGroupsStorage,
    created_entities: CreatedEntitiesStorage,
}

impl GroupActionFacade {
    pub(super) fn replaced_group_builders(
        &mut self,
    ) -> impl Iterator<Item = (NonZeroUsize, BuildGroupFn)> + '_ {
        let replaced_groups = &mut self.replaced_groups;
        let deleted_groups = &self.deleted_groups;
        self.modified_groups
            .idxs()
            .filter(move |&g| !deleted_groups.is_marked_as_deleted(g))
            .filter_map(move |g| replaced_groups.remove(g).map(|f| (g, f)))
    }

    pub(super) fn mark_group_as_replaced(
        &mut self,
        group_idx: NonZeroUsize,
        build_fn: BuildGroupFn,
    ) {
        self.replaced_groups.add(group_idx, Box::new(build_fn));
        self.modified_groups.add(group_idx)
    }

    pub(super) fn deleted_group_idxs(&self) -> impl Iterator<Item = NonZeroUsize> + '_ {
        self.modified_groups
            .idxs()
            .filter(move |&g| self.deleted_groups.is_marked_as_deleted(g))
    }

    pub(super) fn mark_group_as_deleted(&mut self, group_idx: NonZeroUsize) {
        self.deleted_groups.add(group_idx);
        self.modified_groups.add(group_idx);
    }

    pub(super) fn entity_builders(&mut self) -> impl Iterator<Item = CreateEntityFn> + '_ {
        let replaced_groups = &mut self.replaced_groups;
        let deleted_groups = &self.deleted_groups;
        let created_entities = &mut self.created_entities;
        self.modified_groups
            .idxs()
            .filter(move |&g| !deleted_groups.is_marked_as_deleted(g))
            .filter(move |&g| !replaced_groups.is_marked_as_replaced(g))
            .flat_map(move |g| created_entities.remove(g).into_iter())
    }

    pub(super) fn add_entity_to_create(
        &mut self,
        group_idx: NonZeroUsize,
        create_fn: CreateEntityFn,
    ) {
        self.created_entities.add(group_idx, create_fn);
        self.modified_groups.add(group_idx);
    }

    pub(super) fn reset(&mut self) {
        for group_idx in self.modified_groups.idxs() {
            self.replaced_groups.reset(group_idx);
            self.deleted_groups.delete(group_idx);
            self.created_entities.remove(group_idx);
        }
        self.modified_groups.reset();
    }
}

#[cfg(test)]
mod tests_group_action_facade {
    use super::*;
    use std::convert::TryInto;

    #[test]
    fn mark_group_as_replaced() {
        let mut facade = GroupActionFacade::default();
        let group_idx = 2.try_into().unwrap();

        facade.mark_group_as_replaced(group_idx, Box::new(|_| ()));

        assert!(facade.replaced_groups.remove(group_idx).is_some());
        assert_iter!(facade.modified_groups.idxs(), [group_idx]);
    }

    #[test]
    fn mark_group_as_deleted() {
        let mut facade = GroupActionFacade::default();
        let group_idx = 2.try_into().unwrap();

        facade.mark_group_as_deleted(group_idx);

        assert!(facade.deleted_groups.is_marked_as_deleted(group_idx));
        assert_iter!(facade.modified_groups.idxs(), [group_idx]);
    }

    #[test]
    fn add_entity_to_create() {
        let mut facade = GroupActionFacade::default();
        let group_idx = 2.try_into().unwrap();

        facade.add_entity_to_create(group_idx, Box::new(|_| ()));

        assert_eq!(facade.created_entities.remove(group_idx).len(), 1);
        assert_iter!(facade.modified_groups.idxs(), [group_idx]);
    }

    #[test]
    fn retrieve_replaced_group_builders() {
        let mut facade = GroupActionFacade::default();
        let group1_idx = 1.try_into().unwrap();
        let group2_idx = 2.try_into().unwrap();
        facade.mark_group_as_deleted(group1_idx);
        facade.mark_group_as_replaced(group1_idx, Box::new(|_| ()));
        facade.mark_group_as_replaced(group2_idx, Box::new(|_| ()));

        let replaced_group_idxs: Vec<_> = facade.replaced_group_builders().collect();

        assert_eq!(replaced_group_idxs.len(), 1);
        assert_eq!(replaced_group_idxs[0].0, group2_idx);
        assert!(facade.replaced_groups.remove(group2_idx).is_none());
    }

    #[test]
    fn retrieve_deleted_group_idxs() {
        let mut facade = GroupActionFacade::default();
        let group1_idx = 1.try_into().unwrap();
        let group2_idx = 2.try_into().unwrap();
        facade.mark_group_as_replaced(group1_idx, Box::new(|_| ()));
        facade.mark_group_as_deleted(group2_idx);
        facade.mark_group_as_replaced(group2_idx, Box::new(|_| ()));

        let deleted_group_idxs = facade.deleted_group_idxs();

        assert_iter!(deleted_group_idxs, [group2_idx]);
    }

    #[test]
    fn retrieve_entity_builders_when_no_deleted_and_replaced_groups() {
        let mut facade = GroupActionFacade::default();
        let group_idx = 2.try_into().unwrap();
        facade.add_entity_to_create(group_idx, Box::new(|_| ()));

        let entity_builders: Vec<_> = facade.entity_builders().collect();

        assert_eq!(entity_builders.len(), 1);
    }

    #[test]
    fn retrieve_entity_builders_when_deleted_and_replaced_groups() {
        let mut facade = GroupActionFacade::default();
        let group1_idx = 1.try_into().unwrap();
        let group2_idx = 2.try_into().unwrap();
        let group3_idx = 3.try_into().unwrap();
        facade.mark_group_as_replaced(group1_idx, Box::new(|_| ()));
        facade.add_entity_to_create(group1_idx, Box::new(|_| ()));
        facade.mark_group_as_deleted(group2_idx);
        facade.add_entity_to_create(group2_idx, Box::new(|_| ()));
        facade.add_entity_to_create(group3_idx, Box::new(|_| ()));

        let entity_builders: Vec<_> = facade.entity_builders().collect();

        assert_eq!(entity_builders.len(), 1);
    }

    #[test]
    fn reset() {
        let mut facade = GroupActionFacade::default();
        let group1_idx = 1.try_into().unwrap();
        let group2_idx = 2.try_into().unwrap();
        facade.mark_group_as_replaced(group1_idx, Box::new(|_| ()));
        facade.mark_group_as_replaced(group2_idx, Box::new(|_| ()));
        facade.mark_group_as_deleted(group1_idx);
        facade.mark_group_as_deleted(group2_idx);
        facade.add_entity_to_create(group1_idx, Box::new(|_| ()));
        facade.add_entity_to_create(group2_idx, Box::new(|_| ()));

        facade.reset();

        assert!(!facade.replaced_groups.is_marked_as_replaced(group1_idx));
        assert!(!facade.replaced_groups.is_marked_as_replaced(group2_idx));
        assert!(!facade.deleted_groups.is_marked_as_deleted(group1_idx));
        assert!(!facade.deleted_groups.is_marked_as_deleted(group2_idx));
        assert_eq!(facade.created_entities.remove(group1_idx).len(), 0);
        assert_eq!(facade.created_entities.remove(group2_idx).len(), 0);
        assert!(facade.modified_groups.idxs().next().is_none())
    }
}
