use crate::internal::entities::data::EntityLocation;
use crate::internal::entities::storages::{ArchetypeEntityStorage, EntityStorage, LocationStorage};

pub(super) mod data;
mod storages;

#[derive(Default)]
pub(super) struct EntityFacade {
    entities: EntityStorage,
    locations: LocationStorage,
    archetype_entities: ArchetypeEntityStorage,
}

impl EntityFacade {
    pub(super) fn location(&self, entity_idx: usize) -> Option<EntityLocation> {
        self.locations.get(entity_idx)
    }

    pub(super) fn idxs(&self, archetype_idx: usize) -> &[usize] {
        self.archetype_entities.idxs(archetype_idx)
    }

    pub(super) fn create(&mut self) -> usize {
        self.entities.create()
    }

    pub(super) fn move_(&mut self, entity_idx: usize, dst_archetype_idx: Option<usize>) {
        let src_location = self.locations.get(entity_idx);
        let move_info = self
            .archetype_entities
            .move_(entity_idx, src_location, dst_archetype_idx);
        let (dst_location, moved_entity_idx) = move_info;
        self.locations.set(entity_idx, dst_location);
        if let Some(moved_entity_idx) = moved_entity_idx {
            self.locations.set(moved_entity_idx, src_location);
        }
    }

    pub(super) fn delete(&mut self, entity_idx: usize) {
        self.entities.delete(entity_idx);
        let location = self.locations.remove(entity_idx);
        let move_info = self.archetype_entities.move_(entity_idx, location, None);
        if let Some(moved_entity_idx) = move_info.1 {
            self.locations.set(moved_entity_idx, location);
        }
    }
}

#[cfg(test)]
mod tests_entity_facade {
    use super::*;

    #[test]
    fn create_entity() {
        let mut facade = EntityFacade::default();

        let entity_idx = facade.create();

        assert_eq!(entity_idx, 0);
        assert_eq!(facade.entities.create(), 1);
    }

    #[test]
    fn move_entity_to_empty_archetype() {
        let mut facade = EntityFacade::default();
        let entity_idx = facade.create();

        facade.move_(entity_idx, Some(6));

        let location = EntityLocation::new(6, 0);
        assert_eq!(facade.locations.get(entity_idx), Some(location));
        assert_eq!(facade.archetype_entities.idxs(6), [entity_idx]);
    }

    #[test]
    fn move_entity_to_nonempty_archetype_with_moving_other_entity() {
        let mut facade = EntityFacade::default();
        let entity1_idx = facade.create();
        let entity2_idx = facade.create();
        facade.move_(entity1_idx, Some(5));
        facade.move_(entity2_idx, Some(5));

        facade.move_(entity1_idx, Some(6));

        let location = Some(EntityLocation::new(6, 0));
        assert_eq!(facade.locations.get(entity1_idx), location);
        let location = Some(EntityLocation::new(5, 0));
        assert_eq!(facade.locations.get(entity2_idx), location);
        assert_eq!(facade.archetype_entities.idxs(5), [entity2_idx]);
        assert_eq!(facade.archetype_entities.idxs(6), [entity1_idx]);
    }

    #[test]
    fn move_entity_to_nonempty_archetype_without_moving_other_entity() {
        let mut facade = EntityFacade::default();
        let entity1_idx = facade.create();
        let entity2_idx = facade.create();
        facade.move_(entity1_idx, Some(5));
        facade.move_(entity2_idx, Some(5));

        facade.move_(entity2_idx, Some(6));

        let location = Some(EntityLocation::new(5, 0));
        assert_eq!(facade.locations.get(entity1_idx), location);
        let location = Some(EntityLocation::new(6, 0));
        assert_eq!(facade.locations.get(entity2_idx), location);
        assert_eq!(facade.archetype_entities.idxs(5), [entity1_idx]);
        assert_eq!(facade.archetype_entities.idxs(6), [entity2_idx]);
    }

    #[test]
    fn retrieve_entity_location() {
        let mut facade = EntityFacade::default();
        let entity1_idx = facade.create();
        let entity2_idx = facade.create();
        facade.move_(entity1_idx, Some(5));
        facade.move_(entity2_idx, Some(5));

        let actual_entity_location = facade.location(entity2_idx);

        assert_eq!(actual_entity_location, Some(EntityLocation::new(5, 1)));
    }

    #[test]
    fn retrieve_entity_idxs_from_archetype() {
        let mut facade = EntityFacade::default();
        let entity1_idx = facade.create();
        let entity2_idx = facade.create();
        facade.move_(entity1_idx, Some(5));
        facade.move_(entity2_idx, Some(5));

        let entity_idxs = facade.idxs(5);

        assert_eq!(entity_idxs, [entity1_idx, entity2_idx]);
    }

    #[test]
    fn delete_entity_with_moving_other_entity() {
        let mut facade = EntityFacade::default();
        let entity1_idx = facade.create();
        let entity2_idx = facade.create();
        facade.move_(entity1_idx, Some(5));
        facade.move_(entity2_idx, Some(5));

        facade.delete(entity1_idx);

        assert_eq!(facade.locations.get(entity1_idx), None);
        let location = Some(EntityLocation::new(5, 0));
        assert_eq!(facade.locations.get(entity2_idx), location);
        assert_eq!(facade.archetype_entities.idxs(5), [entity2_idx]);
        assert_eq!(facade.entities.create(), entity1_idx);
    }

    #[test]
    fn delete_entity_without_moving_other_entity() {
        let mut facade = EntityFacade::default();
        let entity1_idx = facade.create();
        let entity2_idx = facade.create();
        facade.move_(entity1_idx, Some(5));
        facade.move_(entity2_idx, Some(5));

        facade.delete(entity2_idx);

        let location = Some(EntityLocation::new(5, 0));
        assert_eq!(facade.locations.get(entity1_idx), location);
        assert_eq!(facade.locations.get(entity2_idx), None);
        assert_eq!(facade.archetype_entities.idxs(5), [entity1_idx]);
        assert_eq!(facade.entities.create(), entity2_idx);
    }
}
