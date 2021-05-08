use crate::internal::system_state::data::{SystemLocation, TypeState};
use crate::TypeAccess;
use fxhash::FxHashMap;
use std::any::TypeId;
use std::num::NonZeroUsize;

#[derive(Default)]
pub(super) struct SystemTypesStorage(Vec<Vec<Vec<TypeAccess>>>);

impl SystemTypesStorage {
    pub(super) fn get(&self, group_idx: usize, system_idx: usize) -> &[TypeAccess] {
        &self.0[group_idx][system_idx]
    }

    pub(super) fn set(&mut self, group_idx: usize, system_idx: usize, types: Vec<TypeAccess>) {
        (self.0.len()..=group_idx).for_each(|_| self.0.push(Vec::new()));
        (self.0[group_idx].len()..=system_idx).for_each(|_| self.0[group_idx].push(Vec::new()));
        self.0[group_idx][system_idx] = types;
    }

    pub(super) fn delete(&mut self, group_idx: NonZeroUsize) {
        if let Some(systems) = self.0.get_mut(group_idx.get()) {
            *systems = Vec::new();
        }
    }
}

#[derive(Default)]
pub(super) struct SystemActionsStorage(Vec<Vec<bool>>);

impl SystemActionsStorage {
    pub(super) fn has_actions(&self, group_idx: usize, system_idx: usize) -> bool {
        self.0[group_idx][system_idx]
    }

    pub(super) fn set(&mut self, group_idx: usize, system_idx: usize, actions: bool) {
        (self.0.len()..=group_idx).for_each(|_| self.0.push(Vec::new()));
        (self.0[group_idx].len()..=system_idx).for_each(|_| self.0[group_idx].push(false));
        self.0[group_idx][system_idx] = actions;
    }

    pub(super) fn delete(&mut self, group_idx: NonZeroUsize) {
        if let Some(systems) = self.0.get_mut(group_idx.get()) {
            *systems = Vec::new();
        }
    }
}

#[derive(Default)]
pub(super) struct TypeStateStorage(FxHashMap<TypeId, TypeState>);

impl TypeStateStorage {
    pub(super) fn can_be_locked(&self, types: &[TypeAccess]) -> bool {
        types.iter().all(|t| match t {
            TypeAccess::Read(type_id) => {
                let status = &self.0[type_id];
                matches!(status, TypeState::Free) || matches!(status, TypeState::Read(_))
            }
            TypeAccess::Write(type_id) => {
                let status = &self.0[type_id];
                matches!(status, TypeState::Free)
            }
        })
    }

    pub(super) fn add(&mut self, type_id: TypeId) {
        self.0.insert(type_id, TypeState::Free);
    }

    pub(super) fn lock(&mut self, types: &[TypeAccess]) {
        types.iter().for_each(|t| match t {
            TypeAccess::Read(type_id) => {
                let status = self.0.get_mut(type_id).unwrap();
                if let TypeState::Free = status {
                    *status = TypeState::Read(0);
                } else if let TypeState::Read(count) = status {
                    *count += 1;
                } else if let TypeState::Written = status {
                    panic!("internal error: lock read type with write")
                }
            }
            TypeAccess::Write(type_id) => {
                let status = self.0.get_mut(type_id).unwrap();
                if let TypeState::Free = status {
                    *status = TypeState::Written;
                } else {
                    panic!("internal error: lock not free type with write")
                }
            }
        })
    }

    pub(super) fn unlock(&mut self, types: &[TypeAccess]) {
        types.iter().for_each(|t| match t {
            TypeAccess::Read(type_id) => {
                let status = self.0.get_mut(type_id).unwrap();
                if let TypeState::Read(0) = status {
                    *status = TypeState::Free;
                } else if let TypeState::Read(count) = status {
                    *count -= 1;
                } else {
                    panic!("internal error: unlock not read type with read")
                }
            }
            TypeAccess::Write(type_id) => {
                let status = self.0.get_mut(type_id).unwrap();
                if let TypeState::Written = status {
                    *status = TypeState::Free;
                } else {
                    panic!("internal error: unlock not written type with write")
                }
            }
        })
    }
}

