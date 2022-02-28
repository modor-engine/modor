use crate::storages::archetypes::EntityLocation;
use std::mem;
use typed_index_collections::TiVec;

#[derive(Default)]
pub(crate) struct EntityStorage {
    deleted_idxs: Vec<EntityIdx>,
    locations: TiVec<EntityIdx, Option<EntityLocation>>,
    parent_idxs: TiVec<EntityIdx, Option<EntityIdx>>,
    child_idxs: TiVec<EntityIdx, Vec<EntityIdx>>,
    depths: TiVec<EntityIdx, usize>,
}

impl EntityStorage {
    pub(crate) fn location(&self, entity_idx: EntityIdx) -> Option<EntityLocation> {
        self.locations.get(entity_idx).copied().flatten()
    }

    pub(crate) fn parent_idx(&self, entity_idx: EntityIdx) -> Option<EntityIdx> {
        self.parent_idxs[entity_idx]
    }

    pub(crate) fn child_idxs(&self, entity_idx: EntityIdx) -> &[EntityIdx] {
        &self.child_idxs[entity_idx]
    }

    pub(crate) fn depth(&self, entity_idx: EntityIdx) -> usize {
        self.depths[entity_idx]
    }

    #[allow(clippy::option_if_let_else)]
    pub(super) fn create(
        &mut self,
        location: EntityLocation,
        parent_idx: Option<EntityIdx>,
    ) -> EntityIdx {
        let depth = parent_idx.map_or(0, |p| self.depths[p] + 1);
        let entity_idx = if let Some(entity_idx) = self.deleted_idxs.pop() {
            self.locations[entity_idx] = Some(location);
            self.parent_idxs[entity_idx] = parent_idx;
            self.depths[entity_idx] = depth;
            entity_idx
        } else {
            self.locations.push(Some(location));
            self.parent_idxs.push(parent_idx);
            self.child_idxs.push(vec![]);
            self.depths.push_and_get_key(depth)
        };
        if let Some(parent_idx) = parent_idx {
            self.child_idxs[parent_idx].push(entity_idx);
        }
        entity_idx
    }

    pub(super) fn set_location(&mut self, entity_idx: EntityIdx, location: EntityLocation) {
        self.locations[entity_idx] = Some(location);
    }

    pub(super) fn delete<F>(&mut self, entity_idx: EntityIdx, mut for_each_deleted_entity_fn: F)
    where
        F: FnMut(&mut Self, EntityLocation),
    {
        if let Some(parent_idx) = self.parent_idxs[entity_idx] {
            let entity_pos = self.child_idxs[parent_idx]
                .iter()
                .position(|&c| c == entity_idx)
                .expect("internal error: child not registered in parent entity");
            self.child_idxs[parent_idx].swap_remove(entity_pos);
        }
        self.delete_internal(entity_idx, &mut for_each_deleted_entity_fn);
    }

    pub(super) fn delete_internal<F>(
        &mut self,
        entity_idx: EntityIdx,
        for_each_deleted_entity_fn: &mut F,
    ) where
        F: FnMut(&mut Self, EntityLocation),
    {
        for child_idx in mem::take(&mut self.child_idxs[entity_idx]) {
            self.delete_internal(child_idx, for_each_deleted_entity_fn);
        }
        self.deleted_idxs.push(entity_idx);
        let location = self.locations[entity_idx]
            .take()
            .expect("internal error: cannot delete entity with no location");
        for_each_deleted_entity_fn(self, location);
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
        let location4 = EntityLocation::new(8.into(), 9.into());
        let unchanged_idx = storage.create(location1, None);
        let moved_idx = storage.create(location2, None);
        let child_idx = storage.create(location4, Some(unchanged_idx));
        storage.set_location(moved_idx, location3);
        assert_eq!(unchanged_idx, 0.into());
        assert_eq!(moved_idx, 1.into());
        assert_eq!(child_idx, 2.into());
        assert_eq!(storage.location(unchanged_idx), Some(location1));
        assert_eq!(storage.location(moved_idx), Some(location3));
        assert_eq!(storage.parent_idx(unchanged_idx), None);
        assert_eq!(storage.parent_idx(child_idx), Some(unchanged_idx));
        assert_eq!(storage.child_idxs(unchanged_idx), [child_idx]);
        assert_eq!(storage.child_idxs(child_idx), []);
        assert_eq!(storage.depth(unchanged_idx), 0);
        assert_eq!(storage.depth(child_idx), 1);
    }

    #[test]
    fn delete_entities() {
        let mut storage = EntityStorage::default();
        let location1 = EntityLocation::new(2.into(), 3.into());
        let location2 = EntityLocation::new(4.into(), 5.into());
        let location3 = EntityLocation::new(6.into(), 7.into());
        let location4 = EntityLocation::new(8.into(), 9.into());
        let root_entity = storage.create(location1, None);
        let deleted_idx = storage.create(location2, Some(root_entity));
        let child_idx = storage.create(location3, Some(deleted_idx));
        let mut deleted_locations = Vec::new();
        storage.delete(deleted_idx, |_, l| deleted_locations.push(l));
        assert_eq!(deleted_locations, [location3, location2]);
        assert_eq!(storage.location(deleted_idx), None);
        assert_eq!(storage.location(child_idx), None);
        assert_eq!(storage.child_idxs(root_entity), []);
        let new_idx = storage.create(location4, None);
        assert_eq!(new_idx, deleted_idx);
        assert_eq!(storage.location(new_idx), Some(location4));
    }
}
