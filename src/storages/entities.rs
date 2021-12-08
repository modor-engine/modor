use crate::storages::archetypes::EntityLocationInArchetype;
use typed_index_collections::TiVec;

#[derive(Default)]
pub(crate) struct EntityStorage {
    deleted_idxs: Vec<EntityIdx>,
    locations: TiVec<EntityIdx, Option<EntityLocationInArchetype>>,
}

impl EntityStorage {
    pub(crate) fn location(&self, entity_idx: EntityIdx) -> Option<EntityLocationInArchetype> {
        self.locations.get(entity_idx).copied().flatten()
    }

    #[allow(clippy::option_if_let_else)]
    pub(super) fn create(&mut self, location: EntityLocationInArchetype) -> EntityIdx {
        if let Some(entity_idx) = self.deleted_idxs.pop() {
            self.locations[entity_idx] = Some(location);
            entity_idx
        } else {
            self.locations.push_and_get_key(Some(location))
        }
    }

    pub(super) fn set_location(
        &mut self,
        entity_idx: EntityIdx,
        location: EntityLocationInArchetype,
    ) {
        self.locations[entity_idx] = Some(location);
    }

    pub(super) fn delete(&mut self, entity_idx: EntityIdx) {
        self.deleted_idxs.push(entity_idx);
        self.locations[entity_idx] = None;
    }
}

idx_type!(pub EntityIdx);

#[cfg(test)]
mod entity_storage_tests {
    use super::*;

    #[test]
    fn create_entities() {
        let mut storage = EntityStorage::default();
        let location1 = EntityLocationInArchetype::new(2.into(), 3.into());
        let location2 = EntityLocationInArchetype::new(4.into(), 5.into());

        let entity1_idx = storage.create(location1);
        let entity2_idx = storage.create(location2);

        assert_eq!(entity1_idx, 0.into());
        assert_eq!(entity2_idx, 1.into());
        assert_eq!(storage.location(entity1_idx), Some(location1));
        assert_eq!(storage.location(entity2_idx), Some(location2));
    }

    #[test]
    fn set_location() {
        let mut storage = EntityStorage::default();
        let entity_idx = storage.create(EntityLocationInArchetype::new(2.into(), 3.into()));
        let new_location = EntityLocationInArchetype::new(4.into(), 5.into());

        storage.set_location(entity_idx, new_location);

        assert_eq!(storage.location(entity_idx), Some(new_location));
    }

    #[test]
    fn delete_entity() {
        let mut storage = EntityStorage::default();
        let entity1_idx = storage.create(EntityLocationInArchetype::new(2.into(), 3.into()));
        let location2 = EntityLocationInArchetype::new(4.into(), 5.into());
        let entity2_idx = storage.create(location2);

        storage.delete(entity1_idx);

        assert_eq!(storage.location(entity1_idx), None);
        assert_eq!(storage.location(entity2_idx), Some(location2));
    }

    #[test]
    fn create_entities_after_deletion() {
        let mut storage = EntityStorage::default();
        let entity1_idx = storage.create(EntityLocationInArchetype::new(2.into(), 3.into()));
        let entity2_idx = storage.create(EntityLocationInArchetype::new(4.into(), 5.into()));
        storage.delete(entity1_idx);
        let location3 = EntityLocationInArchetype::new(6.into(), 7.into());

        let entity3_idx = storage.create(location3);

        assert_eq!(entity3_idx, entity1_idx);
        assert_eq!(storage.location(entity1_idx), Some(location3));
        let entity4_idx = storage.create(EntityLocationInArchetype::new(4.into(), 5.into()));
        assert_eq!(entity4_idx, entity2_idx.next());
    }
}