#[derive(Default)]
pub(super) struct ActionStateStorage(bool);

impl ActionStateStorage {
    pub(super) fn can_be_locked(&self, actions: bool) -> bool {
        !(self.0 && actions)
    }

    pub(super) fn lock(&mut self, actions: bool) {
        if self.0 && actions {
            panic!("internal error: actions cannot be locked twice");
        }
        self.0 = self.0 || actions;
    }

    pub(super) fn unlock(&mut self, actions: bool) {
        if !self.0 && actions {
            panic!("internal error: already unlocked actions cannot be unlock");
        } else if self.0 == actions {
            self.0 = false;
        }
    }
}

#[derive(Default)]
pub(super) struct SystemsToRunStorage(Vec<Vec<bool>>);

impl SystemsToRunStorage {
    pub(super) fn is_empty(&self) -> bool {
        self.0
            .iter()
            .all(|states| states.iter().all(|&state| !state))
    }

    pub(super) fn iter(&self) -> impl Iterator<Item = SystemLocation> + '_ {
        self.0
            .iter()
            .enumerate()
            .flat_map(|(group_idx, states)| {
                states
                    .iter()
                    .enumerate()
                    .map(move |(system_idx, &state)| (group_idx, system_idx, state))
            })
            .filter(|(_, _, state)| *state)
            .map(|(group_idx, system_idx, _)| SystemLocation::new(group_idx, system_idx))
    }

    pub(super) fn add(&mut self, group_idx: usize, system_idx: usize) {
        (self.0.len()..=group_idx).for_each(|_| self.0.push(Vec::new()));
        (self.0[group_idx].len()..=system_idx).for_each(|_| self.0[group_idx].push(true));
    }

    pub(super) fn delete(&mut self, group_idx: NonZeroUsize) {
        if let Some(systems) = self.0.get_mut(group_idx.get()) {
            *systems = Vec::new();
        }
    }

    pub(super) fn set_as_run(&mut self, system_location: SystemLocation) {
        self.0[system_location.group_idx][system_location.system_idx] = false;
    }

    pub(super) fn reset(&mut self) {
        for group in &mut self.0 {
            for system_state in group {
                *system_state = true;
            }
        }
    }
}

#[cfg(test)]
mod tests_system_type_storage {
    use super::*;
    use std::convert::TryInto;

    #[test]
    fn set_system_types() {
        let mut storage = SystemTypesStorage::default();

        storage.set(2, 3, vec![TypeAccess::Read(TypeId::of::<u32>())]);

        assert_eq!(storage.get(2, 3), [TypeAccess::Read(TypeId::of::<u32>())]);
        assert_eq!(storage.get(2, 2), []);
        assert_panics!(storage.get(1, 3));
    }

    #[test]
    fn delete_nonexisting_group() {
        let mut storage = SystemTypesStorage::default();
        storage.set(1, 2, vec![TypeAccess::Read(TypeId::of::<u32>())]);

        storage.delete(2.try_into().unwrap());

        assert_eq!(storage.get(1, 2), [TypeAccess::Read(TypeId::of::<u32>())]);
    }

    #[test]
    fn delete_existing_group() {
        let mut storage = SystemTypesStorage::default();
        storage.set(1, 2, vec![TypeAccess::Read(TypeId::of::<i64>())]);
        storage.set(2, 3, vec![TypeAccess::Read(TypeId::of::<u32>())]);

        storage.delete(1.try_into().unwrap());

        assert_eq!(storage.get(2, 3), [TypeAccess::Read(TypeId::of::<u32>())]);
        assert_panics!(storage.get(1, 2));
    }
}

#[cfg(test)]
mod tests_system_actions_storage {
    use super::*;
    use std::convert::TryInto;

