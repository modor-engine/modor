use crate::internal::archetypes::data::{ExistingComponentError, MissingComponentError};
use fxhash::FxHashMap;
use std::mem;
use std::num::NonZeroUsize;

#[derive(Default)]
pub(super) struct PropertyStorage {
    properties: Vec<Option<(Vec<usize>, NonZeroUsize)>>,
    deleted_archetype_idxs: Vec<usize>,
}

impl PropertyStorage {
    pub(super) fn type_idxs(&self, archetype_idx: usize) -> &[usize] {
        &self.properties[archetype_idx]
            .as_ref()
            .expect("internal error: component types retrieved for deleted archetype")
            .0
    }

    pub(super) fn group_idx(&self, archetype_idx: usize) -> NonZeroUsize {
        self.properties[archetype_idx]
            .as_ref()
            .expect("internal error: groups retrieved for deleted archetype")
            .1
    }

    pub(super) fn next_idx(
        &self,
        group_idx: NonZeroUsize,
        archetype_idx: Option<usize>,
        type_idx: usize,
    ) -> Result<Option<usize>, ExistingComponentError> {
        let type_idxs = if let Some(archetype_idx) = archetype_idx {
            self.next_type_idxs(archetype_idx, type_idx)?
        } else {
            vec![type_idx]
        };
        Ok(self
            .properties
            .iter()
            .map(Option::as_ref)
            .position(|p| p.map_or(false, |p| p.0 == type_idxs && p.1 == group_idx)))
    }

    #[allow(clippy::option_option)]
    pub(super) fn previous_idx(
        &self,
        group_idx: NonZeroUsize,
        archetype_idx: usize,
        type_idx: usize,
    ) -> Result<Option<Option<usize>>, MissingComponentError> {
        let type_idxs = self.previous_type_idxs(archetype_idx, type_idx)?;
        Ok(if type_idxs.is_empty() {
            Some(None)
        } else {
            self.properties
                .iter()
                .map(Option::as_ref)
                .position(|p| p.map_or(false, |p| p.0 == type_idxs && p.1 == group_idx))
                .map(Some)
        })
    }

    pub(super) fn create_next(
        &mut self,
        group_idx: NonZeroUsize,
        archetype_idx: Option<usize>,
        type_idx: usize,
    ) -> Result<usize, ExistingComponentError> {
        let type_idxs = if let Some(archetype_idx) = archetype_idx {
            self.next_type_idxs(archetype_idx, type_idx)?
        } else {
            vec![type_idx]
        };
        let new_archetype_idx = self.generate_archetype_idx();
        self.properties[new_archetype_idx] = Some((type_idxs, group_idx));
        Ok(new_archetype_idx)
    }

    pub(super) fn create_previous(
        &mut self,
        group_idx: NonZeroUsize,
        archetype_idx: usize,
        type_idx: usize,
    ) -> Result<Option<usize>, MissingComponentError> {
        let type_idxs = self.previous_type_idxs(archetype_idx, type_idx)?;
        Ok(if type_idxs.is_empty() {
            None
        } else {
            let new_archetype_idx = self.generate_archetype_idx();
            self.properties[new_archetype_idx] = Some((type_idxs, group_idx));
            Some(new_archetype_idx)
        })
    }

    pub(super) fn delete(&mut self, archetype_idx: usize) {
        self.properties[archetype_idx] = None;
        self.deleted_archetype_idxs.push(archetype_idx);
    }

    fn next_type_idxs(
        &self,
        archetype_idx: usize,
        type_idx: usize,
    ) -> Result<Vec<usize>, ExistingComponentError> {
        let mut type_idxs = self.type_idxs(archetype_idx).to_vec();
        if let Err(pos) = type_idxs.binary_search(&type_idx) {
            type_idxs.insert(pos, type_idx);
            Ok(type_idxs)
        } else {
            Err(ExistingComponentError)
        }
    }

    fn previous_type_idxs(
        &self,
        archetype_idx: usize,
        type_idx: usize,
    ) -> Result<Vec<usize>, MissingComponentError> {
        let mut type_idxs = self.type_idxs(archetype_idx).to_vec();
        type_idxs
            .binary_search(&type_idx)
            .map(|pos| {
                type_idxs.remove(pos);
                type_idxs
            })
            .map_err(|_| MissingComponentError)
    }

