use crate::storages::components::ComponentTypeIdx;
use crate::storages::entities::EntityIdx;
use crate::utils;
use non_empty_vec::NonEmpty;
use std::slice::Iter;
use typed_index_collections::{TiSlice, TiVec};

pub(crate) struct ArchetypeStorage {
    type_idxs: TiVec<ArchetypeIdx, Vec<ComponentTypeIdx>>,
    entity_idxs: TiVec<ArchetypeIdx, TiVec<ArchetypeEntityPos, EntityIdx>>,
    next_idxs: TiVec<ArchetypeIdx, TiVec<ComponentTypeIdx, ArchetypeIdx>>,
    previous_idxs: TiVec<ArchetypeIdx, TiVec<ComponentTypeIdx, ArchetypeIdx>>,
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
        archetype_filter: &'a ArchetypeFilter,
    ) -> FilteredArchetypeIdxIter<'a> {
        FilteredArchetypeIdxIter {
            archetype_type_idxs: &self.type_idxs,
            archetype_idxs,
            filtered_type_idxs,
            archetype_filter,
        }
    }

    pub(super) fn add_component(
        &mut self,
        src_archetype_idx: ArchetypeIdx,
        type_idx: ComponentTypeIdx,
    ) -> Result<ArchetypeIdx, ExistingComponentError> {
        if let Some(&archetype_idx) = self.next_idxs[src_archetype_idx].get(type_idx) {
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
        utils::set_value(next_idxs, type_idx, dst_archetype_idx);
        Ok(dst_archetype_idx)
    }

    pub(super) fn delete_component(
        &mut self,
        src_archetype_idx: ArchetypeIdx,
        type_idx: ComponentTypeIdx,
    ) -> Result<ArchetypeIdx, MissingComponentError> {
        if let Some(&archetype_idx) = self.previous_idxs[src_archetype_idx].get(type_idx) {
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
        utils::set_value(previous_idxs, type_idx, dst_archetype_idx);
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
    archetype_filter: &'a ArchetypeFilter,
}

impl Iterator for FilteredArchetypeIdxIter<'_> {
    type Item = ArchetypeIdx;

    fn next(&mut self) -> Option<Self::Item> {
        Self::next_idx(
            &mut self.archetype_idxs,
            self.archetype_type_idxs,
            self.filtered_type_idxs,
            self.archetype_filter,
        )
    }
}

impl DoubleEndedIterator for FilteredArchetypeIdxIter<'_> {
    fn next_back(&mut self) -> Option<Self::Item> {
        Self::next_idx(
            (&mut self.archetype_idxs).rev(),
            self.archetype_type_idxs,
            self.filtered_type_idxs,
            self.archetype_filter,
        )
    }
}

impl FilteredArchetypeIdxIter<'_> {
    fn next_idx<'a, I>(
        archetype_idxs: I,
        archetype_type_idxs: &TiVec<ArchetypeIdx, Vec<ComponentTypeIdx>>,
        filtered_type_idxs: &[ComponentTypeIdx],
        archetype_filter: &ArchetypeFilter,
    ) -> Option<ArchetypeIdx>
    where
        I: Iterator<Item = &'a ArchetypeIdx>,
    {
        for &archetype_idx in archetype_idxs {
            let archetype_type_idxs = &archetype_type_idxs[archetype_idx];
            if !Self::contains_all_types(archetype_type_idxs, filtered_type_idxs) {
                continue;
            }
            match archetype_filter {
                ArchetypeFilter::None => return None,
                ArchetypeFilter::All => return Some(archetype_idx),
                ArchetypeFilter::Union(type_idxs) => {
                    if Self::contains_any_type(archetype_type_idxs, type_idxs) {
                        return Some(archetype_idx);
                    }
                }
                ArchetypeFilter::Intersection(type_idxs) => {
                    if Self::contains_all_types(archetype_type_idxs, type_idxs) {
                        return Some(archetype_idx);
                    }
                }
            }
        }
        None
    }
}

impl FilteredArchetypeIdxIter<'_> {
    fn contains_all_types(
        type_idxs: &[ComponentTypeIdx],
        contained_type_idxs: &[ComponentTypeIdx],
    ) -> bool {
        contained_type_idxs
            .iter()
            .all(|t| type_idxs.binary_search(t).is_ok())
    }

    fn contains_any_type(
        type_idxs: &[ComponentTypeIdx],
        contained_type_idxs: &[ComponentTypeIdx],
    ) -> bool {
        contained_type_idxs
            .iter()
            .any(|t| type_idxs.binary_search(t).is_ok())
    }
}

