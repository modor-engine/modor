use crate::storages::components::ComponentTypeIdx;
use crate::storages::entities::EntityIdx;
use crate::utils;
use std::cmp::Ordering;
use typed_index_collections::{TiSlice, TiVec};

pub(crate) struct ArchetypeStorage {
    type_idxs: TiVec<ArchetypeIdx, Vec<ComponentTypeIdx>>,
    entity_idxs: TiVec<ArchetypeIdx, TiVec<ArchetypeEntityPos, EntityIdx>>,
    next_idxs: TiVec<ArchetypeIdx, TiVec<ComponentTypeIdx, Option<ArchetypeIdx>>>,
    previous_idxs: TiVec<ArchetypeIdx, TiVec<ComponentTypeIdx, Option<ArchetypeIdx>>>,
}

impl Default for ArchetypeStorage {
    fn default() -> Self {
        Self {
            type_idxs: ti_vec![vec![]],
            entity_idxs: ti_vec![ti_vec![]],
            next_idxs: ti_vec![ti_vec![]],
            previous_idxs: ti_vec![ti_vec![]],
        }
    }
}

impl ArchetypeStorage {
    pub(crate) const DEFAULT_IDX: ArchetypeIdx = ArchetypeIdx(0);

    pub(crate) fn entity_idxs(
        &self,
        archetype_idx: ArchetypeIdx,
    ) -> &TiSlice<ArchetypeEntityPos, EntityIdx> {
        &self.entity_idxs[archetype_idx]
    }

    pub(super) fn sorted_type_idxs(&self, archetype_idx: ArchetypeIdx) -> &[ComponentTypeIdx] {
        &self.type_idxs[archetype_idx]
    }

    pub(super) fn next_entity_pos(&self, archetype_idx: ArchetypeIdx) -> ArchetypeEntityPos {
        self.entity_idxs[archetype_idx].next_key()
    }

    pub(crate) fn all_sorted(&self) -> Vec<ArchetypeInfo> {
        self.entity_idxs
            .iter_enumerated()
            .map(|(a, e)| ArchetypeInfo {
                idx: a,
                entity_count: e.len(),
            })
            .collect()
    }

    pub(crate) fn sorted_with_all_types(
        &self,
        type_idxs: &[ComponentTypeIdx],
    ) -> Vec<ArchetypeInfo> {
        self.entity_idxs
            .iter_enumerated()
            .zip(&self.type_idxs)
            .filter(move |((_, _), t)| Self::contains_all_types(t, type_idxs))
            .map(|((a, e), _)| ArchetypeInfo {
                idx: a,
                entity_count: e.len(),
            })
            .collect()
    }

    pub(super) fn add_component(
        &mut self,
        src_archetype_idx: ArchetypeIdx,
        type_idx: ComponentTypeIdx,
    ) -> Result<ArchetypeIdx, ExistingComponentError> {
        if let Some(&Some(archetype_idx)) = self.next_idxs[src_archetype_idx].get(type_idx) {
            return Ok(archetype_idx);
        }
        let type_pos = self.type_idxs[src_archetype_idx]
            .binary_search(&type_idx)
            .err()
            .ok_or(ExistingComponentError)?;
        let mut dst_type_idxs = self.type_idxs[src_archetype_idx].clone();
        dst_type_idxs.insert(type_pos, type_idx);
        let dst_archetype_idx = self
            .search_idx(&dst_type_idxs)
            .unwrap_or_else(|| self.create_archetype(dst_type_idxs));
        let next_idxs = &mut self.next_idxs[src_archetype_idx];
        utils::set_value(next_idxs, type_idx, Some(dst_archetype_idx));
        Ok(dst_archetype_idx)
    }

    pub(super) fn delete_component(
        &mut self,
        src_archetype_idx: ArchetypeIdx,
        type_idx: ComponentTypeIdx,
    ) -> Result<ArchetypeIdx, MissingComponentError> {
        if let Some(&Some(archetype_idx)) = self.previous_idxs[src_archetype_idx].get(type_idx) {
            return Ok(archetype_idx);
        }
        let type_pos = self.type_idxs[src_archetype_idx]
            .binary_search(&type_idx)
            .map_err(|_| MissingComponentError)?;
        let mut dst_type_idxs = self.type_idxs[src_archetype_idx].clone();
        dst_type_idxs.remove(type_pos);
        let dst_archetype_idx = self
            .search_idx(&dst_type_idxs)
            .unwrap_or_else(|| self.create_archetype(dst_type_idxs));
        let previous_idxs = &mut self.previous_idxs[src_archetype_idx];
        utils::set_value(previous_idxs, type_idx, Some(dst_archetype_idx));
        Ok(dst_archetype_idx)
    }

