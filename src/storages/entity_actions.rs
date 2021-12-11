use crate::storages::archetypes::{ArchetypeIdx, EntityLocationInArchetype};
use crate::storages::components::ComponentTypeIdx;
use crate::storages::core::CoreStorage;
use crate::storages::entities::EntityIdx;
use crate::utils;
use std::mem;
use typed_index_collections::TiVec;

pub(crate) type AddComponentTypeFn = fn(&mut CoreStorage, ArchetypeIdx) -> ArchetypeIdx;
pub(crate) type AddComponentFn =
    Box<dyn FnOnce(&mut CoreStorage, EntityLocationInArchetype) + Sync + Send>;

#[derive(Default)]
pub(crate) struct EntityActionStorage {
    entity_states: TiVec<EntityIdx, EntityState>,
    modified_entity_idxs: Vec<EntityIdx>,
}

impl EntityActionStorage {
    pub(crate) fn drain_entity_states(
        &mut self,
    ) -> impl Iterator<Item = (EntityIdx, EntityState)> + '_ {
        self.modified_entity_idxs
            .drain(..)
            .map(|e| (e, mem::take(&mut self.entity_states[e])))
    }

    pub(crate) fn delete_entity(&mut self, entity_idx: EntityIdx) {
        self.add_modified_entity(entity_idx);
        utils::set_value(&mut self.entity_states, entity_idx, EntityState::Deleted);
    }

    pub(crate) fn add_component(
        &mut self,
        entity_idx: EntityIdx,
        add_type_fn: AddComponentTypeFn,
        add_fn: AddComponentFn,
    ) {
        self.add_modified_entity(entity_idx);
        if let Some(EntityState::Unchanged(add_type_fns, add_fns, _)) =
            self.entity_states.get_mut(entity_idx)
        {
            add_type_fns.push(add_type_fn);
            add_fns.push(add_fn);
        }
    }

    pub(crate) fn delete_component(&mut self, entity_idx: EntityIdx, type_idx: ComponentTypeIdx) {
        self.add_modified_entity(entity_idx);
        let state = self.entity_states.get_mut(entity_idx);
        if let Some(EntityState::Unchanged(_, _, deleted_types)) = state {
            deleted_types.push(type_idx);
        }
    }

    fn add_modified_entity(&mut self, entity_idx: EntityIdx) {
        let state = self.entity_states.get(entity_idx);
        if let Some(EntityState::Unchanged(add_type_fns, add_fns, deleted_types)) = state {
            if add_type_fns.is_empty() && add_fns.is_empty() && deleted_types.is_empty() {
                self.modified_entity_idxs.push(entity_idx);
            }
        } else if state.is_none() {
            self.modified_entity_idxs.push(entity_idx);
            utils::set_value(&mut self.entity_states, entity_idx, EntityState::default());
        }
    }
}

type DeletedComponentTypeIdx = ComponentTypeIdx;

pub(crate) enum EntityState {
    Unchanged(
        Vec<AddComponentTypeFn>,
        Vec<AddComponentFn>,
        Vec<DeletedComponentTypeIdx>,
    ),
    Deleted,
}

impl Default for EntityState {
    fn default() -> Self {
        Self::Unchanged(vec![], vec![], vec![])
    }
}

#[cfg(test)]
mod entity_action_storage_tests {
    use super::*;

    #[test]
    fn delete_entities() {
        let mut storage = EntityActionStorage::default();

        storage.delete_entity(3.into());
        storage.delete_entity(1.into());
        storage.delete_entity(3.into());

        let states: Vec<_> = storage.drain_entity_states().collect();
        assert_eq!(states.len(), 2);
        assert_eq!(states[0].0, 3.into());
        assert!(matches!(states[0].1, EntityState::Deleted));
        assert_eq!(states[1].0, 1.into());
        assert!(matches!(states[1].1, EntityState::Deleted));
        assert_eq!(storage.drain_entity_states().count(), 0);
    }

    #[test]
    fn add_components() {
        let mut storage = EntityActionStorage::default();
        storage.delete_entity(3.into());

        storage.add_component(3.into(), |_, a| a, Box::new(|_, _| ()));
        storage.add_component(1.into(), |_, a| a, Box::new(|_, _| ()));
        storage.add_component(1.into(), |_, a| a, Box::new(|_, _| ()));

        let states: Vec<_> = storage.drain_entity_states().collect();
        assert_eq!(states.len(), 2);
        assert_eq!(states[0].0, 3.into());
        assert!(matches!(states[0].1, EntityState::Deleted));
        assert_eq!(states[1].0, 1.into());
        if let EntityState::Unchanged(add_type_fn, add_fn, deleted_type_idxs) = &states[1].1 {
            assert_eq!(add_type_fn.len(), 2);
            assert_eq!(add_fn.len(), 2);
            assert_eq!(deleted_type_idxs.len(), 0);
        } else {
            panic!("assertion failed: `states[1].1` matches `EntityState::Unchanged(_, _, _)`");
        }
        assert_eq!(storage.drain_entity_states().count(), 0);
    }

    #[test]
    fn delete_components() {
        let mut storage = EntityActionStorage::default();
        storage.delete_entity(3.into());
        storage.add_component(1.into(), |_, a| a, Box::new(|_, _| ()));

        storage.delete_component(3.into(), 0.into());
        storage.delete_component(1.into(), 1.into());
        storage.delete_component(1.into(), 2.into());

        let states: Vec<_> = storage.drain_entity_states().collect();
        assert_eq!(states.len(), 2);
        assert_eq!(states[0].0, 3.into());
        assert!(matches!(states[0].1, EntityState::Deleted));
        if let EntityState::Unchanged(add_type_fn, add_fn, deleted_type_idxs) = &states[1].1 {
            assert_eq!(add_type_fn.len(), 1);
            assert_eq!(add_fn.len(), 1);
            assert_eq!(deleted_type_idxs, &[1.into(), 2.into()]);
        } else {
            panic!("assertion failed: `states[1].1` matches `EntityState::Unchanged(_, _, _)`");
        }
        assert_eq!(storage.drain_entity_states().count(), 0);
    }
}
