use crate::storages::archetypes::EntityLocation;
use typed_index_collections::TiVec;

#[derive(Default)]
pub(crate) struct EntityStorage {
    deleted_idxs: Vec<EntityIdx>,
    locations: TiVec<EntityIdx, Option<EntityLocation>>,
}

impl EntityStorage {
    pub(crate) fn location(&self, entity_idx: EntityIdx) -> Option<EntityLocation> {
        self.locations.get(entity_idx).copied().flatten()
    }

    #[allow(clippy::option_if_let_else)]
    pub(super) fn create(&mut self, location: EntityLocation) -> EntityIdx {
        if let Some(entity_idx) = self.deleted_idxs.pop() {
            self.locations[entity_idx] = Some(location);
            entity_idx
        } else {
            self.locations.push_and_get_key(Some(location))
        }
    }

    pub(super) fn set_location(&mut self, entity_idx: EntityIdx, location: EntityLocation) {
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
    use crate::storages::archetypes::EntityLocation;
    use crate::storages::entities::EntityStorage;

    #[test]
    fn configure_entities() {
        let mut storage = EntityStorage::default();
        let location1 = EntityLocation::new(2.into(), 3.into());
        let location2 = EntityLocation::new(4.into(), 5.into());
        let location3 = EntityLocation::new(6.into(), 7.into());
        let unchanged_entity_idx = storage.create(location1);
        let moved_entity_idx = storage.create(location2);
        storage.set_location(moved_entity_idx, location3);
        assert_eq!(unchanged_entity_idx, 0.into());
        assert_eq!(moved_entity_idx, 1.into());
        assert_eq!(storage.location(unchanged_entity_idx), Some(location1));
        assert_eq!(storage.location(moved_entity_idx), Some(location3));
    }

    #[test]
    fn delete_entities() {
        let mut storage = EntityStorage::default();
        let location1 = EntityLocation::new(2.into(), 3.into());
        let location2 = EntityLocation::new(4.into(), 5.into());
        let deleted_entity_idx = storage.create(location1);
        storage.delete(deleted_entity_idx);
        assert_eq!(storage.location(deleted_entity_idx), None);
        let new_entity_idx = storage.create(location2);
        assert_eq!(new_entity_idx, deleted_entity_idx);
        assert_eq!(storage.location(new_entity_idx), Some(location2));
    }
}
