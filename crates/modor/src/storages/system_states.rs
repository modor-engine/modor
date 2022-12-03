use super::systems::FullSystemProperties;
use crate::storages::actions::{ActionIdx, ActionStorage};
use crate::storages::components::ComponentTypeIdx;
use crate::storages::systems::{Access, ComponentTypeAccess, SystemIdx};
use modor_internal::ti_vec::TiVecSafeOperations;
use typed_index_collections::TiVec;

#[derive(Default)]
pub(super) struct SystemStateStorage {
    component_type_states: TiVec<ComponentTypeIdx, LockState>,
    updater_state: LockState,
    runnable_idxs: Vec<SystemIdx>,
    remaining_action_count: TiVec<ActionIdx, usize>,
    action_idxs: TiVec<SystemIdx, ActionIdx>,
}

impl SystemStateStorage {
    pub(super) fn add_system(
        &mut self,
        component_types: &[ComponentTypeAccess],
        action_idx: ActionIdx,
    ) {
        for component_types in component_types {
            *self
                .component_type_states
                .get_mut_or_create(component_types.type_idx) = LockState::Free;
        }
        self.action_idxs.push(action_idx);
    }

    pub(super) fn reset(
        &mut self,
        system_idxs: impl Iterator<Item = SystemIdx>,
        action_count: TiVec<ActionIdx, usize>,
    ) {
        for state in &mut self.component_type_states {
            *state = LockState::Free;
        }
        self.updater_state = LockState::Free;
        self.runnable_idxs.clear();
        self.runnable_idxs.extend(system_idxs);
        self.remaining_action_count = action_count;
    }

    pub(super) fn lock_next_system(
        &mut self,
        previous_system_idx: Option<SystemIdx>,
        actions: &ActionStorage,
        properties: &TiVec<SystemIdx, FullSystemProperties>,
    ) -> LockedSystem {
        if let Some(system_idx) = previous_system_idx {
            self.unlock(system_idx, &properties[system_idx]);
        }
        if self.runnable_idxs.is_empty() {
            LockedSystem::Done
        } else if let Some(system_idx) = self.extract_lockable_system_idx(actions, properties) {
            self.lock(&properties[system_idx]);
            LockedSystem::Remaining(Some(system_idx))
        } else {
            LockedSystem::Remaining(None)
        }
    }

    fn extract_lockable_system_idx(
        &mut self,
        actions: &ActionStorage,
        properties: &TiVec<SystemIdx, FullSystemProperties>,
    ) -> Option<SystemIdx> {
        self.runnable_idxs
            .iter()
            .copied()
            .position(|s| {
                (!properties[s].can_update || self.updater_state.is_lockable(Access::Write))
                    && actions
                        .dependency_idxs(self.action_idxs[s])
                        .iter()
                        .all(|&a| self.remaining_action_count[a] == 0)
                    && properties[s]
                        .component_types
                        .iter()
                        .all(|a| self.component_type_states[a.type_idx].is_lockable(a.access))
            })
            .map(|p| self.runnable_idxs.swap_remove(p))
    }

    fn unlock(&mut self, system_idx: SystemIdx, system: &FullSystemProperties) {
        for access in &system.component_types {
            let state = self.component_type_states[access.type_idx].unlock();
            self.component_type_states[access.type_idx] = state;
        }
        if system.can_update {
            self.updater_state = self.updater_state.unlock();
        }
        let action_idx = self.action_idxs[system_idx];
        self.remaining_action_count[action_idx] -= 1;
    }

    fn lock(&mut self, system: &FullSystemProperties) {
        for access in &system.component_types {
            let state = self.component_type_states[access.type_idx].lock(access.access);
            self.component_type_states[access.type_idx] = state;
        }
        if system.can_update {
            self.updater_state = self.updater_state.lock(Access::Write);
        }
    }
}

#[derive(PartialEq, Eq, Debug, Clone, Copy)]
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
    fn is_lockable(self, access: Access) -> bool {
        match access {
            Access::Read => matches!(self, Self::Free | Self::Read(_)),
            Access::Write => matches!(self, Self::Free),
        }
    }

    fn lock(self, access: Access) -> Self {
        match access {
            Access::Read => match self {
                Self::Free => Self::Read(0),
                Self::Read(count) => Self::Read(count + 1),
                Self::Written => panic!("internal error: cannot read written component"),
            },
            Access::Write => match self {
                Self::Free => Self::Written,
                Self::Read(_) => panic!("internal error: cannot write read component"),
                Self::Written => panic!("internal error: cannot write written component"),
            },
        }
    }

    fn unlock(self) -> Self {
        match self {
            Self::Free => panic!("internal error: cannot free already freed component"),
            Self::Read(0) | Self::Written => Self::Free,
            Self::Read(count) => Self::Read(count - 1),
        }
    }
}

#[derive(PartialEq, Eq, Debug)]
pub(super) enum LockedSystem {
    Remaining(Option<SystemIdx>),
    Done,
}

#[cfg(test)]
mod lock_state_tests {
    use crate::storages::system_states::LockState;
    use crate::storages::systems::Access;

    #[test]
    #[should_panic]
    fn lock_write_for_read_resource() {
        LockState::Read(0).lock(Access::Write);
    }

    #[test]
    #[should_panic]
    fn lock_read_for_written_resource() {
        LockState::Written.lock(Access::Read);
    }

    #[test]
    #[should_panic]
    fn lock_write_for_written_resource() {
        LockState::Written.lock(Access::Write);
    }

    #[test]
    #[should_panic]
    fn unlock_unlocked_resource() {
        LockState::Free.unlock();
    }
}
