use crate::storages::components::ComponentTypeIdx;
use crate::storages::systems::{Access, ComponentTypeAccess, SystemIdx};
use crate::utils;
use itertools::Itertools;
use typed_index_collections::{TiSlice, TiVec};

#[derive(Default)]
pub(super) struct SystemStateStorage {
    component_type_state: TiVec<ComponentTypeIdx, LockState>,
    entity_action_state: LockState,
    runnable_idxs: Vec<SystemIdx>,
}

impl SystemStateStorage {
    pub(super) fn register_component_type(&mut self, type_idx: ComponentTypeIdx) {
        utils::set_value(&mut self.component_type_state, type_idx, LockState::Free);
    }

    pub(super) fn reset(&mut self, system_idxs: impl Iterator<Item = SystemIdx>) {
        for state in &mut self.component_type_state {
            *state = LockState::Free;
        }
        self.entity_action_state = LockState::Free;
        self.runnable_idxs.clear();
        self.runnable_idxs.extend(system_idxs);
    }

    pub(super) fn lock_next_system(
        &mut self,
        previous_system_idx: Option<SystemIdx>,
        system_properties: SystemProperties<'_>,
    ) -> LockedSystem {
        if let Some(system_idx) = previous_system_idx {
            self.unlock(system_idx, system_properties);
        }
        if self.runnable_idxs.is_empty() {
            LockedSystem::Done
        } else if let Some(system_idx) = self.extract_lockable_system_idx(system_properties) {
            self.lock(system_idx, system_properties);
            LockedSystem::Remaining(Some(system_idx))
        } else {
            LockedSystem::Remaining(None)
        }
    }

    fn extract_lockable_system_idx(
        &mut self,
        system_properties: SystemProperties<'_>,
    ) -> Option<SystemIdx> {
        self.runnable_idxs
            .iter()
            .copied()
            .find_position(|&s| {
                (!system_properties.have_entity_actions[s]
                    || self.entity_action_state.is_lockable(Access::Write))
                    && system_properties.component_types[s]
                        .iter()
                        .all(|a| self.component_type_state[a.idx].is_lockable(a.access))
            })
            .map(|(p, i)| {
                self.runnable_idxs.swap_remove(p);
                i
            })
    }

    fn unlock(&mut self, system_idx: SystemIdx, system_properties: SystemProperties<'_>) {
        for type_access in &system_properties.component_types[system_idx] {
            self.component_type_state[type_access.idx].unlock();
        }
        if system_properties.have_entity_actions[system_idx] {
            self.entity_action_state.unlock();
        }
    }

    fn lock(&mut self, system_idx: SystemIdx, system_properties: SystemProperties<'_>) {
        for type_access in &system_properties.component_types[system_idx] {
            self.component_type_state[type_access.idx].lock(type_access.access);
        }
        if system_properties.have_entity_actions[system_idx] {
            self.entity_action_state.lock(Access::Write);
        }
    }
}

#[derive(PartialEq, Eq, Debug)]
enum LockState {
    Free,
    Read(usize),
    Written,
}

impl Default for LockState {
    fn default() -> Self {
        Self::Free
    }
}

impl LockState {
    fn is_lockable(&self, access: Access) -> bool {
        match access {
            Access::Read => matches!(self, Self::Free | Self::Read(_)),
            Access::Write => matches!(self, Self::Free),
        }
    }

    fn lock(&mut self, access: Access) {
        match access {
            Access::Read => match self {
                Self::Free => *self = Self::Read(0),
                Self::Read(count) => *count += 1,
                Self::Written => panic!("internal error: cannot read written component"),
            },
            Access::Write => match self {
                Self::Free => *self = Self::Written,
                Self::Read(_) => panic!("internal error: cannot write read component"),
                Self::Written => panic!("internal error: cannot write written component"),
            },
        }
    }

    fn unlock(&mut self) {
        match self {
            Self::Free => panic!("internal error: cannot free already freed component"),
            Self::Read(0) | Self::Written => *self = Self::Free,
            Self::Read(count) => *count -= 1,
        }
    }
}

#[derive(Clone, Copy)]
pub(super) struct SystemProperties<'a> {
    pub(super) component_types: &'a TiSlice<SystemIdx, Vec<ComponentTypeAccess>>,
    pub(super) have_entity_actions: &'a TiSlice<SystemIdx, bool>,
}

#[derive(PartialEq, Eq, Debug)]
pub(super) enum LockedSystem {
    Remaining(Option<SystemIdx>),
    Done,
}

#[cfg(test)]
mod system_state_storage_tests {
    use super::*;

    impl SystemStateStorage {
        pub(in super::super) fn last_component_type_idx(&self) -> Option<ComponentTypeIdx> {
            self.component_type_state.last_key()
        }
    }

    #[test]
    fn register_missing_component_type() {
        let mut storage = SystemStateStorage {
            component_type_state: ti_vec![LockState::Written],
            ..Default::default()
        };

        storage.register_component_type(2.into());

        let state = ti_vec![LockState::Written, LockState::Free, LockState::Free];
        assert_eq!(storage.component_type_state, state);
    }

    #[test]
    fn register_existing_component_type() {
        let mut storage = SystemStateStorage {
            component_type_state: ti_vec![LockState::Written],
            ..Default::default()
        };

        storage.register_component_type(0.into());

        assert_eq!(storage.component_type_state, ti_vec![LockState::Free]);
    }