    pub(super) fn add_entity(
        &mut self,
        entity_idx: EntityIdx,
        archetype_idx: ArchetypeIdx,
    ) -> ArchetypeEntityPos {
        self.entity_idxs[archetype_idx].push_and_get_key(entity_idx)
    }

    pub(super) fn delete_entity(&mut self, location: EntityLocationInArchetype) {
        self.entity_idxs[location.idx].swap_remove(location.pos);
    }

    fn contains_all_types(
        type_idxs: &[ComponentTypeIdx],
        search_type_idxs: &[ComponentTypeIdx],
    ) -> bool {
        search_type_idxs
            .iter()
            .all(|t| type_idxs.binary_search(t).is_ok())
    }

    fn search_idx(&self, type_idxs: &[ComponentTypeIdx]) -> Option<ArchetypeIdx> {
        self.type_idxs
            .iter()
            .position(|t| t == type_idxs)
            .map(Into::into)
    }

    fn create_archetype(&mut self, type_idxs: Vec<ComponentTypeIdx>) -> ArchetypeIdx {
        self.type_idxs.push(type_idxs);
        self.entity_idxs.push(ti_vec![]);
        self.next_idxs.push(ti_vec![]);
        self.previous_idxs.push_and_get_key(ti_vec![])
    }
}

idx_type!(pub ArchetypeIdx);
idx_type!(pub ArchetypeEntityPos);

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct EntityLocationInArchetype {
    pub(crate) idx: ArchetypeIdx,
    pub(crate) pos: ArchetypeEntityPos,
}

#[derive(Eq, Clone, Copy, Debug)]
pub(crate) struct ArchetypeInfo {
    pub(crate) idx: ArchetypeIdx,
    pub(crate) entity_count: usize,
}

impl PartialEq<Self> for ArchetypeInfo {
    fn eq(&self, other: &Self) -> bool {
        self.idx == other.idx
    }
}

impl PartialOrd for ArchetypeInfo {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.idx.partial_cmp(&other.idx)
    }
}

impl Ord for ArchetypeInfo {
    fn cmp(&self, other: &Self) -> Ordering {
        self.idx.cmp(&other.idx)
    }
}

#[derive(PartialEq, Eq, Debug)]
pub(super) struct MissingComponentError;

#[derive(PartialEq, Eq, Debug)]
pub(super) struct ExistingComponentError;

#[cfg(test)]
mod archetype_storage_tests {
    use super::*;

    impl EntityLocationInArchetype {
        pub(crate) fn new(idx: ArchetypeIdx, pos: ArchetypeEntityPos) -> Self {
            Self { idx, pos }
        }
    }

    #[test]
    fn retrieve_default_archetype() {
        let storage = ArchetypeStorage::default();

        let archetype_idx = ArchetypeStorage::DEFAULT_IDX;

        assert_eq!(archetype_idx, 0.into());
        assert_eq!(storage.entity_idxs(archetype_idx).to_vec(), ti_vec![]);
        assert_eq!(storage.sorted_type_idxs(archetype_idx), []);
        assert_eq!(storage.next_entity_pos(archetype_idx), 0.into());
    }

    #[test]
    fn add_component_to_missing_archetype() {
        let mut storage = ArchetypeStorage::default();
        let archetype_idx = ArchetypeStorage::DEFAULT_IDX;
        let type_idx = 10.into();

        let new_archetype_idx = storage.add_component(archetype_idx, type_idx).unwrap();

        assert_eq!(new_archetype_idx, 1.into());
        assert_eq!(storage.sorted_type_idxs(new_archetype_idx), [type_idx]);
    }

