use crate::internal::entity_actions::storages::{DeletedEntitiesStorage, ModifiedEntitiesStorage};

mod storages;

#[derive(Default)]
pub(crate) struct EntityActionFacade {
    deleted_entities: DeletedEntitiesStorage,
    modified_entities: ModifiedEntitiesStorage,
}

impl EntityActionFacade {
    pub(super) fn deleted_entity_idxs(&self) -> impl Iterator<Item = usize> + '_ {
        let deleted_entities = &self.deleted_entities;
        self.modified_entities
            .idxs()
            .filter(move |&i| deleted_entities.is_marked_as_deleted(i))
    }

    pub(crate) fn mark_entity_as_deleted(&mut self, entity_idx: usize) {
        self.deleted_entities.add(entity_idx);
        self.modified_entities.add(entity_idx);
    }

    pub(super) fn reset(&mut self) {
        for entity_idx in self.modified_entities.idxs() {
            self.deleted_entities.delete(entity_idx);
        }
        self.modified_entities.reset();
    }
}

#[cfg(test)]
mod tests_entity_action_facade {
    use super::*;

    #[test]
    fn mark_entity_as_deleted() {
        let mut facade = EntityActionFacade::default();

        facade.mark_entity_as_deleted(1);

        assert!(facade.deleted_entities.is_marked_as_deleted(1));
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