idx_type!(pub ArchetypeIdx);
idx_type!(pub ArchetypeEntityPos);

#[derive(Clone, PartialEq, Debug)]
pub(crate) enum ArchetypeFilter {
    None,
    All,
    Union(NonEmpty<ComponentTypeIdx>),
    Intersection(NonEmpty<ComponentTypeIdx>),
}

impl ArchetypeFilter {
    pub(crate) fn merge(self, other: Self) -> Self {
        match self {
            Self::None => other,
            Self::All => match other {
                Self::None => Self::All,
                other @ (Self::All | Self::Union(_) | Self::Intersection(_)) => other,
            },
            Self::Union(mut type_idxs) => match other {
                Self::None | Self::All => Self::Union(type_idxs),
                Self::Union(other_type_idxs) => {
                    other_type_idxs.into_iter().for_each(|t| type_idxs.push(t));
                    Self::Union(type_idxs)
                }
                Self::Intersection(other_type_idxs) => Self::Intersection(other_type_idxs),
            },
            Self::Intersection(mut type_idxs) => match other {
                Self::None | Self::All | Self::Union(_) => Self::Intersection(type_idxs),
                Self::Intersection(other_type_idxs) => {
                    other_type_idxs.into_iter().for_each(|t| type_idxs.push(t));
                    Self::Intersection(type_idxs)
                }
            },
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct EntityLocationInArchetype {
    pub(crate) idx: ArchetypeIdx,
    pub(crate) pos: ArchetypeEntityPos,
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
        assert_eq!(storage.all_sorted_idxs(), [0.into(), 1.into()]);
        let next_idx = storage.next_idxs[archetype_idx][type_idx];
        assert_eq!(next_idx, new_archetype_idx);
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
        assert_eq!(storage.all_sorted_idxs(), [0.into(), 1.into()]);
        let next_idx = storage.next_idxs[archetype1_idx][type_idx];
        assert_eq!(next_idx, new_archetype_idx);
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
        let type_idxs = storage.sorted_type_idxs(new_archetype_idx);
        assert_eq!(type_idxs, [type1_idx, type2_idx]);
        let all_sorted_idxs = [0.into(), 1.into(), 2.into(), 3.into()];
        assert_eq!(storage.all_sorted_idxs(), all_sorted_idxs);
        let next_idx = storage.next_idxs[archetype4_idx][type1_idx];
        assert_eq!(next_idx, new_archetype_idx);
    }

    #[test]
    fn add_component_with_existing_type() {
        let mut storage = ArchetypeStorage::default();
        let archetype1_idx = ArchetypeStorage::DEFAULT_IDX;
        let type_idx = 10.into();
        let archetype2_idx = storage.add_component(archetype1_idx, type_idx).unwrap();

        let new_archetype_idx = storage.add_component(archetype2_idx, type_idx);

        assert_eq!(new_archetype_idx, Err(ExistingComponentError));
        assert!(storage.next_idxs[archetype2_idx].is_empty());
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
        let previous_idx = storage.previous_idxs[archetype3_idx][type1_idx];
        assert_eq!(previous_idx, new_archetype_idx);
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
        let previous_idx = storage.previous_idxs[archetype3_idx][type1_idx];
        assert_eq!(previous_idx, new_archetype_idx);
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
        let previous_idx = storage.previous_idxs[archetype4_idx][type2_idx];
        assert_eq!(previous_idx, new_archetype_idx);
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
        assert!(storage.previous_idxs[archetype2_idx].is_empty());
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
}

#[cfg(test)]
mod filtered_archetype_idx_iter_tests {
    use super::*;

    impl<'a> FilteredArchetypeIdxIter<'a> {
        pub(crate) fn new(
            archetype_idxs: &'a [ArchetypeIdx],
            archetype_type_idxs: &'a TiVec<ArchetypeIdx, Vec<ComponentTypeIdx>>,
        ) -> Self {
            Self {
                archetype_type_idxs,
                archetype_idxs: archetype_idxs.iter(),
                filtered_type_idxs: &[],
                archetype_filter: &ArchetypeFilter::All,
            }
        }
    }

    #[test]
    fn iter_when_none_archetype_filter() {
        let type_idxs = ti_vec![vec![2.into()]];
        let archetype_idxs = vec![0.into()];

        let mut iter = FilteredArchetypeIdxIter {
            archetype_type_idxs: &type_idxs,
            archetype_idxs: archetype_idxs.iter(),
            filtered_type_idxs: &[],
            archetype_filter: &ArchetypeFilter::None,
        };

        assert_eq!(iter.next(), None);
    }

    #[test]
    fn iter_when_no_filtered_type_and_all_archetype_filter() {
        let type_idxs = ti_vec![vec![2.into()], vec![3.into()]];
        let archetype_idxs = vec![0.into(), 1.into()];

        let mut iter = FilteredArchetypeIdxIter {
            archetype_type_idxs: &type_idxs,
            archetype_idxs: archetype_idxs.iter(),
            filtered_type_idxs: &[],
            archetype_filter: &ArchetypeFilter::All,
        };

        assert_eq!(iter.next(), Some(0.into()));
        assert_eq!(iter.next(), Some(1.into()));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn iter_when_filtered_types_and_all_archetype_filter() {
        let type_idxs = ti_vec![vec![2.into()], vec![3.into()]];
        let archetype_idxs = vec![0.into(), 1.into()];

        let mut iter = FilteredArchetypeIdxIter {
            archetype_type_idxs: &type_idxs,
            archetype_idxs: archetype_idxs.iter(),
            filtered_type_idxs: &[3.into()],
            archetype_filter: &ArchetypeFilter::All,
        };

        assert_eq!(iter.next(), Some(1.into()));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn iter_when_filtered_types_and_union_filter() {
        let type_idxs = ti_vec![
            vec![2.into()],
            vec![2.into(), 3.into()],
            vec![3.into(), 4.into()]
        ];
        let archetype_idxs = vec![0.into(), 1.into(), 2.into()];

        let mut iter = FilteredArchetypeIdxIter {
            archetype_type_idxs: &type_idxs,
            archetype_idxs: archetype_idxs.iter(),
            filtered_type_idxs: &[3.into()],
            archetype_filter: &ArchetypeFilter::Union(ne_vec![2.into(), 4.into()]),
        };

        assert_eq!(iter.next(), Some(1.into()));
        assert_eq!(iter.next(), Some(2.into()));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn iter_when_filtered_types_and_intersection_filter() {
        let type_idxs = ti_vec![
            vec![2.into()],
            vec![2.into(), 3.into()],
            vec![2.into(), 3.into(), 4.into()]
        ];
        let archetype_idxs = vec![0.into(), 1.into(), 2.into()];

        let mut iter = FilteredArchetypeIdxIter {
            archetype_type_idxs: &type_idxs,
            archetype_idxs: archetype_idxs.iter(),
            filtered_type_idxs: &[3.into()],
            archetype_filter: &ArchetypeFilter::Intersection(ne_vec![2.into(), 4.into()]),
        };

        assert_eq!(iter.next(), Some(2.into()));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn iter_reversed_when_none_archetype_filter() {
        let type_idxs = ti_vec![vec![2.into()]];
        let archetype_idxs = vec![0.into()];

        let mut iter = FilteredArchetypeIdxIter {
            archetype_type_idxs: &type_idxs,
            archetype_idxs: archetype_idxs.iter(),
            filtered_type_idxs: &[],
            archetype_filter: &ArchetypeFilter::None,
        }
        .rev();

        assert_eq!(iter.next(), None);
    }

    #[test]
    fn iter_reversed_when_no_filtered_type_and_all_archetype_filter() {
        let type_idxs = ti_vec![vec![2.into()], vec![3.into()]];
        let archetype_idxs = vec![0.into(), 1.into()];

        let mut iter = FilteredArchetypeIdxIter {
            archetype_type_idxs: &type_idxs,
            archetype_idxs: archetype_idxs.iter(),
            filtered_type_idxs: &[],
            archetype_filter: &ArchetypeFilter::All,
        }
        .rev();

        assert_eq!(iter.next(), Some(1.into()));
        assert_eq!(iter.next(), Some(0.into()));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn iter_reversed_when_filtered_types_and_all_archetype_filter() {
        let type_idxs = ti_vec![vec![2.into()], vec![3.into()]];
        let archetype_idxs = vec![0.into(), 1.into()];
        let filtered_type_idxs = [3.into()];

        let mut iter = FilteredArchetypeIdxIter {
            archetype_type_idxs: &type_idxs,
            archetype_idxs: archetype_idxs.iter(),
            filtered_type_idxs: &filtered_type_idxs,
            archetype_filter: &ArchetypeFilter::All,
        }
        .rev();

        assert_eq!(iter.next(), Some(1.into()));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn iter_reversed_when_filtered_types_and_union_filter() {
        let type_idxs = ti_vec![
            vec![2.into()],
            vec![2.into(), 3.into()],
            vec![3.into(), 4.into()]
        ];
        let archetype_idxs = vec![0.into(), 1.into(), 2.into()];
        let filtered_type_idxs = [3.into()];
        let archetype_filter = ArchetypeFilter::Union(ne_vec![2.into(), 4.into()]);

        let mut iter = FilteredArchetypeIdxIter {
            archetype_type_idxs: &type_idxs,
            archetype_idxs: archetype_idxs.iter(),
            filtered_type_idxs: &filtered_type_idxs,
            archetype_filter: &archetype_filter,
        }
        .rev();

        assert_eq!(iter.next(), Some(2.into()));
        assert_eq!(iter.next(), Some(1.into()));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn iter_reversed_when_filtered_types_and_intersection_filter() {
        let type_idxs = ti_vec![
            vec![2.into()],
            vec![2.into(), 3.into()],
            vec![2.into(), 3.into(), 4.into()]
        ];
        let archetype_idxs = vec![0.into(), 1.into(), 2.into()];
        let filtered_type_idxs = [3.into()];
        let archetype_filter = ArchetypeFilter::Intersection(ne_vec![2.into(), 4.into()]);

        let mut iter = FilteredArchetypeIdxIter {
            archetype_type_idxs: &type_idxs,
            archetype_idxs: archetype_idxs.iter(),
            filtered_type_idxs: &filtered_type_idxs,
            archetype_filter: &archetype_filter,
        }
        .rev();

        assert_eq!(iter.next(), Some(2.into()));
        assert_eq!(iter.next(), None);
    }
}

#[cfg(test)]
mod archetype_filter_tests {
    use crate::storages::archetypes::ArchetypeFilter::{All, Intersection, None, Union};

    #[test]
    fn merge_none() {
        assert_eq!(None.merge(None), None);
        assert_eq!(None.merge(All), All);
        let merged = None.merge(Union(ne_vec![0.into()]));
        assert_eq!(merged, Union(ne_vec![0.into()]));
        let merged = None.merge(Intersection(ne_vec![0.into()]));
        assert_eq!(merged, Intersection(ne_vec![0.into()]));
    }

    #[test]
    fn merge_all() {
        assert_eq!(All.merge(None), All);
        assert_eq!(All.merge(All), All);
        let merged = All.merge(Union(ne_vec![0.into()]));
        assert_eq!(merged, Union(ne_vec![0.into()]));
        let merged = All.merge(Intersection(ne_vec![0.into()]));
        assert_eq!(merged, Intersection(ne_vec![0.into()]));
    }

    #[test]
    fn merge_union() {
        let merged = Union(ne_vec![1.into()]).merge(None);
        assert_eq!(merged, Union(ne_vec![1.into()]));
        let merged = Union(ne_vec![1.into()]).merge(All);
        assert_eq!(merged, Union(ne_vec![1.into()]));
        let merged = Union(ne_vec![1.into()]).merge(Union(ne_vec![0.into()]));
        assert_eq!(merged, Union(ne_vec![1.into(), 0.into()]));
        let merged = Union(ne_vec![1.into()]).merge(Intersection(ne_vec![0.into()]));
        assert_eq!(merged, Intersection(ne_vec![0.into()]));
    }

    #[test]
    fn merge_intersection() {
        let merged = Intersection(ne_vec![1.into()]).merge(None);
        assert_eq!(merged, Intersection(ne_vec![1.into()]));
        let merged = Intersection(ne_vec![1.into()]).merge(All);
        assert_eq!(merged, Intersection(ne_vec![1.into()]));
        let merged = Intersection(ne_vec![1.into()]).merge(Union(ne_vec![0.into()]));
        assert_eq!(merged, Intersection(ne_vec![1.into()]));
        let merged = Intersection(ne_vec![1.into()]).merge(Intersection(ne_vec![0.into()]));
        assert_eq!(merged, Intersection(ne_vec![1.into(), 0.into()]));
    }
}
