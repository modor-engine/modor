use crate::storages::archetypes::{ArchetypeIdx, EntityLocation};
use crate::storages::components::ComponentTypeIdx;
use crate::storages::core::CoreStorage;
use crate::storages::entities::EntityIdx;
use crate::utils;
use std::mem;
use typed_index_collections::TiVec;

pub(crate) type AddComponentTypeFn = fn(&mut CoreStorage, ArchetypeIdx) -> ArchetypeIdx;
pub(crate) type AddComponentFn =
    Box<dyn FnOnce(&mut CoreStorage, EntityLocation) + Sync + Send>;

#[derive(Default)]
pub(crate) struct UpdateStorage {
    entity_updates: TiVec<EntityIdx, EntityUpdate>,
    modified_entity_idxs: Vec<EntityIdx>,
}

impl UpdateStorage {
    pub(crate) fn drain_entity_updates(
        &mut self,
    ) -> impl Iterator<Item = (EntityIdx, EntityUpdate)> + '_ {
        self.modified_entity_idxs
            .drain(..)
            .map(|e| (e, mem::take(&mut self.entity_updates[e])))
    }

    pub(crate) fn delete_entity(&mut self, entity_idx: EntityIdx) {
        self.add_modified_entity(entity_idx);
        utils::set_value(&mut self.entity_updates, entity_idx, EntityUpdate::Deleted);
    }

    pub(crate) fn add_component(
        &mut self,
        entity_idx: EntityIdx,
        add_type_fn: AddComponentTypeFn,
        add_fn: AddComponentFn,
    ) {
        self.add_modified_entity(entity_idx);
        if let Some(EntityUpdate::Updated(add_fns, _)) = self.entity_updates.get_mut(entity_idx) {
            add_fns.push(AddComponentFns {
                add_type_fn,
                add_fn,
            });
        }
    }

    pub(crate) fn delete_component(&mut self, entity_idx: EntityIdx, type_idx: ComponentTypeIdx) {
        self.add_modified_entity(entity_idx);
        let update = self.entity_updates.get_mut(entity_idx);
        if let Some(EntityUpdate::Updated(_, deleted_types)) = update {
            deleted_types.push(type_idx);
        }
    }

    fn add_modified_entity(&mut self, entity_idx: EntityIdx) {
        let state = self.entity_updates.get(entity_idx);
        if let Some(EntityUpdate::Updated(add_fns, deleted_types)) = state {
            if add_fns.is_empty() && deleted_types.is_empty() {
                self.modified_entity_idxs.push(entity_idx);
            }
        } else if state.is_none() {
            self.modified_entity_idxs.push(entity_idx);
            utils::set_value(
                &mut self.entity_updates,
                entity_idx,
                EntityUpdate::default(),
            );
        }
    }
}

type DeletedComponentTypeIdx = ComponentTypeIdx;

pub(crate) enum EntityUpdate {
    Updated(Vec<AddComponentFns>, Vec<DeletedComponentTypeIdx>),
    Deleted,
}

pub(crate) struct AddComponentFns {
    pub(crate) add_type_fn: AddComponentTypeFn,
    pub(crate) add_fn: AddComponentFn,
}

impl Default for EntityUpdate {
    fn default() -> Self {
        Self::Updated(vec![], vec![])
    }
}

#[cfg(test)]
mod update_storage_tests {
    use crate::storages::updates::{EntityUpdate, UpdateStorage};

    #[test]
    fn delete_entities() {
        let mut storage = UpdateStorage::default();

        storage.delete_entity(3.into());
        storage.delete_entity(1.into());
        storage.delete_entity(3.into());

        let entity_updates: Vec<_> = storage.drain_entity_updates().collect();
        assert_eq!(entity_updates.len(), 2);
        assert_eq!(entity_updates[0].0, 3.into());
        assert!(matches!(entity_updates[0].1, EntityUpdate::Deleted));
        assert_eq!(entity_updates[1].0, 1.into());
        assert!(matches!(entity_updates[1].1, EntityUpdate::Deleted));
        assert_eq!(storage.drain_entity_updates().count(), 0);
    }

    #[test]
    fn add_components() {
        let mut storage = UpdateStorage::default();
        storage.delete_entity(3.into());

        storage.add_component(3.into(), |_, a| a, Box::new(|_, _| ()));
        storage.add_component(1.into(), |_, a| a, Box::new(|_, _| ()));
        storage.add_component(1.into(), |_, a| a, Box::new(|_, _| ()));

        let entity_updates: Vec<_> = storage.drain_entity_updates().collect();
        assert_eq!(entity_updates.len(), 2);
        assert_eq!(entity_updates[0].0, 3.into());
        assert!(matches!(entity_updates[0].1, EntityUpdate::Deleted));
        assert_eq!(entity_updates[1].0, 1.into());
        if let EntityUpdate::Updated(add_fns, deleted_type_idxs) = &entity_updates[1].1 {
            assert_eq!(add_fns.len(), 2);
            assert_eq!(deleted_type_idxs.len(), 0);
        } else {
            panic!("assertion failed: `states[1].1` matches `EntityState::Unchanged(_, _, _)`");
        }
        assert_eq!(storage.drain_entity_updates().count(), 0);
    }

    #[test]
    fn delete_components() {
        let mut storage = UpdateStorage::default();
        storage.delete_entity(3.into());
        storage.add_component(1.into(), |_, a| a, Box::new(|_, _| ()));

        storage.delete_component(3.into(), 0.into());
        storage.delete_component(1.into(), 1.into());
        storage.delete_component(1.into(), 2.into());

        let entity_updates: Vec<_> = storage.drain_entity_updates().collect();
        assert_eq!(entity_updates.len(), 2);
        assert_eq!(entity_updates[0].0, 3.into());
        assert!(matches!(entity_updates[0].1, EntityUpdate::Deleted));
        if let EntityUpdate::Updated(add_fns, deleted_type_idxs) = &entity_updates[1].1 {
            assert_eq!(add_fns.len(), 1);
            assert_eq!(deleted_type_idxs, &[1.into(), 2.into()]);
        } else {
            panic!("assertion failed: `states[1].1` matches `EntityState::Unchanged(_, _, _)`");
        }
        assert_eq!(storage.drain_entity_updates().count(), 0);
    }
}
