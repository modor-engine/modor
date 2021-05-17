use crate::internal::entity_actions::data::AddComponentFn;
use crate::internal::entity_actions::storages::{
    AddedComponentsStorage, DeletedComponentsStorage, DeletedEntitiesStorage,
    ModifiedEntitiesStorage,
};
use std::any::{Any, TypeId};

pub(super) mod data;
mod storages;

#[derive(Default)]
pub(super) struct EntityActionFacade {
    deleted_entities: DeletedEntitiesStorage,
    added_components: AddedComponentsStorage,
    deleted_components: DeletedComponentsStorage,
    modified_entities: ModifiedEntitiesStorage,
}

impl EntityActionFacade {
    pub(super) fn deleted_entity_idxs(&self) -> impl Iterator<Item = usize> + '_ {
        let deleted_entities = &self.deleted_entities;
        self.modified_entities
            .idxs()
            .filter(move |&i| deleted_entities.is_marked_as_deleted(i))
    }

    // TODO: rename in "delete_entity" (and rename other methods)
    pub(super) fn mark_entity_as_deleted(&mut self, entity_idx: usize) {
        self.deleted_entities.add(entity_idx);
        self.modified_entities.add(entity_idx);
    }

    pub(super) fn component_adders(&mut self) -> impl Iterator<Item = AddComponentFn> + '_ {
        let deleted_entities = &self.deleted_entities;
        let added_components = &mut self.added_components;
        self.modified_entities
            .idxs()
            .filter(move |&i| !deleted_entities.is_marked_as_deleted(i))
            .flat_map(move |i| added_components.remove(i).into_iter())
    }

    pub(super) fn add_component_to_add(
        &mut self,
        entity_idx: usize,
        add_component_fn: AddComponentFn,
    ) {
        self.added_components.add(entity_idx, add_component_fn);
        self.modified_entities.add(entity_idx);
    }

    pub(super) fn deleted_component_types(&mut self) -> impl Iterator<Item = (usize, TypeId)> + '_ {
        let deleted_entities = &self.deleted_entities;
        let deleted_components = &mut self.deleted_components;
        self.modified_entities
            .idxs()
            .filter(move |&i| !deleted_entities.is_marked_as_deleted(i))
            .flat_map(move |i| {
                deleted_components
                    .remove(i)
                    .into_iter()
                    .map(move |t| (i, t))
            })
    }

    pub(super) fn mark_component_as_deleted<C>(&mut self, entity_idx: usize)
    where
        C: Any,
    {
        self.deleted_components.add::<C>(entity_idx);
        self.modified_entities.add(entity_idx);
    }

    pub(super) fn reset(&mut self) {
        for entity_idx in self.modified_entities.idxs() {
            self.deleted_entities.delete(entity_idx);
            self.added_components.reset(entity_idx);
            self.deleted_components.reset(entity_idx);
        }
        self.modified_entities.reset();
    }
}

#[cfg(test)]
mod tests_entity_action_facade {
    use super::*;
    use std::iter;

    #[test]
    fn mark_entity_as_deleted() {
        let mut facade = EntityActionFacade::default();

        facade.mark_entity_as_deleted(1);

        assert!(facade.deleted_entities.is_marked_as_deleted(1));
        assert_iter!(facade.modified_entities.idxs(), [1]);
    }

    #[test]
    fn add_component_to_add() {
        let mut facade = EntityActionFacade::default();

        facade.add_component_to_add(1, Box::new(|_| ()));

        assert_eq!(facade.added_components.remove(1).len(), 1);
        assert_iter!(facade.modified_entities.idxs(), [1]);
    }

    #[test]
    fn mark_component_as_deleted() {
        let mut facade = EntityActionFacade::default();

        facade.mark_component_as_deleted::<u32>(1);

        assert_eq!(
            facade.deleted_components.remove(1),
            iter::once(TypeId::of::<u32>()).collect()
        );
        assert_iter!(facade.modified_entities.idxs(), [1]);
    }

    #[test]
    fn retrieve_deleted_entities() {
        let mut facade = EntityActionFacade::default();
        facade.mark_entity_as_deleted(1);
        facade.mark_entity_as_deleted(3);

        let deleted_entity_idxs = facade.deleted_entity_idxs();

        assert_iter!(deleted_entity_idxs, [1, 3]);
    }

    #[test]
    fn retrieve_component_adders() {
        let mut facade = EntityActionFacade::default();
        facade.add_component_to_add(1, Box::new(|_| ()));
        facade.add_component_to_add(1, Box::new(|_| ()));
        facade.mark_entity_as_deleted(2);
        facade.add_component_to_add(2, Box::new(|_| ()));

        let component_adders: Vec<_> = facade.component_adders().collect();

        assert_eq!(component_adders.len(), 2);
    }

    #[test]
    fn retrieve_deleted_component_types() {
        let mut facade = EntityActionFacade::default();
        facade.mark_component_as_deleted::<u32>(1);
        facade.mark_component_as_deleted::<i64>(2);
        facade.mark_entity_as_deleted(2);

        let deleted_component_types = facade.deleted_component_types();

        assert_iter!(deleted_component_types, [(1, TypeId::of::<u32>())]);
    }

    #[test]
    fn reset() {
        let mut facade = EntityActionFacade::default();
        facade.mark_entity_as_deleted(1);
        facade.mark_entity_as_deleted(3);

        facade.reset();

        assert!(!facade.deleted_entities.is_marked_as_deleted(1));
        assert!(!facade.deleted_entities.is_marked_as_deleted(3));
        assert!(facade.modified_entities.idxs().next().is_none())
    }
}
