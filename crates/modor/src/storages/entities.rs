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
