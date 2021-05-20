use crate::internal::entities::data::EntityLocation;

const EMPTY_IDX_SLICE: [usize; 0] = [];

#[derive(Default)]
pub(super) struct EntityStorage {
    deleted_idxs: Vec<usize>,
    next_idx: usize,
}

impl EntityStorage {
    pub(super) fn create(&mut self) -> usize {
        self.deleted_idxs.pop().unwrap_or_else(|| {
            let idx = self.next_idx;
            self.next_idx += 1;
            idx
        })
    }

    pub(super) fn delete(&mut self, entity_idx: usize) {
        if entity_idx >= self.next_idx {
            panic!("internal error: cannot delete not existing entity");
        }
        self.deleted_idxs.push(entity_idx);
    }
}

#[derive(Default)]
pub(super) struct LocationStorage(Vec<Option<EntityLocation>>);

impl LocationStorage {
    pub(super) fn get(&self, entity_idx: usize) -> Option<EntityLocation> {
        self.0.get(entity_idx).copied().flatten()
    }

    pub(super) fn set(&mut self, entity_idx: usize, location: Option<EntityLocation>) {
        (self.0.len()..=entity_idx).for_each(|_| self.0.push(None));
        self.0[entity_idx] = location;
    }

    pub(super) fn remove(&mut self, entity_idx: usize) -> Option<EntityLocation> {
        self.0.get_mut(entity_idx).and_then(Option::take)
    }
}

#[derive(Default)]
pub(super) struct ArchetypeEntityStorage(Vec<Vec<usize>>);

impl ArchetypeEntityStorage {
    pub(super) fn idxs(&self, archetype_idx: usize) -> &[usize] {
        self.0
            .get(archetype_idx)
            .map_or(&EMPTY_IDX_SLICE, |i| i.as_slice())
    }

    pub(super) fn move_(
        &mut self,
        entity_idx: usize,
        src_location: Option<EntityLocation>,
        dst_archetype_idx: Option<usize>,
    ) -> (Option<EntityLocation>, Option<usize>) {
        let moved_entity_idx = src_location.and_then(|src_location| {
            let src_archetype = &mut self.0[src_location.archetype_idx];
            src_archetype.swap_remove(src_location.entity_pos);
            src_archetype.get(src_location.entity_pos).copied()
        });
        let dst_location = dst_archetype_idx.map(|dst_archetype_idx| {
            (self.0.len()..=dst_archetype_idx).for_each(|_| self.0.push(Vec::new()));
            let dst_entity_pos = self.0[dst_archetype_idx].len();
            self.0[dst_archetype_idx].push(entity_idx);
            EntityLocation::new(dst_archetype_idx, dst_entity_pos)
        });
        (dst_location, moved_entity_idx)
    }
}

#[cfg(test)]
mod entity_storage_tests {
    use super::*;

    #[test]
    fn create_first_entity() {
        let mut storage = EntityStorage::default();

        let entity_idx = storage.create();

        assert_eq!(entity_idx, 0);
    }

    #[test]
    fn create_other_entity() {
        let mut storage = EntityStorage::default();
        storage.create();

        let entity_idx = storage.create();

        assert_eq!(entity_idx, 1);
    }

    #[test]
    #[should_panic]
    fn delete_missing_entity() {
        let mut storage = EntityStorage::default();
        storage.create();
        storage.create();

        storage.delete(2);
    }

    #[test]
    fn delete_exiting_entity() {
        let mut storage = EntityStorage::default();
        storage.create();
        storage.create();
        storage.create();

        storage.delete(1);

        assert_eq!(storage.create(), 1);
        assert_eq!(storage.create(), 3);
    }
}

#[cfg(test)]
mod location_storage_tests {
    use super::*;

    #[test]
    fn set_entity_locations() {
        let mut storage = LocationStorage::default();

        storage.set(1, Some(EntityLocation::new(10, 20)));
        storage.set(2, Some(EntityLocation::new(11, 21)));
        storage.set(2, None);

        assert_eq!(storage.get(0), None);
        assert_eq!(storage.get(1), Some(EntityLocation::new(10, 20)));
        assert_eq!(storage.get(2), None);
        assert_eq!(storage.get(3), None);
    }