    #[test]
    fn reset() {
        let mut storage = SystemStateStorage {
            component_type_state: ti_vec![LockState::Written, LockState::Read(0)],
            entity_action_state: LockState::Read(0),
            runnable_idxs: vec![2.into(), 3.into()],
        };

        storage.reset([0.into(), 1.into()].into_iter());

        let state = ti_vec![LockState::Free, LockState::Free];
        assert_eq!(storage.component_type_state, state);
        assert_eq!(storage.entity_action_state, LockState::Free);
        assert_eq!(storage.runnable_idxs, [0.into(), 1.into()]);
    }

    #[test]
    fn lock_systems_with_entity_action() {
        let mut storage = SystemStateStorage::default();
        storage.reset([0.into(), 1.into()].into_iter());
        let properties = SystemProperties {
            component_types: &ti_vec![vec![], vec![]],
            have_entity_actions: &ti_vec![true, true],
        };

        let locked_system = storage.lock_next_system(None, properties);
        assert_eq!(locked_system, LockedSystem::Remaining(Some(0.into())));
        assert_eq!(storage.entity_action_state, LockState::Written);
        assert_eq!(storage.runnable_idxs, vec![1.into()]);

        let locked_system = storage.lock_next_system(None, properties);
        assert_eq!(locked_system, LockedSystem::Remaining(None));
        assert_eq!(storage.entity_action_state, LockState::Written);
        assert_eq!(storage.runnable_idxs, vec![1.into()]);

        let locked_system = storage.lock_next_system(Some(0.into()), properties);
        assert_eq!(locked_system, LockedSystem::Remaining(Some(1.into())));
        assert_eq!(storage.entity_action_state, LockState::Written);
        assert_eq!(storage.runnable_idxs, vec![]);

        let locked_system = storage.lock_next_system(Some(1.into()), properties);
        assert_eq!(locked_system, LockedSystem::Done);
        assert_eq!(storage.entity_action_state, LockState::Free);
    }

    #[test]
    fn lock_systems_with_components() {
        let mut storage = SystemStateStorage::default();
        storage.reset([0.into(), 1.into()].into_iter());
        storage.register_component_type(1.into());
        let component_type_access = create_type_access(1.into(), Access::Write);
        let properties = SystemProperties {
            component_types: &ti_vec![vec![component_type_access], vec![component_type_access]],
            have_entity_actions: &ti_vec![false, false],
        };

        let locked_system = storage.lock_next_system(None, properties);
        assert_eq!(locked_system, LockedSystem::Remaining(Some(0.into())));
        let state = ti_vec![LockState::Free, LockState::Written];
        assert_eq!(storage.component_type_state, state);
        assert_eq!(storage.runnable_idxs, vec![1.into()]);

        let locked_system = storage.lock_next_system(None, properties);
        assert_eq!(locked_system, LockedSystem::Remaining(None));
        let state = ti_vec![LockState::Free, LockState::Written];
        assert_eq!(storage.component_type_state, state);
        assert_eq!(storage.runnable_idxs, vec![1.into()]);

        let locked_system = storage.lock_next_system(Some(0.into()), properties);
        assert_eq!(locked_system, LockedSystem::Remaining(Some(1.into())));
        let state = ti_vec![LockState::Free, LockState::Written];
        assert_eq!(storage.component_type_state, state);
        assert_eq!(storage.runnable_idxs, vec![]);

        let locked_system = storage.lock_next_system(Some(1.into()), properties);
        assert_eq!(locked_system, LockedSystem::Done);
        let state = ti_vec![LockState::Free, LockState::Free];
        assert_eq!(storage.component_type_state, state);
    }

    fn create_type_access(type_idx: ComponentTypeIdx, access: Access) -> ComponentTypeAccess {
        ComponentTypeAccess {
            idx: type_idx,
            access,
        }
    }
}

#[cfg(test)]
mod lock_state_tests {
    use super::*;

    #[test]
    fn retrieve_whether_is_lockable() {
        assert!(LockState::Free.is_lockable(Access::Read));
        assert!(LockState::Free.is_lockable(Access::Write));
        assert!(LockState::Read(0).is_lockable(Access::Read));
        assert!(!LockState::Read(0).is_lockable(Access::Write));
        assert!(!LockState::Written.is_lockable(Access::Read));
        assert!(!LockState::Written.is_lockable(Access::Write));
    }

    #[test]
    fn lock_read_once() {
        let mut state = LockState::Free;

        state.lock(Access::Read);

        assert_eq!(state, LockState::Read(0));
        assert_panics!(state.lock(Access::Write));
    }

    #[test]
    fn lock_read_multiple_times() {
        let mut state = LockState::Free;

        state.lock(Access::Read);
        state.lock(Access::Read);
        state.lock(Access::Read);

        assert_eq!(state, LockState::Read(2));
        assert_panics!(state.lock(Access::Write));
    }

    #[test]
    fn lock_write() {
        let mut state = LockState::Free;

        state.lock(Access::Write);

        assert_eq!(state, LockState::Written);
        assert_panics!(state.lock(Access::Read));
        assert_panics!(state.lock(Access::Write));
    }

    #[test]
    fn unlock_read_once() {
        let mut state = LockState::Read(0);

        state.unlock();

        assert_eq!(state, LockState::Free);
        assert_panics!(state.unlock());
    }

    #[test]
    fn unlock_read_multiple_time() {
        let mut state = LockState::Read(2);

        state.unlock();

        assert_eq!(state, LockState::Read(1));
    }

    #[test]
    fn unlock_written() {
        let mut state = LockState::Written;

        state.unlock();

        assert_eq!(state, LockState::Free);
    }
}
