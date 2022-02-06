use crate::storages::actions::{ActionIdx, ActionStorage};
use crate::storages::components::ComponentTypeIdx;
use crate::storages::systems::{Access, ComponentTypeAccess, SystemIdx};
use crate::utils;
use itertools::Itertools;
use typed_index_collections::{TiSlice, TiVec};

#[derive(Default)]
pub(super) struct SystemStateStorage {
    component_type_state: TiVec<ComponentTypeIdx, LockState>,
    updater_state: LockState,
    runnable_idxs: Vec<SystemIdx>,
    remaining_action_count: TiVec<ActionIdx, usize>,
}

impl SystemStateStorage {
    pub(super) fn register_component_type(&mut self, type_idx: ComponentTypeIdx) {
        utils::set_value(&mut self.component_type_state, type_idx, LockState::Free);
    }

    pub(super) fn reset(
        &mut self,
        system_idxs: impl Iterator<Item = SystemIdx>,
        action_count: TiVec<ActionIdx, usize>,
    ) {
        for state in &mut self.component_type_state {
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
        system_properties: AllSystemProperties<'_>,
        actions: &ActionStorage,
    ) -> LockedSystem {
        if let Some(system_idx) = previous_system_idx {
            self.unlock(system_idx, system_properties);
        }
        if self.runnable_idxs.is_empty() {
            LockedSystem::Done
        } else if let Some(system_idx) =
            self.extract_lockable_system_idx(system_properties, actions)
        {
            self.lock(system_idx, system_properties);
            LockedSystem::Remaining(Some(system_idx))
        } else {
            LockedSystem::Remaining(None)
        }
    }

    fn extract_lockable_system_idx(
        &mut self,
        system_properties: AllSystemProperties<'_>,
        actions: &ActionStorage,
    ) -> Option<SystemIdx> {
        self.runnable_idxs
            .iter()
            .copied()
            .find_position(|&s| {
                (!system_properties.can_update[s] || self.updater_state.is_lockable(Access::Write))
                    && actions
                        .dependency_idxs(system_properties.action_idxs[s])
                        .iter()
                        .all(|&a| self.remaining_action_count[a] == 0)
                    && system_properties.component_types[s]
                        .iter()
                        .all(|a| self.component_type_state[a.type_idx].is_lockable(a.access))
            })
            .map(|(p, i)| {
                self.runnable_idxs.swap_remove(p);
                i
            })
    }

    fn unlock(&mut self, system_idx: SystemIdx, system_properties: AllSystemProperties<'_>) {
        for type_access in &system_properties.component_types[system_idx] {
            let state = self.component_type_state[type_access.type_idx].unlock();
            self.component_type_state[type_access.type_idx] = state;
        }
        if system_properties.can_update[system_idx] {
            self.updater_state = self.updater_state.unlock();
        }
        let action_idx = system_properties.action_idxs[system_idx];
        self.remaining_action_count[action_idx] -= 1;
    }

    fn lock(&mut self, system_idx: SystemIdx, system_properties: AllSystemProperties<'_>) {
        for type_access in &system_properties.component_types[system_idx] {
            let state = self.component_type_state[type_access.type_idx].lock(type_access.access);
            self.component_type_state[type_access.type_idx] = state;
        }
        if system_properties.can_update[system_idx] {
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

#[derive(Clone, Copy)]
pub(super) struct AllSystemProperties<'a> {
    pub(super) component_types: &'a TiSlice<SystemIdx, Vec<ComponentTypeAccess>>,
    pub(super) can_update: &'a TiSlice<SystemIdx, bool>,
    pub(super) action_idxs: &'a TiSlice<SystemIdx, ActionIdx>,
}

#[derive(PartialEq, Eq, Debug)]
pub(super) enum LockedSystem {
    Remaining(Option<SystemIdx>),
    Done,
}

#[cfg(test)]
mod system_state_storage_tests {
    use crate::storages::actions::{ActionDefinition, ActionDependencies, ActionStorage};
    use crate::storages::components::ComponentTypeIdx;
    use crate::storages::system_states::{AllSystemProperties, LockedSystem, SystemStateStorage};
    use crate::storages::systems::{Access, ComponentTypeAccess};
    use std::any::TypeId;

    impl SystemStateStorage {
        pub(in super::super) fn last_component_type_idx(&self) -> Option<ComponentTypeIdx> {
            self.component_type_state.last_key()
        }
    }

    #[test]
    fn lock_systems_that_can_update() {
        let mut storage = SystemStateStorage::default();
        let mut actions = ActionStorage::default();
        let actions_idx = actions.idx_or_create(ActionDefinition {
            type_: None,
            dependency_types: ActionDependencies::Types(vec![]),
        });
        let properties = AllSystemProperties {
            component_types: &ti_vec![vec![], vec![]],
            can_update: &ti_vec![true, true],
            action_idxs: &ti_vec![actions_idx, actions_idx],
        };
        storage.reset([0.into(), 1.into()].into_iter(), ti_vec![2]);
        let locked_system = storage.lock_next_system(None, properties, &actions);
        assert_eq!(locked_system, LockedSystem::Remaining(Some(0.into())));
        let locked_system = storage.lock_next_system(None, properties, &actions);
        assert_eq!(locked_system, LockedSystem::Remaining(None));
        let locked_system = storage.lock_next_system(Some(0.into()), properties, &actions);
        assert_eq!(locked_system, LockedSystem::Remaining(Some(1.into())));
        let locked_system = storage.lock_next_system(Some(1.into()), properties, &actions);
        assert_eq!(locked_system, LockedSystem::Done);
        storage.reset([0.into(), 1.into()].into_iter(), ti_vec![2]);
        let locked_system = storage.lock_next_system(None, properties, &actions);
        assert_eq!(locked_system, LockedSystem::Remaining(Some(0.into())));
    }

    #[test]
    fn lock_systems_with_components() {
        let mut storage = SystemStateStorage::default();
        let component_type_access = ComponentTypeAccess {
            access: Access::Write,
            type_idx: 10.into(),
        };
        let mut actions = ActionStorage::default();
        let action_idx = actions.idx_or_create(ActionDefinition {
            type_: None,
            dependency_types: ActionDependencies::Types(vec![]),
        });
        let properties = AllSystemProperties {
            component_types: &ti_vec![vec![component_type_access], vec![component_type_access]],
            can_update: &ti_vec![false, false],
            action_idxs: &ti_vec![action_idx, action_idx],
        };
        storage.register_component_type(10.into());
        storage.reset([0.into(), 1.into()].into_iter(), ti_vec![2]);
        let locked_system = storage.lock_next_system(None, properties, &actions);
        assert_eq!(locked_system, LockedSystem::Remaining(Some(0.into())));
        let locked_system = storage.lock_next_system(None, properties, &actions);
        assert_eq!(locked_system, LockedSystem::Remaining(None));
        let locked_system = storage.lock_next_system(Some(0.into()), properties, &actions);
        assert_eq!(locked_system, LockedSystem::Remaining(Some(1.into())));
        let locked_system = storage.lock_next_system(Some(1.into()), properties, &actions);
        assert_eq!(locked_system, LockedSystem::Done);
        storage.reset([0.into(), 1.into()].into_iter(), ti_vec![2]);
        let locked_system = storage.lock_next_system(None, properties, &actions);
        assert_eq!(locked_system, LockedSystem::Remaining(Some(0.into())));
    }

    #[test]
    fn lock_systems_with_action_dependencies() {
        let mut storage = SystemStateStorage::default();
        let mut actions = ActionStorage::default();
        let action1_idx = actions.idx_or_create(ActionDefinition {
            type_: Some(TypeId::of::<u32>()),
            dependency_types: ActionDependencies::Types(vec![]),
        });
        let action2_idx = actions.idx_or_create(ActionDefinition {
            type_: None,
            dependency_types: ActionDependencies::Types(vec![TypeId::of::<u32>()]),
        });
        let properties = AllSystemProperties {
            component_types: &ti_vec![vec![], vec![]],
            can_update: &ti_vec![false, false],
            action_idxs: &ti_vec![action1_idx, action2_idx],
        };
        storage.reset([0.into(), 1.into()].into_iter(), ti_vec![1, 1]);
        let locked_system = storage.lock_next_system(None, properties, &actions);
        assert_eq!(locked_system, LockedSystem::Remaining(Some(0.into())));
        let locked_system = storage.lock_next_system(None, properties, &actions);
        assert_eq!(locked_system, LockedSystem::Remaining(None));
        let locked_system = storage.lock_next_system(Some(0.into()), properties, &actions);
        assert_eq!(locked_system, LockedSystem::Remaining(Some(1.into())));
        let locked_system = storage.lock_next_system(Some(1.into()), properties, &actions);
        assert_eq!(locked_system, LockedSystem::Done);
        storage.reset([0.into(), 1.into()].into_iter(), ti_vec![1, 1]);
        let locked_system = storage.lock_next_system(None, properties, &actions);
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