    #[test]
    fn add_component_to_direct_existing_archetype() {
        let mut storage = ArchetypeStorage::default();
        let archetype1_idx = ArchetypeStorage::DEFAULT_IDX;
        let type_idx = 10.into();
        let archetype2_idx = storage.add_component(archetype1_idx, type_idx).unwrap();

        let new_archetype_idx = storage.add_component(archetype1_idx, type_idx).unwrap();

        assert_eq!(new_archetype_idx, archetype2_idx);
        assert_eq!(storage.sorted_type_idxs(new_archetype_idx), [type_idx]);
    }

    #[test]
    fn add_component_to_indirect_existing_archetype() {
        let mut storage = ArchetypeStorage::default();
        let archetype1_idx = ArchetypeStorage::DEFAULT_IDX;
        let type1_idx = 10.into();
        let type2_idx = 11.into();
        let archetype2_idx = storage.add_component(archetype1_idx, type1_idx).unwrap();
        let archetype3_idx = storage.add_component(archetype2_idx, type2_idx).unwrap();
        let archetype4_idx = storage.add_component(archetype1_idx, type2_idx).unwrap();

        let new_archetype_idx = storage.add_component(archetype4_idx, type1_idx).unwrap();

        assert_eq!(new_archetype_idx, archetype3_idx);
        assert_eq!(
            storage.sorted_type_idxs(new_archetype_idx),
            [type1_idx, type2_idx]
        );
    }

    #[test]
    fn add_component_with_existing_type() {
        let mut storage = ArchetypeStorage::default();
        let archetype1_idx = ArchetypeStorage::DEFAULT_IDX;
        let type_idx = 10.into();
        let archetype2_idx = storage.add_component(archetype1_idx, type_idx).unwrap();

        let new_archetype_idx = storage.add_component(archetype2_idx, type_idx);

        assert_eq!(new_archetype_idx, Err(ExistingComponentError));
    }

    #[test]
    fn delete_component_to_missing_archetype() {
        let mut storage = ArchetypeStorage::default();
        let archetype1_idx = ArchetypeStorage::DEFAULT_IDX;
        let type1_idx = 10.into();
        let type2_idx = 11.into();
        let archetype2_idx = storage.add_component(archetype1_idx, type1_idx).unwrap();
        let archetype3_idx = storage.add_component(archetype2_idx, type2_idx).unwrap();

        let new_archetype_idx = storage.delete_component(archetype3_idx, type1_idx).unwrap();

        assert_eq!(new_archetype_idx, 3.into());
        assert_eq!(storage.sorted_type_idxs(new_archetype_idx), [type2_idx]);
    }

    #[test]
    fn delete_component_to_direct_existing_archetype() {
        let mut storage = ArchetypeStorage::default();
        let archetype1_idx = ArchetypeStorage::DEFAULT_IDX;
        let type1_idx = 10.into();
        let type2_idx = 11.into();
        let archetype2_idx = storage.add_component(archetype1_idx, type1_idx).unwrap();
        let archetype3_idx = storage.add_component(archetype2_idx, type2_idx).unwrap();
        let archetype4_idx = storage.delete_component(archetype3_idx, type1_idx).unwrap();

        let new_archetype_idx = storage.delete_component(archetype3_idx, type1_idx).unwrap();

        assert_eq!(new_archetype_idx, archetype4_idx);
        assert_eq!(storage.sorted_type_idxs(new_archetype_idx), [type2_idx]);
    }

    #[test]
    fn delete_component_to_indirect_existing_archetype() {
        let mut storage = ArchetypeStorage::default();
        let archetype1_idx = ArchetypeStorage::DEFAULT_IDX;
        let type1_idx = 10.into();
        let type2_idx = 11.into();
        let archetype2_idx = storage.add_component(archetype1_idx, type1_idx).unwrap();
        let archetype3_idx = storage.add_component(archetype2_idx, type2_idx).unwrap();
        let archetype4_idx = storage.delete_component(archetype3_idx, type1_idx).unwrap();

        let new_archetype_idx = storage.delete_component(archetype4_idx, type2_idx).unwrap();

        assert_eq!(new_archetype_idx, archetype1_idx);
        assert_eq!(storage.sorted_type_idxs(new_archetype_idx), []);
    }

