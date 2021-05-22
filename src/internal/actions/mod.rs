use crate::internal::actions::data::ActionResult;
use crate::internal::entity_actions::data::AddComponentFn;
use crate::internal::entity_actions::EntityActionFacade;
use crate::internal::group_actions::data::{BuildGroupFn, CreateEntityFn};
use crate::internal::group_actions::GroupActionFacade;
use std::any::Any;
use std::num::NonZeroUsize;

pub(super) mod data;

#[derive(Default)]
pub(crate) struct ActionFacade {
    entity_actions: EntityActionFacade,
    group_actions: GroupActionFacade,
}

impl ActionFacade {
    pub(crate) fn delete_entity(&mut self, entity_idx: usize) {
        self.entity_actions.delete_entity(entity_idx)
    }

    pub(crate) fn add_component(&mut self, entity_idx: usize, add_component_fn: AddComponentFn) {
        self.entity_actions
            .add_component(entity_idx, add_component_fn);
    }

    pub(crate) fn delete_component<C>(&mut self, entity_idx: usize)
    where
        C: Any,
    {
        self.entity_actions.delete_component::<C>(entity_idx);
    }

    pub(crate) fn replace_group(&mut self, group_idx: NonZeroUsize, build_fn: BuildGroupFn) {
        self.group_actions.replace_group(group_idx, build_fn)
    }

    pub(crate) fn delete_group(&mut self, group_idx: NonZeroUsize) {
        self.group_actions.delete_group(group_idx)
    }

    pub(crate) fn create_entity(&mut self, group_idx: NonZeroUsize, create_fn: CreateEntityFn) {
        self.group_actions.create_entity(group_idx, create_fn)
    }

    pub(super) fn reset(&mut self) -> ActionResult {
        let result = ActionResult {
            deleted_entity_idxs: self.entity_actions.deleted_entity_idxs().collect(),
            entity_builders: self.group_actions.entity_builders().collect(),
            deleted_component_types: self.entity_actions.deleted_component_types().collect(),
            component_adders: self.entity_actions.component_adders().collect(),
            deleted_group_idxs: self.group_actions.deleted_group_idxs().collect(),
            replaced_group_builders: self.group_actions.replaced_group_builders().collect(),
        };
        self.group_actions.reset();
        self.entity_actions.reset();
        result
    }
}

#[cfg(test)]
mod action_facade_tests {
    use super::*;
    use std::any::TypeId;
    use std::convert::TryInto;

    #[test]
    fn mark_entity_as_deleted() {
        let mut facade = ActionFacade::default();

        facade.delete_entity(1);

        let result = facade.reset();
        assert_eq!(result.deleted_entity_idxs, [1]);
    }

    #[test]
    fn add_component_to_add() {
        let mut facade = ActionFacade::default();

        facade.add_component(1, Box::new(|_| ()));

        let result = facade.reset();
        assert_eq!(result.component_adders.len(), 1);
    }

    #[test]
    fn mark_component_as_deleted() {
        let mut facade = ActionFacade::default();

        facade.delete_component::<u32>(1);

        let result = facade.reset();
        assert_eq!(result.deleted_component_types, [(1, TypeId::of::<u32>())]);
    }

    #[test]
    fn replace_group() {
        let mut facade = ActionFacade::default();
        let group_idx = 2.try_into().unwrap();

        facade.replace_group(group_idx, Box::new(|_| ()));

        let result = facade.reset();
        assert_eq!(result.replaced_group_builders.len(), 1);
        assert_eq!(result.replaced_group_builders[0].0, group_idx);
    }

    #[test]
    fn delete_group() {
        let mut facade = ActionFacade::default();
        let group_idx = 2.try_into().unwrap();

        facade.delete_group(group_idx);

        let result = facade.reset();
        assert_eq!(result.deleted_group_idxs, [group_idx])
    }

    #[test]
    fn create_entity() {
        let mut facade = ActionFacade::default();
        let group_idx = 2.try_into().unwrap();

        facade.create_entity(group_idx, Box::new(|_| ()));

        let result = facade.reset();
        assert_eq!(result.entity_builders.len(), 1);
    }

    #[test]
    fn reset() {
        let mut facade = ActionFacade::default();
        facade.delete_entity(1);
        facade.delete_group(2.try_into().unwrap());

        facade.reset();

        assert_iter!(facade.entity_actions.deleted_entity_idxs(), []);
        assert_iter!(facade.group_actions.deleted_group_idxs(), []);
    }
}