    fn generate_archetype_idx(&mut self) -> usize {
        let new_archetype_idx = self
            .deleted_archetype_idxs
            .pop()
            .unwrap_or_else(|| self.properties.len());
        (self.properties.len()..=new_archetype_idx).for_each(|_| self.properties.push(None));
        new_archetype_idx
    }
}

#[derive(Default)]
pub(super) struct GroupArchetypeStorage(Vec<Vec<usize>>);

impl GroupArchetypeStorage {
    pub(super) fn idxs(&self, group_idx: NonZeroUsize) -> impl Iterator<Item = usize> + '_ {
        let group_pos = group_idx.get() - 1;
        self.0
            .get(group_pos)
            .into_iter()
            .flat_map(|i| i.iter())
            .copied()
    }

    pub(super) fn add(&mut self, group_idx: NonZeroUsize, archetype_idx: usize) {
        let group_pos = group_idx.get() - 1;
        (self.0.len()..=group_pos).for_each(|_| self.0.push(Vec::new()));
        self.0[group_pos].push(archetype_idx);
    }

    pub(super) fn remove(&mut self, group_idx: NonZeroUsize) -> Vec<usize> {
        let group_pos = group_idx.get() - 1;
        self.0.get_mut(group_pos).map_or_else(Vec::new, mem::take)
    }
}

#[derive(Default)]
pub(super) struct TypeArchetypeStorage(Vec<Vec<usize>>);

impl TypeArchetypeStorage {
    pub(super) fn idxs(&self, type_idxs: &[usize]) -> Vec<usize> {
        if let Some((&reference_type_idx, other_type_idxs)) = type_idxs.split_first() {
            let mut archetype_idxs = self.type_archetype_idxs(reference_type_idx).to_vec();
            for &type_idx in other_type_idxs {
                let other_archetype_idxs = self.type_archetype_idxs(type_idx);
                for archetype_pos in (0..archetype_idxs.len()).rev() {
                    let archetype_idx = archetype_idxs[archetype_pos];
                    if !other_archetype_idxs.contains(&archetype_idx) {
                        archetype_idxs.swap_remove(archetype_pos);
                    }
                }
            }
            archetype_idxs
        } else {
            Vec::new()
        }
    }

    pub(super) fn add(&mut self, type_idx: usize, archetype_idx: usize) {
        (self.0.len()..=type_idx).for_each(|_| self.0.push(Vec::new()));
        self.0[type_idx].push(archetype_idx);
    }

    pub(super) fn delete(&mut self, type_idx: usize, archetype_idx: usize) {
        let archetype_idxs = &mut self.0[type_idx];
        let archetype_pos = archetype_idxs
            .iter()
            .position(|&a| a == archetype_idx)
            .expect("internal error: delete not existing archetype from component types");
        archetype_idxs.swap_remove(archetype_pos);
    }

    fn type_archetype_idxs(&self, type_idx: usize) -> &[usize] {
        self.0.get(type_idx).map_or(&[] as _, Vec::as_slice)
    }
}

#[derive(Default)]
pub(super) struct NextArchetypeStorage(Vec<FxHashMap<(Option<usize>, usize), usize>>);

impl NextArchetypeStorage {
    pub(super) fn idx(
        &self,
        group_idx: NonZeroUsize,
        archetype_idx: Option<usize>,
        type_idx: usize,
    ) -> Option<usize> {
        let group_pos = group_idx.get() - 1;
        self.0
            .get(group_pos)
            .and_then(|a| a.get(&(archetype_idx, type_idx)))
            .copied()
    }

    pub(super) fn add(
        &mut self,
        group_idx: NonZeroUsize,
        archetype_idx: Option<usize>,
        type_idx: usize,
        next_archetype_idx: usize,
    ) {
        let group_pos = group_idx.get() - 1;
        (self.0.len()..=group_pos).for_each(|_| self.0.push(FxHashMap::default()));
        self.0[group_pos].insert((archetype_idx, type_idx), next_archetype_idx);
    }

    pub(super) fn delete(&mut self, group_idx: NonZeroUsize) {
        let group_pos = group_idx.get() - 1;
        if let Some(archetype_idxs) = self.0.get_mut(group_pos) {
            *archetype_idxs = FxHashMap::default();
        }
    }
}

#[derive(Default)]
pub(super) struct PreviousArchetypeStorage(Vec<FxHashMap<(usize, usize), Option<usize>>>);