    #[test]
    fn set_system_actions() {
        let mut storage = SystemActionsStorage::default();

        storage.set(2, 3, true);

        assert!(storage.has_actions(2, 3));
        assert!(!storage.has_actions(2, 2));
        assert_panics!(storage.has_actions(1, 3));
    }

    #[test]
    fn delete_nonexisting_group() {
        let mut storage = SystemActionsStorage::default();
        storage.set(1, 2, true);

        storage.delete(2.try_into().unwrap());

        assert!(storage.has_actions(1, 2));
    }

    #[test]
    fn delete_existing_group() {
        let mut storage = SystemActionsStorage::default();
        storage.set(1, 2, false);
        storage.set(2, 3, true);

        storage.delete(1.try_into().unwrap());

        assert!(storage.has_actions(2, 3));
        assert_panics!(storage.has_actions(1, 2));
    }
}

#[cfg(test)]
mod tests_type_state_storage {
    use super::*;

    #[test]
    fn add_types() {
        let mut storage = TypeStateStorage::default();
        let type1_id = TypeId::of::<u32>();
        let type2_id = TypeId::of::<i64>();

        storage.add(type1_id);
        storage.add(type2_id);

        let type1_write = TypeAccess::Write(type1_id);
        let type2_read = TypeAccess::Read(type2_id);
        assert!(storage.can_be_locked(&[TypeAccess::Read(type1_id)]));
        assert!(storage.can_be_locked(&[type1_write]));
        assert!(storage.can_be_locked(&[type2_read]));
        assert!(storage.can_be_locked(&[TypeAccess::Write(type2_id)]));
        assert!(storage.can_be_locked(&[type1_write, type2_read]));
        assert_panics!(storage.can_be_locked(&[TypeAccess::Read(TypeId::of::<String>())]));
        assert_panics!(storage.can_be_locked(&[TypeAccess::Write(TypeId::of::<String>())]));
    }

    #[test]
    #[should_panic]
    fn lock_nonexisting_type_for_read() {
        let mut storage = TypeStateStorage::default();
        let type1_id = TypeId::of::<u32>();
        let type2_id = TypeId::of::<i64>();
        storage.add(type1_id);

        storage.lock(&[TypeAccess::Read(type2_id)]);
    }

    #[test]
    fn lock_free_type_for_read() {
        let mut storage = TypeStateStorage::default();
        let type1_id = TypeId::of::<u32>();
        let type2_id = TypeId::of::<i64>();
        storage.add(type1_id);
        storage.add(type2_id);

        storage.lock(&[TypeAccess::Read(type1_id)]);

        let type1_write = TypeAccess::Write(type1_id);
        let type2_read = TypeAccess::Read(type2_id);
        assert!(storage.can_be_locked(&[TypeAccess::Read(type1_id)]));
        assert!(!storage.can_be_locked(&[type1_write]));
        assert!(storage.can_be_locked(&[type2_read]));
        assert!(storage.can_be_locked(&[TypeAccess::Write(type2_id)]));
        assert!(!storage.can_be_locked(&[type1_write, type2_read]));
    }

    #[test]
    fn lock_read_type_for_read() {
        let mut storage = TypeStateStorage::default();
        let type1_id = TypeId::of::<u32>();
        let type2_id = TypeId::of::<i64>();
        storage.add(type1_id);
        storage.add(type2_id);
        storage.lock(&[TypeAccess::Read(type1_id)]);

        storage.lock(&[TypeAccess::Read(type1_id)]);

        let type1_write = TypeAccess::Write(type1_id);
        let type2_read = TypeAccess::Read(type2_id);
        assert!(storage.can_be_locked(&[TypeAccess::Read(type1_id)]));
        assert!(!storage.can_be_locked(&[type1_write]));
        assert!(storage.can_be_locked(&[type2_read]));
        assert!(storage.can_be_locked(&[TypeAccess::Write(type2_id)]));
        assert!(!storage.can_be_locked(&[type1_write, type2_read]));
    }