    #[test]
    fn delete_component_with_missing_type() {
        let mut storage = ArchetypeStorage::default();
        let archetype1_idx = ArchetypeStorage::DEFAULT_IDX;
        let type1_idx = 10.into();
        let type2_idx = 11.into();
        let archetype2_idx = storage.add_component(archetype1_idx, type1_idx).unwrap();

        let new_archetype_idx = storage.delete_component(archetype2_idx, type2_idx);

        assert_eq!(new_archetype_idx, Err(MissingComponentError));
    }

    #[test]
    fn add_entities() {
        let mut storage = ArchetypeStorage::default();
        let archetype_idx = ArchetypeStorage::DEFAULT_IDX;

        let entity1_pos = storage.add_entity(5.into(), archetype_idx);
        let entity2_pos = storage.add_entity(7.into(), archetype_idx);

        assert_eq!(entity1_pos, 0.into());
        assert_eq!(entity2_pos, 1.into());
        let entity_idxs = storage.entity_idxs(archetype_idx).to_vec();
        assert_eq!(entity_idxs, ti_vec![5.into(), 7.into()]);
        assert_eq!(storage.next_entity_pos(archetype_idx), 2.into());
    }

    #[test]
    fn delete_entity() {
        let mut storage = ArchetypeStorage::default();
        let archetype_idx = ArchetypeStorage::DEFAULT_IDX;
        storage.add_entity(5.into(), archetype_idx);
        storage.add_entity(7.into(), archetype_idx);
        storage.add_entity(9.into(), archetype_idx);
        let deleted_location = EntityLocationInArchetype::new(archetype_idx, 0.into());

        storage.delete_entity(deleted_location);

        let entity_idxs = storage.entity_idxs(archetype_idx).to_vec();
        assert_eq!(entity_idxs, ti_vec![9.into(), 7.into()]);
        assert_eq!(storage.next_entity_pos(archetype_idx), 2.into());
    }

    #[test]
    fn retrieve_all_sorted_archetypes() {
        let mut storage = ArchetypeStorage::default();
        let type1_idx = 10.into();
        let type2_idx = 11.into();
        let archetype1_idx = ArchetypeStorage::DEFAULT_IDX;
        let archetype2_idx = storage.add_component(archetype1_idx, type1_idx).unwrap();
        let archetype3_idx = storage.add_component(archetype1_idx, type2_idx).unwrap();

        let archetypes = storage.all_sorted();

        let archetype1 = create_archetype_info(archetype1_idx, 0);
        let archetype2 = create_archetype_info(archetype2_idx, 0);
        let archetype3 = create_archetype_info(archetype3_idx, 1);
        assert_eq!(archetypes, [archetype1, archetype2, archetype3]);
    }

    #[test]
    fn retrieve_sorted_archetypes_with_one_type() {
        let mut storage = ArchetypeStorage::default();
        let type1_idx = 10.into();
        let type2_idx = 11.into();
        let archetype1_idx = ArchetypeStorage::DEFAULT_IDX;
        let archetype2_idx = storage.add_component(archetype1_idx, type1_idx).unwrap();
        let archetype3_idx = storage.add_component(archetype2_idx, type2_idx).unwrap();

        let archetypes = storage.sorted_with_all_types(&[type1_idx]);

        let archetype1 = create_archetype_info(archetype2_idx, 0);
        let archetype2 = create_archetype_info(archetype3_idx, 1);
        assert_eq!(archetypes, [archetype1, archetype2]);
    }

    #[test]
    fn retrieve_sorted_archetypes_with_multiple_type() {
        let mut storage = ArchetypeStorage::default();
        let type1_idx = 10.into();
        let type2_idx = 11.into();
        let archetype1_idx = ArchetypeStorage::DEFAULT_IDX;
        let archetype2_idx = storage.add_component(archetype1_idx, type1_idx).unwrap();
        let archetype3_idx = storage.add_component(archetype2_idx, type2_idx).unwrap();

        let archetypes = storage.sorted_with_all_types(&[type1_idx, type2_idx]);

        assert_eq!(archetypes, [create_archetype_info(archetype3_idx, 1)]);
    }

    fn create_archetype_info(idx: ArchetypeIdx, entity_count: usize) -> ArchetypeInfo {
        ArchetypeInfo { idx, entity_count }
    }
}
