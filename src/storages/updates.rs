use crate::storages::archetypes::{ArchetypeIdx, EntityLocation};
use crate::storages::components::ComponentTypeIdx;
use crate::storages::core::CoreStorage;
use crate::storages::entities::EntityIdx;
use crate::utils;
use std::mem;
use typed_index_collections::TiVec;

pub(crate) type AddComponentTypeFn = fn(&mut CoreStorage, ArchetypeIdx) -> ArchetypeIdx;
pub(crate) type AddComponentFn = Box<dyn FnOnce(&mut CoreStorage, EntityLocation) + Sync + Send>;

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
        utils::set_value(&mut self.entity_updates, entity_idx, EntityUpdate::Deletion);
    }

    pub(crate) fn add_component(
        &mut self,
        entity_idx: EntityIdx,
        add_type_fn: AddComponentTypeFn,
        add_fn: AddComponentFn,
    ) {
        self.add_modified_entity(entity_idx);
        if let Some(EntityUpdate::Change(add_fns, _)) = self.entity_updates.get_mut(entity_idx) {
            add_fns.push(AddComponentFns {
                add_type_fn,
                add_fn,
            });
        }
    }

    pub(crate) fn delete_component(&mut self, entity_idx: EntityIdx, type_idx: ComponentTypeIdx) {
        self.add_modified_entity(entity_idx);
        let update = self.entity_updates.get_mut(entity_idx);
        if let Some(EntityUpdate::Change(_, deleted_types)) = update {
            deleted_types.push(type_idx);
        }
    }

    fn add_modified_entity(&mut self, entity_idx: EntityIdx) {
        let update = self.entity_updates.get(entity_idx);
        if let Some(EntityUpdate::Change(add_fns, deleted_types)) = update {
            if add_fns.is_empty() && deleted_types.is_empty() {
                self.modified_entity_idxs.push(entity_idx);
            }
        } else if update.is_none() {
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
    Change(Vec<AddComponentFns>, Vec<DeletedComponentTypeIdx>),
    Deletion,
}

pub(crate) struct AddComponentFns {
    pub(crate) add_type_fn: AddComponentTypeFn,
    pub(crate) add_fn: AddComponentFn,
}

impl Default for EntityUpdate {
    fn default() -> Self {
        Self::Change(vec![], vec![])
    }
}

#[cfg(test)]
mod update_storage_tests {
    use crate::storages::components::ComponentTypeIdx;
    use crate::storages::updates::{EntityUpdate, UpdateStorage};

    #[test]
    fn update_entities() {
        let mut storage = UpdateStorage::default();
        storage.delete_entity(5.into());
        storage.add_component(5.into(), |_, a| a, Box::new(|_, _| ()));
        storage.add_component(3.into(), |_, a| a, Box::new(|_, _| ()));
        storage.delete_component(5.into(), 10.into());
        storage.delete_component(3.into(), 20.into());
        storage.delete_component(1.into(), 30.into());
        let entity_updates: Vec<_> = storage.drain_entity_updates().collect();
        assert_eq!(storage.drain_entity_updates().count(), 0);
        assert_eq!(entity_updates.len(), 3);
        assert_eq!(entity_updates[0].0, 5.into());
        assert!(matches!(entity_updates[0].1, EntityUpdate::Deletion));
        assert_eq!(entity_updates[1].0, 3.into());
        assert_entity_is_updated(&entity_updates[1].1, 1, &[20.into()]);
        assert_eq!(entity_updates[2].0, 1.into());
        assert_entity_is_updated(&entity_updates[2].1, 0, &[30.into()]);
    }

    fn assert_entity_is_updated(
        update: &EntityUpdate,
        added_component_count: usize,
        deleted_type_idxs: &[ComponentTypeIdx],
    ) {
        assert!(matches!(update, EntityUpdate::Change(_, _)));
        if let EntityUpdate::Change(add_fns, actual_deleted_type_idxs) = update {
            assert_eq!(add_fns.len(), added_component_count);
            assert_eq!(actual_deleted_type_idxs, deleted_type_idxs);
        }
    }
}