    #[test]
    #[should_panic]
    fn lock_written_type_for_read() {
        let mut storage = TypeStateStorage::default();
        let type1_id = TypeId::of::<u32>();
        storage.add(type1_id);
        storage.lock(&[TypeAccess::Write(type1_id)]);

        storage.lock(&[TypeAccess::Read(type1_id)]);
    }

    #[test]
    #[should_panic]
    fn lock_nonexisting_type_for_write() {
        let mut storage = TypeStateStorage::default();
        let type1_id = TypeId::of::<u32>();
        let type2_id = TypeId::of::<i64>();
        storage.add(type1_id);

        storage.lock(&[TypeAccess::Write(type2_id)]);
    }

    #[test]
    fn lock_free_type_for_write() {
        let mut storage = TypeStateStorage::default();
        let type1_id = TypeId::of::<u32>();
        let type2_id = TypeId::of::<i64>();
        storage.add(type1_id);
        storage.add(type2_id);

        storage.lock(&[TypeAccess::Write(type1_id)]);

        assert!(!storage.can_be_locked(&[TypeAccess::Read(type1_id)]));
        assert!(!storage.can_be_locked(&[TypeAccess::Write(type1_id)]));
        assert!(storage.can_be_locked(&[TypeAccess::Read(type2_id)]));
        assert!(storage.can_be_locked(&[TypeAccess::Write(type2_id)]));
    }

    #[test]
    #[should_panic]
    fn lock_read_type_for_write() {
        let mut storage = TypeStateStorage::default();
        let type1_id = TypeId::of::<u32>();
        storage.add(type1_id);
        storage.lock(&[TypeAccess::Read(type1_id)]);

        storage.lock(&[TypeAccess::Write(type1_id)]);
    }

    #[test]
    #[should_panic]
    fn lock_written_type_for_write() {
        let mut storage = TypeStateStorage::default();
        let type1_id = TypeId::of::<u32>();
        storage.add(type1_id);
        storage.lock(&[TypeAccess::Write(type1_id)]);

        storage.lock(&[TypeAccess::Write(type1_id)]);
    }

    #[test]
    fn lock_multiple_types() {
        let mut storage = TypeStateStorage::default();
        let type1_id = TypeId::of::<u32>();
        let type2_id = TypeId::of::<i64>();
        storage.add(type1_id);
        storage.add(type2_id);

        storage.lock(&[TypeAccess::Read(type1_id), TypeAccess::Write(type2_id)]);

        assert!(storage.can_be_locked(&[TypeAccess::Read(type1_id)]));
        assert!(!storage.can_be_locked(&[TypeAccess::Write(type1_id)]));
        assert!(!storage.can_be_locked(&[TypeAccess::Read(type2_id)]));
        assert!(!storage.can_be_locked(&[TypeAccess::Write(type2_id)]));
    }

    #[test]
    #[should_panic]
    fn unlock_nonexisting_type_for_read() {
        let mut storage = TypeStateStorage::default();
        let type1_id = TypeId::of::<u32>();
        let type2_id = TypeId::of::<i64>();
        storage.add(type1_id);

        storage.unlock(&[TypeAccess::Read(type2_id)]);
    }

    #[test]
    #[should_panic]
    fn unlock_free_type_for_read() {
        let mut storage = TypeStateStorage::default();
        let type1_id = TypeId::of::<u32>();
        storage.add(type1_id);

        storage.unlock(&[TypeAccess::Read(type1_id)]);
    }

    #[test]
    fn unlock_one_time_read_type_for_read() {
        let mut storage = TypeStateStorage::default();
        let type1_id = TypeId::of::<u32>();
        let type2_id = TypeId::of::<i64>();
        storage.add(type1_id);
        storage.add(type2_id);
        storage.lock(&[TypeAccess::Read(type1_id)]);

        storage.unlock(&[TypeAccess::Read(type1_id)]);

        assert!(storage.can_be_locked(&[TypeAccess::Read(type1_id)]));
        assert!(storage.can_be_locked(&[TypeAccess::Write(type1_id)]));
        assert!(storage.can_be_locked(&[TypeAccess::Read(type2_id)]));
        assert!(storage.can_be_locked(&[TypeAccess::Write(type2_id)]));
    }

