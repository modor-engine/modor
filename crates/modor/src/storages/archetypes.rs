use crate::storages::components::ComponentTypeIdx;
use crate::storages::entities::EntityIdx;
use modor_internal::ti_vec;
use std::slice::Iter;
use typed_index_collections::{TiSlice, TiVec};

pub(crate) struct ArchetypeStorage {
    type_idxs: TiVec<ArchetypeIdx, Vec<ComponentTypeIdx>>,
    entity_idxs: TiVec<ArchetypeIdx, TiVec<ArchetypeEntityPos, EntityIdx>>,
    next_idxs: TiVec<ArchetypeIdx, TiVec<ComponentTypeIdx, Option<ArchetypeIdx>>>,
    previous_idxs: TiVec<ArchetypeIdx, TiVec<ComponentTypeIdx, Option<ArchetypeIdx>>>,
    all_sorted_idxs: Vec<ArchetypeIdx>,
}

impl Default for ArchetypeStorage {
    fn default() -> Self {
        Self {
            type_idxs: ti_vec![vec![]],
            entity_idxs: ti_vec![ti_vec![]],
            next_idxs: ti_vec![ti_vec![]],
            previous_idxs: ti_vec![ti_vec![]],
            all_sorted_idxs: vec![0.into()],
        }
    }
}

impl ArchetypeStorage {
    pub(crate) const DEFAULT_IDX: ArchetypeIdx = ArchetypeIdx(0);

    #[inline]
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

    pub(crate) fn all_sorted_idxs(&self) -> &[ArchetypeIdx] {
        &self.all_sorted_idxs
    }

    pub(crate) fn filter_idxs<'a>(
        &'a self,
        archetype_idxs: Iter<'a, ArchetypeIdx>,
        filtered_type_idxs: &'a [ComponentTypeIdx],
    ) -> FilteredArchetypeIdxIter<'a> {
        FilteredArchetypeIdxIter {
            archetype_type_idxs: &self.type_idxs,
            archetype_idxs,
            filtered_type_idxs,
        }
    }

    #[inline]
    pub(crate) fn has_types(
        &self,
        archetype_idx: ArchetypeIdx,
        type_idxs: &[ComponentTypeIdx],
    ) -> bool {
        let archetype_type_idxs = &self.type_idxs[archetype_idx];
        type_idxs
            .iter()
            .all(|t| archetype_type_idxs.binary_search(t).is_ok())
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
        ti_vec::set_value(next_idxs, type_idx, Some(dst_archetype_idx));
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
        ti_vec::set_value(previous_idxs, type_idx, Some(dst_archetype_idx));
        Ok(dst_archetype_idx)
    }

    pub(super) fn add_entity(
        &mut self,
        entity_idx: EntityIdx,
        archetype_idx: ArchetypeIdx,
    ) -> ArchetypeEntityPos {
        self.entity_idxs[archetype_idx].push_and_get_key(entity_idx)
    }

    pub(super) fn delete_entity(&mut self, location: EntityLocation) {
        self.entity_idxs[location.idx].swap_remove(location.pos);
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
        let archetype_idx = self.previous_idxs.push_and_get_key(ti_vec![]);
        self.all_sorted_idxs.push(archetype_idx);
        archetype_idx
    }
}

#[derive(Clone)]
pub(crate) struct FilteredArchetypeIdxIter<'a> {
    archetype_type_idxs: &'a TiVec<ArchetypeIdx, Vec<ComponentTypeIdx>>,
    archetype_idxs: Iter<'a, ArchetypeIdx>,
    filtered_type_idxs: &'a [ComponentTypeIdx],
}

impl Iterator for FilteredArchetypeIdxIter<'_> {
    type Item = ArchetypeIdx;

    fn next(&mut self) -> Option<Self::Item> {
        Self::next_idx(
            &mut self.archetype_idxs,
            self.archetype_type_idxs,
            self.filtered_type_idxs,
        )
    }
}

impl DoubleEndedIterator for FilteredArchetypeIdxIter<'_> {
    fn next_back(&mut self) -> Option<Self::Item> {
        Self::next_idx(
            (&mut self.archetype_idxs).rev(),
            self.archetype_type_idxs,
            self.filtered_type_idxs,
        )
    }
}

impl FilteredArchetypeIdxIter<'_> {
    fn next_idx<'a, I>(
        archetype_idxs: I,
        archetype_type_idxs: &TiVec<ArchetypeIdx, Vec<ComponentTypeIdx>>,
        filtered_type_idxs: &[ComponentTypeIdx],
    ) -> Option<ArchetypeIdx>
    where
        I: Iterator<Item = &'a ArchetypeIdx>,
    {
        for &archetype_idx in archetype_idxs {
            let archetype_type_idxs = &archetype_type_idxs[archetype_idx];
            if filtered_type_idxs
                .iter()
                .all(|t| archetype_type_idxs.binary_search(t).is_ok())
            {
                return Some(archetype_idx);
            }
        }
        None
    }
}

idx_type!(pub ArchetypeIdx);
idx_type!(pub ArchetypeEntityPos);

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct EntityLocation {
    pub(crate) idx: ArchetypeIdx,
    pub(crate) pos: ArchetypeEntityPos,
}

#[derive(PartialEq, Eq, Debug)]
pub(super) struct MissingComponentError;

#[derive(PartialEq, Eq, Debug)]
pub(super) struct ExistingComponentError;