impl PreviousArchetypeStorage {
    #[allow(clippy::option_option)]
    pub(super) fn idx(
        &self,
        group_idx: NonZeroUsize,
        archetype_idx: usize,
        type_idx: usize,
    ) -> Option<Option<usize>> {
        let group_pos = group_idx.get() - 1;
        self.0
            .get(group_pos)
            .and_then(|a| a.get(&(archetype_idx, type_idx)))
            .copied()
    }

    pub(super) fn add(
        &mut self,
        group_idx: NonZeroUsize,
        archetype_idx: usize,
        type_idx: usize,
        previous_archetype_idx: Option<usize>,
    ) {
        let group_pos = group_idx.get() - 1;
        (self.0.len()..=group_pos).for_each(|_| self.0.push(FxHashMap::default()));
        self.0[group_pos].insert((archetype_idx, type_idx), previous_archetype_idx);
    }

    pub(super) fn delete(&mut self, group_idx: NonZeroUsize) {
        let group_pos = group_idx.get() - 1;
        if let Some(archetype_idxs) = self.0.get_mut(group_pos) {
            *archetype_idxs = FxHashMap::default();
        }
    }
}

#[cfg(test)]
mod property_storage_tests {
    use super::*;
    use std::convert::TryInto;

    #[test]
    fn create_next_archetype_from_no_archetype() {
        let mut storage = PropertyStorage::default();
        let group_idx = 1.try_into().unwrap();

        let archetype_idx = storage.create_next(group_idx, None, 2);

        assert_eq!(archetype_idx, Ok(0));
        assert_eq!(storage.type_idxs(0), [2]);
        assert_eq!(storage.group_idx(0), group_idx);
        assert_panics!(storage.type_idxs(1));
        assert_panics!(storage.group_idx(1));
        assert_eq!(storage.next_idx(group_idx, None, 2), Ok(Some(0)));
        assert_eq!(storage.next_idx(group_idx, None, 3), Ok(None));
        let error = Err(ExistingComponentError);
        assert_eq!(storage.next_idx(group_idx, Some(0), 2), error);
        assert_eq!(storage.next_idx(3.try_into().unwrap(), None, 2), Ok(None));
        assert_panics!(storage.next_idx(group_idx, Some(3), 2));
        assert_eq!(storage.previous_idx(group_idx, 0, 2), Ok(Some(None)));
        assert_panics!(storage.previous_idx(group_idx, 1, 2));
        let error = Err(MissingComponentError);
        assert_eq!(storage.previous_idx(group_idx, 0, 3), error);
    }

    #[test]
    #[should_panic]
    fn create_next_archetype_from_missing_archetype() {
        let mut storage = PropertyStorage::default();

        let _ = storage.create_next(1.try_into().unwrap(), Some(0), 2);
    }

    #[test]
    fn create_next_archetype_from_existing_archetype_using_same_type() {
        let mut storage = PropertyStorage::default();
        let _ = storage.create_next(1.try_into().unwrap(), None, 2);

        let archetype_idx = storage.create_next(1.try_into().unwrap(), Some(0), 2);

        assert_eq!(archetype_idx, Err(ExistingComponentError));
    }

    #[test]
    fn create_next_archetype_from_existing_archetype_using_different_type() {
        let mut storage = PropertyStorage::default();
        let group_idx = 1.try_into().unwrap();
        let _ = storage.create_next(group_idx, None, 2);

        let archetype_idx = storage.create_next(group_idx, Some(0), 5);

        assert_eq!(archetype_idx, Ok(1));
        assert_eq!(storage.type_idxs(1), [2, 5]);
        assert_eq!(storage.group_idx(1), group_idx);
        assert_eq!(storage.next_idx(group_idx, Some(0), 5), Ok(Some(1)));
        assert_eq!(storage.previous_idx(group_idx, 1, 5), Ok(Some(Some(0))));
        assert_eq!(storage.previous_idx(group_idx, 1, 2), Ok(None));
        assert_eq!(storage.previous_idx(3.try_into().unwrap(), 1, 2), Ok(None));
    }

    #[test]
    fn create_previous_archetype_from_existing_archetype_using_missing_type() {
        let mut storage = PropertyStorage::default();
        let group_idx = 1.try_into().unwrap();
        let _ = storage.create_next(group_idx, None, 2);

        let archetype_idx = storage.create_previous(group_idx, 0, 3);

        assert_eq!(archetype_idx, Err(MissingComponentError));
    }

