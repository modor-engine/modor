use crate::storages::actions::{ActionIdx, ActionStorage};
use crate::storages::components::ComponentTypeIdx;
use crate::storages::globals::GlobalIdx;
use crate::storages::systems::{Access, ComponentTypeAccess, GlobalAccess, SystemIdx};
use crate::utils;
use typed_index_collections::TiVec;

#[derive(Default)]
pub(super) struct SystemStateStorage {
    component_type_states: TiVec<ComponentTypeIdx, LockState>,
    global_states: TiVec<GlobalIdx, LockState>,
    updater_state: LockState,
    runnable_idxs: Vec<SystemIdx>,
    remaining_action_count: TiVec<ActionIdx, usize>,
    component_types: TiVec<SystemIdx, Vec<ComponentTypeAccess>>,
    globals: TiVec<SystemIdx, Vec<GlobalAccess>>,
    can_update: TiVec<SystemIdx, bool>,
    action_idxs: TiVec<SystemIdx, ActionIdx>,
}

impl SystemStateStorage {
    pub(super) fn add_system(
        &mut self,
        component_types: Vec<ComponentTypeAccess>,
        globals: Vec<GlobalAccess>,
        can_update: bool,
        action_idx: ActionIdx,
    ) {
        for component_types in &component_types {
            utils::set_value(
                &mut self.component_type_states,
                component_types.type_idx,
                LockState::Free,
            );
        }
        for global in &globals {
            utils::set_value(&mut self.global_states, global.idx, LockState::Free);
        }
        self.component_types.push(component_types);
        self.globals.push(globals);
        self.can_update.push(can_update);
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
        for state in &mut self.global_states {
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
    ) -> LockedSystem {
        if let Some(system_idx) = previous_system_idx {
            self.unlock(system_idx);
        }
        if self.runnable_idxs.is_empty() {
            LockedSystem::Done
        } else if let Some(system_idx) = self.extract_lockable_system_idx(actions) {
            self.lock(system_idx);
            LockedSystem::Remaining(Some(system_idx))
        } else {
            LockedSystem::Remaining(None)
        }
    }

    fn extract_lockable_system_idx(&mut self, actions: &ActionStorage) -> Option<SystemIdx> {
        self.runnable_idxs
            .iter()
            .copied()
            .position(|s| {
                (!self.can_update[s] || self.updater_state.is_lockable(Access::Write))
                    && actions
                        .dependency_idxs(self.action_idxs[s])
                        .iter()
                        .all(|&a| self.remaining_action_count[a] == 0)
                    && self.component_types[s]
                        .iter()
                        .all(|a| self.component_type_states[a.type_idx].is_lockable(a.access))
                    && self.globals[s]
                        .iter()
                        .all(|a| self.global_states[a.idx].is_lockable(a.access))
            })
            .map(|p| self.runnable_idxs.swap_remove(p))
    }

    fn unlock(&mut self, system_idx: SystemIdx) {
        for access in &self.component_types[system_idx] {
            let state = self.component_type_states[access.type_idx].unlock();
            self.component_type_states[access.type_idx] = state;
        }
        for access in &self.globals[system_idx] {
            let state = self.global_states[access.idx].unlock();
            self.global_states[access.idx] = state;
        }
        if self.can_update[system_idx] {
            self.updater_state = self.updater_state.unlock();
        }
        let action_idx = self.action_idxs[system_idx];
        self.remaining_action_count[action_idx] -= 1;
    }

    fn lock(&mut self, system_idx: SystemIdx) {
        for access in &self.component_types[system_idx] {
            let state = self.component_type_states[access.type_idx].lock(access.access);
            self.component_type_states[access.type_idx] = state;
        }
        for access in &self.globals[system_idx] {
            let state = self.global_states[access.idx].lock(access.access);
            self.global_states[access.idx] = state;
        }
        if self.can_update[system_idx] {
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
mod system_state_storage_tests {
    use crate::storages::actions::{ActionDependencies, ActionStorage};
    use crate::storages::system_states::{LockedSystem, SystemStateStorage};
    use crate::storages::systems::{Access, ComponentTypeAccess, GlobalAccess};
    use std::any::TypeId;

    #[test]
    fn lock_systems_that_can_update() {
        let mut actions = ActionStorage::default();
        let action_idx = actions.idx_or_create(None, ActionDependencies::Types(vec![]));
        let mut storage = SystemStateStorage::default();
        storage.add_system(vec![], vec![], true, action_idx);
        storage.add_system(vec![], vec![], true, action_idx);
        storage.reset([0.into(), 1.into()].into_iter(), ti_vec![2]);
        storage.reset([0.into(), 1.into()].into_iter(), ti_vec![2]);
        let locked_system = storage.lock_next_system(None, &actions);
        assert_eq!(locked_system, LockedSystem::Remaining(Some(0.into())));
        let locked_system = storage.lock_next_system(None, &actions);
        assert_eq!(locked_system, LockedSystem::Remaining(None));
        let locked_system = storage.lock_next_system(Some(0.into()), &actions);
        assert_eq!(locked_system, LockedSystem::Remaining(Some(1.into())));
        let locked_system = storage.lock_next_system(Some(1.into()), &actions);
        assert_eq!(locked_system, LockedSystem::Done);
        storage.reset([0.into(), 1.into()].into_iter(), ti_vec![2]);
        let locked_system = storage.lock_next_system(None, &actions);
        assert_eq!(locked_system, LockedSystem::Remaining(Some(0.into())));
    }

    #[test]
    fn lock_systems_with_components() {
        let mut actions = ActionStorage::default();
        let action_idx = actions.idx_or_create(None, ActionDependencies::Types(vec![]));
        let access = ComponentTypeAccess {
            access: Access::Write,
            type_idx: 10.into(),
        };
        let mut storage = SystemStateStorage::default();
        storage.add_system(vec![access], vec![], false, action_idx);
        storage.add_system(vec![access], vec![], false, action_idx);
        storage.reset([0.into(), 1.into()].into_iter(), ti_vec![2]);
        storage.reset([0.into(), 1.into()].into_iter(), ti_vec![2]);
        let locked_system = storage.lock_next_system(None, &actions);
        assert_eq!(locked_system, LockedSystem::Remaining(Some(0.into())));
        let locked_system = storage.lock_next_system(None, &actions);
        assert_eq!(locked_system, LockedSystem::Remaining(None));
        let locked_system = storage.lock_next_system(Some(0.into()), &actions);
        assert_eq!(locked_system, LockedSystem::Remaining(Some(1.into())));
        let locked_system = storage.lock_next_system(Some(1.into()), &actions);
        assert_eq!(locked_system, LockedSystem::Done);
        storage.reset([0.into(), 1.into()].into_iter(), ti_vec![2]);
        let locked_system = storage.lock_next_system(None, &actions);
        assert_eq!(locked_system, LockedSystem::Remaining(Some(0.into())));
    }

    #[test]
    fn lock_systems_with_globals() {
        let mut actions = ActionStorage::default();
        let action_idx = actions.idx_or_create(None, ActionDependencies::Types(vec![]));
        let access = GlobalAccess {
            access: Access::Write,
            idx: 10.into(),
        };
        let mut storage = SystemStateStorage::default();
        storage.add_system(vec![], vec![access], false, action_idx);
        storage.add_system(vec![], vec![access], false, action_idx);
        storage.reset([0.into(), 1.into()].into_iter(), ti_vec![2]);
        storage.reset([0.into(), 1.into()].into_iter(), ti_vec![2]);
        let locked_system = storage.lock_next_system(None, &actions);
        assert_eq!(locked_system, LockedSystem::Remaining(Some(0.into())));
        let locked_system = storage.lock_next_system(None, &actions);
        assert_eq!(locked_system, LockedSystem::Remaining(None));
        let locked_system = storage.lock_next_system(Some(0.into()), &actions);
        assert_eq!(locked_system, LockedSystem::Remaining(Some(1.into())));
        let locked_system = storage.lock_next_system(Some(1.into()), &actions);
        assert_eq!(locked_system, LockedSystem::Done);
        storage.reset([0.into(), 1.into()].into_iter(), ti_vec![2]);
        let locked_system = storage.lock_next_system(None, &actions);
        assert_eq!(locked_system, LockedSystem::Remaining(Some(0.into())));
    }

    #[test]
    fn lock_systems_with_action_dependencies() {
        let mut actions = ActionStorage::default();
        let action1_idx =
            actions.idx_or_create(Some(TypeId::of::<u32>()), ActionDependencies::Types(vec![]));
        let action2_idx =
            actions.idx_or_create(None, ActionDependencies::Types(vec![TypeId::of::<u32>()]));
        let mut storage = SystemStateStorage::default();
        storage.add_system(vec![], vec![], false, action1_idx);
        storage.add_system(vec![], vec![], false, action2_idx);
        storage.reset([0.into(), 1.into()].into_iter(), ti_vec![1, 1]);
        storage.reset([0.into(), 1.into()].into_iter(), ti_vec![1, 1]);
        let locked_system = storage.lock_next_system(None, &actions);
        assert_eq!(locked_system, LockedSystem::Remaining(Some(0.into())));
        let locked_system = storage.lock_next_system(None, &actions);
        assert_eq!(locked_system, LockedSystem::Remaining(None));
        let locked_system = storage.lock_next_system(Some(0.into()), &actions);
        assert_eq!(locked_system, LockedSystem::Remaining(Some(1.into())));
        let locked_system = storage.lock_next_system(Some(1.into()), &actions);
        assert_eq!(locked_system, LockedSystem::Done);
        storage.reset([0.into(), 1.into()].into_iter(), ti_vec![1, 1]);
        let locked_system = storage.lock_next_system(None, &actions);
        assert_eq!(locked_system, LockedSystem::Remaining(Some(0.into())));
    }
}

#[cfg(test)]
mod lock_state_tests {
    use crate::storages::system_states::LockState;
    use crate::storages::systems::Access;

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
    fn lock() {
        assert_eq!(LockState::Free.lock(Access::Read), LockState::Read(0));
        assert_eq!(LockState::Free.lock(Access::Write), LockState::Written);
        assert_eq!(LockState::Read(0).lock(Access::Read), LockState::Read(1));
        assert_eq!(LockState::Read(1).lock(Access::Read), LockState::Read(2));
        assert_panics!(LockState::Read(0).lock(Access::Write));
        assert_panics!(LockState::Written.lock(Access::Read));
        assert_panics!(LockState::Written.lock(Access::Write));
    }

    #[test]
    fn unlock() {
        assert_panics!(LockState::Free.unlock());
        assert_eq!(LockState::Read(0).unlock(), LockState::Free);
        assert_eq!(LockState::Read(1).unlock(), LockState::Read(0));
        assert_eq!(LockState::Written.unlock(), LockState::Free);
    }
}
