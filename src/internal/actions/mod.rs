use crate::internal::actions::data::ActionResult;
use crate::internal::entity_actions::EntityActionFacade;
use crate::internal::group_actions::data::{BuildGroupFn, CreateEntityFn};
use crate::internal::group_actions::GroupActionFacade;
use std::num::NonZeroUsize;

pub(super) mod data;

#[derive(Default)]
pub(crate) struct ActionFacade {
    group_actions: GroupActionFacade,
    entity_actions: EntityActionFacade,
}

impl ActionFacade {
    pub(crate) fn mark_group_as_replaced(
        &mut self,
        group_idx: NonZeroUsize,
        build_fn: BuildGroupFn,
    ) {
        self.group_actions
            .mark_group_as_replaced(group_idx, build_fn)
    }

    pub(crate) fn mark_group_as_deleted(&mut self, group_idx: NonZeroUsize) {
        self.group_actions.mark_group_as_deleted(group_idx)
    }

    pub(crate) fn add_entity_to_create(
        &mut self,
        group_idx: NonZeroUsize,
        create_fn: CreateEntityFn,
    ) {
        self.group_actions
            .add_entity_to_create(group_idx, create_fn)
    }

    pub(crate) fn mark_entity_as_deleted(&mut self, entity_idx: usize) {
        self.entity_actions.mark_entity_as_deleted(entity_idx)
    }

    pub(super) fn reset(&mut self) -> ActionResult {
        let result = ActionResult {
            deleted_group_idxs: self.group_actions.deleted_group_idxs().collect(),
            replaced_group_builders: self.group_actions.replaced_group_builders().collect(),
            entity_builders: self.group_actions.entity_builders().collect(),
            deleted_entity_idxs: self.entity_actions.deleted_entity_idxs().collect(),
        };
        self.group_actions.reset();
        self.entity_actions.reset();
        result
    }
}

#[cfg(test)]
mod tests_action_facade {
    use super::*;
    use std::convert::TryInto;

    #[test]
    fn mark_group_as_replaced() {
        let mut facade = ActionFacade::default();
        let group_idx = 2.try_into().unwrap();

        facade.mark_group_as_replaced(group_idx, Box::new(|_| ()));

        let result = facade.reset();
        assert_eq!(result.replaced_group_builders.len(), 1);
        assert_eq!(result.replaced_group_builders[0].0, group_idx);
    }

    #[test]
    fn mark_group_as_deleted() {
        let mut facade = ActionFacade::default();
        let group_idx = 2.try_into().unwrap();

        facade.mark_group_as_deleted(group_idx);

        let result = facade.reset();
        assert_eq!(result.deleted_group_idxs, [group_idx])
    }

    #[test]
    fn add_entity_to_create() {
        let mut facade = ActionFacade::default();
        let group_idx = 2.try_into().unwrap();

        facade.add_entity_to_create(group_idx, Box::new(|_| ()));

        let result = facade.reset();
        assert_eq!(result.entity_builders.len(), 1);
    }

    #[test]
    fn mark_entity_as_deleted() {
        let mut facade = ActionFacade::default();

        facade.mark_entity_as_deleted(1);

        let result = facade.reset();
        assert_eq!(result.deleted_entity_idxs, [1]);
    }
}