    #[test]
    fn create_previous_archetype_from_existing_archetype_using_only_type() {
        let mut storage = PropertyStorage::default();
        let group_idx = 1.try_into().unwrap();
        let _ = storage.create_next(group_idx, None, 2);

        let archetype_idx = storage.create_previous(group_idx, 0, 2);

        assert_eq!(archetype_idx, Ok(None));
        assert_eq!(storage.previous_idx(group_idx, 0, 2), Ok(Some(None)));
    }

    #[test]
    fn create_previous_archetype_from_existing_archetype_using_one_of_types() {
        let mut storage = PropertyStorage::default();
        let group_idx = 1.try_into().unwrap();
        let _ = storage.create_next(group_idx, None, 2);
        let _ = storage.create_next(group_idx, Some(0), 3);

        let archetype_idx = storage.create_previous(group_idx, 1, 2);

        assert_eq!(archetype_idx, Ok(Some(2)));
        assert_eq!(storage.previous_idx(group_idx, 1, 2), Ok(Some(Some(2))));
        assert_eq!(storage.next_idx(group_idx, Some(2), 2), Ok(Some(1)));
    }

    #[test]
    #[should_panic]
    fn delete_missing_archetype() {
        let mut storage = PropertyStorage::default();

        storage.delete(0);
    }

    #[test]
    fn delete_existing_archetype() {
        let mut storage = PropertyStorage::default();
        let group_idx = 1.try_into().unwrap();
        let _ = storage.create_next(group_idx, None, 2);
        let _ = storage.create_next(group_idx, Some(0), 3);

        storage.delete(0);

        assert_panics!(storage.type_idxs(0));
        assert_panics!(storage.group_idx(0));
        assert_eq!(storage.previous_idx(group_idx, 1, 3), Ok(None));
        assert_eq!(storage.next_idx(group_idx, None, 2), Ok(None));
        assert_eq!(storage.create_next(group_idx, None, 3), Ok(0));
        assert_eq!(storage.create_next(group_idx, None, 4), Ok(2));
        assert_eq!(storage.type_idxs(0), [3]);
        assert_eq!(storage.type_idxs(1), [2, 3]);
        assert_eq!(storage.type_idxs(2), [4]);
    }
}

#[cfg(test)]
mod group_archetype_storage_tests {
    use std::convert::TryInto;

    use super::*;

    #[test]
    fn add_archetypes() {
        let mut storage = GroupArchetypeStorage::default();

        storage.add(3.try_into().unwrap(), 4);
        storage.add(3.try_into().unwrap(), 5);
        storage.add(2.try_into().unwrap(), 6);

        assert_eq!(storage.idxs(1.try_into().unwrap()).next(), None);
        assert_iter!(storage.idxs(2.try_into().unwrap()), [6]);
        assert_iter!(storage.idxs(3.try_into().unwrap()), [4, 5]);
        assert_eq!(storage.idxs(4.try_into().unwrap()).next(), None);
    }

    #[test]
    fn remove_archetype_for_missing_group() {
        let mut storage = GroupArchetypeStorage::default();

        let archetype_idx = storage.remove(1.try_into().unwrap());

        assert_eq!(archetype_idx, Vec::new());
    }

    #[test]
    fn remove_archetype_for_existing_group() {
        let mut storage = GroupArchetypeStorage::default();
        storage.add(1.try_into().unwrap(), 3);
        storage.add(2.try_into().unwrap(), 4);

        let archetype_idx = storage.remove(1.try_into().unwrap());

        assert_eq!(archetype_idx, vec![3]);
        assert_eq!(storage.idxs(1.try_into().unwrap()).next(), None);
        assert_iter!(storage.idxs(2.try_into().unwrap()), [4]);
    }
}

#[cfg(test)]
mod type_archetype_storage_tests {
    use super::*;

    #[test]
    fn add_archetypes() {
        let mut storage = TypeArchetypeStorage::default();

        storage.add(1, 4);
        storage.add(1, 5);
        storage.add(1, 6);
        storage.add(1, 7);
        storage.add(2, 7);
        storage.add(2, 5);
        storage.add(3, 4);
        storage.add(3, 5);
        storage.add(3, 7);

        assert_eq!(storage.idxs(&[]), vec![]);
        assert_eq!(storage.idxs(&[0]), vec![]);
        assert_eq!(storage.idxs(&[1]), vec![4, 5, 6, 7]);
        assert_eq!(storage.idxs(&[2]), vec![7, 5]);
        assert_eq!(storage.idxs(&[3]), vec![4, 5, 7]);
        assert_eq!(storage.idxs(&[1, 3]), vec![4, 5, 7]);
        assert_eq!(storage.idxs(&[1, 2, 3]), vec![7, 5]);
        assert_eq!(storage.idxs(&[3, 2, 1]), vec![7, 5]);
        assert_eq!(storage.idxs(&[0, 1]), vec![]);
    }