    #[test]
    fn unlock_multiple_times_read_type_for_read() {
        let mut storage = TypeStateStorage::default();
        let type1_id = TypeId::of::<u32>();
        let type2_id = TypeId::of::<i64>();
        storage.add(type1_id);
        storage.add(type2_id);
        storage.lock(&[TypeAccess::Read(type1_id)]);
        storage.lock(&[TypeAccess::Read(type1_id)]);

        storage.unlock(&[TypeAccess::Read(type1_id)]);

        assert!(storage.can_be_locked(&[TypeAccess::Read(type1_id)]));
        assert!(!storage.can_be_locked(&[TypeAccess::Write(type1_id)]));
        assert!(storage.can_be_locked(&[TypeAccess::Read(type2_id)]));
        assert!(storage.can_be_locked(&[TypeAccess::Write(type2_id)]));
    }

    #[test]
    #[should_panic]
    fn unlock_written_type_for_read() {
        let mut storage = TypeStateStorage::default();
        let type1_id = TypeId::of::<u32>();
        storage.add(type1_id);
        storage.lock(&[TypeAccess::Write(type1_id)]);

        storage.unlock(&[TypeAccess::Read(type1_id)]);
    }

    #[test]
    #[should_panic]
    fn unlock_nonexisting_type_for_write() {
        let mut storage = TypeStateStorage::default();
        let type1_id = TypeId::of::<u32>();
        let type2_id = TypeId::of::<i64>();
        storage.add(type1_id);

        storage.unlock(&[TypeAccess::Write(type2_id)]);
    }

    #[test]
    #[should_panic]
    fn unlock_free_type_for_write() {
        let mut storage = TypeStateStorage::default();
        let type1_id = TypeId::of::<u32>();
        storage.add(type1_id);

        storage.unlock(&[TypeAccess::Write(type1_id)]);
    }

    #[test]
    #[should_panic]
    fn unlock_read_type_for_write() {
        let mut storage = TypeStateStorage::default();
        let type1_id = TypeId::of::<u32>();
        storage.add(type1_id);
        storage.lock(&[TypeAccess::Read(type1_id)]);

        storage.unlock(&[TypeAccess::Write(type1_id)]);
    }

    #[test]
    fn unlock_written_type_for_write() {
        let mut storage = TypeStateStorage::default();
        let type1_id = TypeId::of::<u32>();
        let type2_id = TypeId::of::<i64>();
        storage.add(type1_id);
        storage.add(type2_id);
        storage.lock(&[TypeAccess::Write(type1_id)]);

        storage.unlock(&[TypeAccess::Write(type1_id)]);

        assert!(storage.can_be_locked(&[TypeAccess::Read(type1_id)]));
        assert!(storage.can_be_locked(&[TypeAccess::Write(type1_id)]));
        assert!(storage.can_be_locked(&[TypeAccess::Read(type2_id)]));
        assert!(storage.can_be_locked(&[TypeAccess::Write(type2_id)]));
    }
}

#[cfg(test)]

mod tests_action_state_storage {
    use super::*;

    #[test]
    fn default() {
        let storage = ActionStateStorage::default();

        assert!(storage.can_be_locked(true));
        assert!(storage.can_be_locked(false));
    }

    #[test]
    fn lock_for_system_requiring_actions_when_not_locked() {
        let mut storage = ActionStateStorage::default();

        storage.lock(true);

        assert!(!storage.can_be_locked(true));
        assert!(storage.can_be_locked(false));
    }

