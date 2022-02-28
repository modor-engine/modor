use crate::storages::archetypes::{ArchetypeIdx, EntityLocation};
use crate::storages::components::ComponentTypeIdx;
use crate::storages::core::CoreStorage;
use crate::storages::entities::EntityIdx;
use crate::utils;
use std::iter::Rev;
use std::mem;
use std::ops::Range;
use std::vec::Drain;
use typed_index_collections::TiVec;

pub(crate) type CreateEntityFn = Box<dyn FnOnce(&mut CoreStorage) + Sync + Send>;
pub(crate) type AddComponentTypeFn = fn(&mut CoreStorage, ArchetypeIdx) -> ArchetypeIdx;
pub(crate) type AddComponentFn = Box<dyn FnOnce(&mut CoreStorage, EntityLocation) + Sync + Send>;

#[derive(Default)]
pub(crate) struct UpdateStorage {
    entity_updates: TiVec<EntityIdx, EntityUpdate>,
    modified_entity_idxs: Vec<EntityIdx>,
    created_root_entities: Vec<CreateEntityFn>,
    created_child_entities: Vec<(CreateEntityFn, ParentEntityIdx)>,
}

impl UpdateStorage {
    pub(crate) fn deleted_entity_drain(&mut self) -> DeletedEntityDrain<'_> {
        DeletedEntityDrain {
            entity_updates: &mut self.entity_updates,
            modified_entity_positions: (0..self.modified_entity_idxs.len()).rev(),
            modified_entity_idxs: &mut self.modified_entity_idxs,
        }
    }

    pub(crate) fn changed_entity_drain(&mut self) -> ChangedEntityDrain<'_> {
        ChangedEntityDrain {
            entity_updates: &mut self.entity_updates,
            modified_entity_positions: (0..self.modified_entity_idxs.len()).rev(),
            modified_entity_idxs: &mut self.modified_entity_idxs,
        }
    }

    pub(crate) fn created_root_entity_drain(&mut self) -> Drain<'_, CreateEntityFn> {
        self.created_root_entities.drain(..)
    }

    pub(crate) fn created_child_entity_drain(
        &mut self,
    ) -> Drain<'_, (CreateEntityFn, ParentEntityIdx)> {
        self.created_child_entities.drain(..)
    }

    pub(crate) fn delete_entity(&mut self, entity_idx: EntityIdx) {
        self.add_modified_entity(entity_idx);
        utils::set_value(&mut self.entity_updates, entity_idx, EntityUpdate::Deletion);
    }

    pub(crate) fn create_entity(
        &mut self,
        parent_idx: Option<ParentEntityIdx>,
        create_fn: CreateEntityFn,
    ) {
        if let Some(parent_idx) = parent_idx {
            self.created_child_entities.push((create_fn, parent_idx));
        } else {
            self.created_root_entities.push(create_fn);
        }
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

pub(crate) struct DeletedEntityDrain<'a> {
    entity_updates: &'a mut TiVec<EntityIdx, EntityUpdate>,
    modified_entity_idxs: &'a mut Vec<EntityIdx>,
    modified_entity_positions: Rev<Range<usize>>,
}

impl Iterator for DeletedEntityDrain<'_> {
    type Item = EntityIdx;

    fn next(&mut self) -> Option<Self::Item> {
        for pos in &mut self.modified_entity_positions {
            let update = &self.entity_updates[self.modified_entity_idxs[pos]];
            if matches!(update, EntityUpdate::Deletion) {
                let entity_idx = self.modified_entity_idxs.swap_remove(pos);
                self.entity_updates[entity_idx] = EntityUpdate::default();
                return Some(entity_idx);
            }
        }
        None
    }
}

pub(crate) struct ChangedEntityDrain<'a> {
    entity_updates: &'a mut TiVec<EntityIdx, EntityUpdate>,
    modified_entity_idxs: &'a mut Vec<EntityIdx>,
    modified_entity_positions: Rev<Range<usize>>,
}

impl Iterator for ChangedEntityDrain<'_> {
    type Item = (
        EntityIdx,
        Vec<AddComponentFns>,
        Vec<DeletedComponentTypeIdx>,
    );

    fn next(&mut self) -> Option<Self::Item> {
        for pos in &mut self.modified_entity_positions {
            let entity_idx = self.modified_entity_idxs[pos];
            if let EntityUpdate::Change(add_fns, delete_fns) = &mut self.entity_updates[entity_idx]
            {
                self.modified_entity_idxs.swap_remove(pos);
                return Some((entity_idx, mem::take(add_fns), mem::take(delete_fns)));
            }
        }
        None
    }
}

type ParentEntityIdx = EntityIdx;
type DeletedComponentTypeIdx = ComponentTypeIdx;

enum EntityUpdate {
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
    use crate::storages::updates::UpdateStorage;

    #[test]
    fn update_entities() {
        let mut storage = UpdateStorage::default();
        storage.add_component(3.into(), |_, a| a, Box::new(|_, _| ()));
        storage.add_component(5.into(), |_, a| a, Box::new(|_, _| ()));
        storage.delete_entity(5.into());
        storage.delete_component(5.into(), 10.into());
        storage.delete_component(3.into(), 20.into());
        storage.delete_component(1.into(), 30.into());
        storage.create_entity(None, Box::new(|_| ()));
        storage.create_entity(Some(40.into()), Box::new(|_| ()));
        let deleted_entity_idxs: Vec<_> = storage.deleted_entity_drain().collect();
        assert_eq!(deleted_entity_idxs, [5.into()]);
        let changed_entities: Vec<_> = storage.changed_entity_drain().collect();
        assert_eq!(changed_entities.len(), 2);
        assert_eq!(changed_entities[0].0, 1.into());
        assert_eq!(changed_entities[0].1.len(), 0);
        assert_eq!(changed_entities[0].2, [30.into()]);
        assert_eq!(changed_entities[1].0, 3.into());
        assert_eq!(changed_entities[1].1.len(), 1);
        assert_eq!(changed_entities[1].2, [20.into()]);
        assert_eq!(storage.created_root_entity_drain().count(), 1);
        let created_entities: Vec<_> = storage.created_child_entity_drain().collect();
        assert_eq!(created_entities.len(), 1);
        assert_eq!(created_entities[0].1, 40.into());
        assert!(storage.deleted_entity_drain().next().is_none());
        assert!(storage.changed_entity_drain().next().is_none());
        assert!(storage.created_root_entity_drain().next().is_none());
        assert!(storage.created_child_entity_drain().next().is_none());
    }
}