    #[test]
    #[should_panic]
    fn delete_archetype_from_missing_type() {
        let mut storage = TypeArchetypeStorage::default();
        storage.add(0, 1);

        storage.delete(1, 2);
    }

    #[test]
    #[should_panic]
    fn delete_missing_archetype_from_existing_type() {
        let mut storage = TypeArchetypeStorage::default();
        storage.add(0, 1);

        storage.delete(0, 2);
    }

    #[test]
    fn delete_existing_archetype() {
        let mut storage = TypeArchetypeStorage::default();
        storage.add(1, 4);
        storage.add(1, 5);
        storage.add(1, 6);
        storage.add(1, 7);
        storage.add(2, 7);
        storage.add(2, 5);

        storage.delete(1, 5);

        assert_eq!(storage.idxs(&[]), vec![]);
        assert_eq!(storage.idxs(&[0]), vec![]);
        assert_eq!(storage.idxs(&[1]), vec![4, 7, 6]);
        assert_eq!(storage.idxs(&[2]), vec![7, 5]);
        assert_eq!(storage.idxs(&[1, 2]), vec![7]);
    }
}

#[cfg(test)]
mod next_archetype_storage_tests {
    use super::*;
    use std::convert::TryInto;

    #[test]
    fn add_next_archetypes() {
        let mut storage = NextArchetypeStorage::default();

        storage.add(1.try_into().unwrap(), Some(1), 2, 3);
        storage.add(2.try_into().unwrap(), None, 4, 5);

        assert_eq!(storage.idx(1.try_into().unwrap(), Some(1), 2), Some(3));
        assert_eq!(storage.idx(2.try_into().unwrap(), None, 4), Some(5));
        assert_eq!(storage.idx(1.try_into().unwrap(), Some(2), 3), None);
    }

    #[test]
    fn delete_missing_group() {
        let mut storage = NextArchetypeStorage::default();

        storage.delete(1.try_into().unwrap());

        assert_eq!(storage.idx(1.try_into().unwrap(), None, 2), None);
    }

    #[test]
    fn delete_existing_group() {
        let mut storage = NextArchetypeStorage::default();
        storage.add(1.try_into().unwrap(), Some(3), 4, 5);
        storage.add(2.try_into().unwrap(), Some(6), 7, 8);

        storage.delete(1.try_into().unwrap());

        assert_eq!(storage.idx(1.try_into().unwrap(), Some(3), 4), None);
        assert_eq!(storage.idx(2.try_into().unwrap(), Some(6), 7), Some(8));
    }
}

#[cfg(test)]
mod previous_archetype_storage_tests {
    use super::*;
    use std::convert::TryInto;

    #[test]
    fn add_previous_archetypes() {
        let mut storage = PreviousArchetypeStorage::default();

        storage.add(1.try_into().unwrap(), 1, 2, Some(3));
        storage.add(2.try_into().unwrap(), 4, 5, None);

        assert_eq!(storage.idx(1.try_into().unwrap(), 1, 2), Some(Some(3)));
        assert_eq!(storage.idx(2.try_into().unwrap(), 4, 5), Some(None));
        assert_eq!(storage.idx(1.try_into().unwrap(), 6, 7), None);
    }

    #[test]
    fn delete_missing_group() {
        let mut storage = PreviousArchetypeStorage::default();

        storage.delete(1.try_into().unwrap());

        assert_eq!(storage.idx(1.try_into().unwrap(), 1, 2), None);
    }

    #[test]
    fn delete_existing_group() {
        let mut storage = PreviousArchetypeStorage::default();
        storage.add(1.try_into().unwrap(), 3, 4, Some(5));
        storage.add(2.try_into().unwrap(), 6, 7, Some(8));

        storage.delete(1.try_into().unwrap());

        assert_eq!(storage.idx(1.try_into().unwrap(), 3, 4), None);
        assert_eq!(storage.idx(2.try_into().unwrap(), 6, 7), Some(Some(8)));
    }
}