    #[test]
    fn lock_for_system_not_requiring_actions_when_not_locked() {
        let mut storage = ActionStateStorage::default();

        storage.lock(false);

        assert!(storage.can_be_locked(true));
        assert!(storage.can_be_locked(false));
    }

    #[test]
    #[should_panic]
    fn lock_for_system_requiring_actions_when_locked() {
        let mut storage = ActionStateStorage::default();
        storage.lock(true);

        storage.lock(true);
    }

    #[test]
    fn lock_for_system_not_requiring_actions_when_locked() {
        let mut storage = ActionStateStorage::default();
        storage.lock(true);

        storage.lock(false);

        assert!(!storage.can_be_locked(true));
        assert!(storage.can_be_locked(false));
    }

    #[test]
    #[should_panic]
    fn unlock_for_system_requiring_actions_when_not_locked() {
        let mut storage = ActionStateStorage::default();

        storage.unlock(true);
    }

    #[test]
    fn unlock_for_system_not_requiring_actions_when_not_locked() {
        let mut storage = ActionStateStorage::default();

        storage.unlock(false);

        assert!(storage.can_be_locked(true));
        assert!(storage.can_be_locked(false));
    }

    #[test]
    fn unlock_for_system_requiring_actions_when_locked() {
        let mut storage = ActionStateStorage::default();
        storage.lock(true);

        storage.unlock(true);

        assert!(storage.can_be_locked(true));
        assert!(storage.can_be_locked(false));
    }

    #[test]
    fn unlock_for_system_not_requiring_actions_when_locked() {
        let mut storage = ActionStateStorage::default();
        storage.lock(true);

        storage.unlock(false);

        assert!(!storage.can_be_locked(true));
        assert!(storage.can_be_locked(false));
    }
}

#[cfg(test)]
mod tests_systems_to_run_storage {
    use super::*;
    use std::convert::TryInto;

    #[test]
    fn add_systems() {
        let mut storage = SystemsToRunStorage::default();

        storage.add(2, 0);
        storage.add(3, 1);

        assert!(!storage.is_empty());
        assert_iter!(
            storage.iter(),
            &[
                SystemLocation::new(2, 0),
                SystemLocation::new(3, 0),
                SystemLocation::new(3, 1)
            ]
        );
    }

    #[test]
    fn delete_nonexisting_group() {
        let mut storage = SystemsToRunStorage::default();
        storage.add(1, 0);

        storage.delete(2.try_into().unwrap());

        assert_iter!(storage.iter(), [SystemLocation::new(1, 0)]);
    }

    #[test]
    fn delete_existing_group() {
        let mut storage = SystemsToRunStorage::default();
        storage.add(1, 0);
        storage.add(2, 0);

        storage.delete(1.try_into().unwrap());

        assert_iter!(storage.iter(), [SystemLocation::new(2, 0)]);
    }

    #[test]
    fn set_one_system_as_run() {
        let mut storage = SystemsToRunStorage::default();
        storage.add(2, 0);
        storage.add(2, 1);

        storage.set_as_run(SystemLocation::new(2, 0));

        assert!(!storage.is_empty());
        assert_iter!(storage.iter(), &[SystemLocation::new(2, 1)]);
    }

    #[test]
    fn set_all_systems_as_run() {
        let mut storage = SystemsToRunStorage::default();
        storage.add(2, 0);
        storage.add(2, 1);

        storage.set_as_run(SystemLocation::new(2, 0));
        storage.set_as_run(SystemLocation::new(2, 1));

        assert!(storage.is_empty());
        assert_eq!(storage.iter().next(), None);
    }

    #[test]
    fn reset() {
        let mut storage = SystemsToRunStorage::default();
        storage.add(2, 0);
        storage.add(2, 1);
        storage.set_as_run(SystemLocation::new(2, 0));

        storage.reset();

        assert!(!storage.is_empty());
        assert_iter!(
            storage.iter(),
            &[SystemLocation::new(2, 0), SystemLocation::new(2, 1)]
        );
    }
}