    #[test]
    fn remove_missing_entity() {
        let mut storage = LocationStorage::default();

        let location = storage.remove(0);

        assert_eq!(location, None);
    }

    #[test]
    fn remove_existing_entity() {
        let mut storage = LocationStorage::default();
        storage.set(0, Some(EntityLocation::new(10, 20)));
        storage.set(1, Some(EntityLocation::new(11, 21)));

        let location = storage.remove(1);

        assert_eq!(location, Some(EntityLocation::new(11, 21)));
        assert_eq!(storage.get(0), Some(EntityLocation::new(10, 20)));
        assert_eq!(storage.get(1), None);
    }
}

#[cfg(test)]
mod archetype_entity_storage_tests {
    use super::*;

    #[test]
    fn move_entity_without_src_location_in_archetype_without_entities() {
        let mut storage = ArchetypeEntityStorage::default();

        let (dst_location, moved_entity_idx) = storage.move_(10, None, Some(1));

        assert_eq!(dst_location, Some(EntityLocation::new(1, 0)));
        assert_eq!(moved_entity_idx, None);
        assert_eq!(storage.idxs(0), []);
        assert_eq!(storage.idxs(1), [10]);
        assert_eq!(storage.idxs(2), []);
    }

    #[test]
    fn move_entity_without_src_location_in_archetype_with_entities() {
        let mut storage = ArchetypeEntityStorage::default();
        storage.move_(10, None, Some(2));

        let (dst_location, moved_entity_idx) = storage.move_(20, None, Some(2));

        assert_eq!(dst_location, Some(EntityLocation::new(2, 1)));
        assert_eq!(moved_entity_idx, None);
        assert_eq!(storage.idxs(0), []);
        assert_eq!(storage.idxs(1), []);
        assert_eq!(storage.idxs(2), [10, 20]);
    }

    #[test]
    fn move_entity_with_src_location_at_archetype_list_end() {
        let mut storage = ArchetypeEntityStorage::default();
        storage.move_(10, None, Some(1));
        storage.move_(20, None, Some(1));
        let (entity_location, _) = storage.move_(30, None, Some(1));

        let (dst_location, moved_entity_idx) = storage.move_(30, entity_location, Some(2));

        assert_eq!(dst_location, Some(EntityLocation::new(2, 0)));
        assert_eq!(moved_entity_idx, None);
        assert_eq!(storage.idxs(0), []);
        assert_eq!(storage.idxs(1), [10, 20]);
        assert_eq!(storage.idxs(2), [30]);
    }

    #[test]
    fn move_entity_with_src_location_not_at_archetype_list_end() {
        let mut storage = ArchetypeEntityStorage::default();
        storage.move_(10, None, Some(1));
        let (entity_location, _) = storage.move_(20, None, Some(1));
        storage.move_(30, None, Some(1));
        storage.move_(40, None, Some(1));

        let (dst_location, moved_entity_idx) = storage.move_(20, entity_location, Some(2));

        assert_eq!(dst_location, Some(EntityLocation::new(2, 0)));
        assert_eq!(moved_entity_idx, Some(40));
        assert_eq!(storage.idxs(0), []);
        assert_eq!(storage.idxs(1), [10, 40, 30]);
        assert_eq!(storage.idxs(2), [20]);
    }

    #[test]
    fn move_entity_with_no_dst_location() {
        let mut storage = ArchetypeEntityStorage::default();
        let (entity_location, _) = storage.move_(10, None, Some(1));
        storage.move_(20, None, Some(1));
        storage.move_(30, None, Some(1));

        let (dst_location, moved_entity_idx) = storage.move_(10, entity_location, None);

        assert_eq!(dst_location, None);
        assert_eq!(moved_entity_idx, Some(30));
        assert_eq!(storage.idxs(0), []);
        assert_eq!(storage.idxs(1), [30, 20]);
    }
}
