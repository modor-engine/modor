use crate::internal::system_state::data::{LockedSystem, SystemLocation};
use crate::internal::system_state::storages::{
    ActionStateStorage, RunnableSystemStorage, SystemActionStorage, SystemTypeStorage,
    TypeStateStorage,
};
use crate::TypeAccess;
use std::any::TypeId;
use std::num::NonZeroUsize;

pub(super) mod data;
mod storages;

#[derive(Default)]
pub(super) struct SystemStateFacade {
    system_component_types: SystemTypeStorage,
    system_actions: SystemActionStorage,
    component_types_state: TypeStateStorage,
    action_state: ActionStateStorage,
    runnable_systems: RunnableSystemStorage,
}

impl SystemStateFacade {
    pub(super) fn delete_group(&mut self, group_idx: NonZeroUsize) {
        self.system_component_types.delete(group_idx);
        self.system_actions.delete(group_idx);
        self.runnable_systems.delete(group_idx);
    }

    pub(super) fn add_system(
        &mut self,
        group_idx: usize,
        system_idx: usize,
        component_types: Vec<TypeAccess>,
        actions: bool,
    ) {
        component_types
            .iter()
            .copied()
            .map(Self::extract_type)
            .for_each(|t| self.component_types_state.add(t));
        self.runnable_systems.add(group_idx, system_idx);
        self.system_component_types
            .set(group_idx, system_idx, component_types);
        self.system_actions.set(group_idx, system_idx, actions);
    }

    pub(super) fn lock_next_system(&mut self, previous_run_system: LockedSystem) -> LockedSystem {
        self.unlock_system(previous_run_system);
        if self.runnable_systems.is_empty() {
            return LockedSystem::Done;
        }
        let system_to_run_location = self.next_system();
        self.lock_system(system_to_run_location)
    }

    pub(super) fn reset(&mut self) {
        self.runnable_systems.reset();
    }

    fn next_system(&mut self) -> Option<SystemLocation> {
        self.runnable_systems.iter().find(|&system_location| {
            let group_idx = system_location.group_idx;
            let system_idx = system_location.system_idx;
            let component_types = self.system_component_types.get(group_idx, system_idx);
            let actions = self.system_actions.has_actions(group_idx, system_idx);
            self.component_types_state.can_be_locked(component_types)
                && self.action_state.can_be_locked(actions)
        })
    }

    fn lock_system(&mut self, system: Option<SystemLocation>) -> LockedSystem {
        system.map_or(LockedSystem::None, |system_location| {
            let group_idx = system_location.group_idx;
            let system_idx = system_location.system_idx;
            let component_types = self.system_component_types.get(group_idx, system_idx);
            self.component_types_state.lock(component_types);
            let actions = self.system_actions.has_actions(group_idx, system_idx);
            self.action_state.lock(actions);
            self.runnable_systems.set_as_run(system_location);
            LockedSystem::Some(system_location)
        })
    }

    fn unlock_system(&mut self, system: LockedSystem) {
        if let LockedSystem::Some(system_location) = system {
            let group_idx = system_location.group_idx;
            let system_idx = system_location.system_idx;
            let component_types = self.system_component_types.get(group_idx, system_idx);
            self.component_types_state.unlock(component_types);
            let actions = self.system_actions.has_actions(group_idx, system_idx);
            self.action_state.unlock(actions);
        }
    }

    pub(crate) fn extract_type(type_access: TypeAccess) -> TypeId {
        match type_access {
            TypeAccess::Read(type_id) | TypeAccess::Write(type_id) => type_id,
        }
    }
}

#[cfg(test)]
mod system_state_facade_tests {
    use super::*;
    use std::any::TypeId;
    use std::convert::TryInto;

    #[test]
    fn add_system() {
        let mut facade = SystemStateFacade::default();
        let type_access = vec![TypeAccess::Read(TypeId::of::<u32>())];

        facade.add_system(2, 0, type_access.clone(), true);

        assert!(facade.component_types_state.can_be_locked(&type_access));
        assert_iter!(facade.runnable_systems.iter(), [SystemLocation::new(2, 0)]);
        assert_eq!(facade.system_component_types.get(2, 0), type_access);
        assert!(facade.system_actions.has_actions(2, 0));
    }

    #[test]
    fn delete_group() {
        let mut facade = SystemStateFacade::default();
        let type_access = vec![TypeAccess::Read(TypeId::of::<u32>())];
        facade.add_system(1, 0, type_access.clone(), true);
        facade.add_system(2, 0, type_access.clone(), true);

        facade.delete_group(1.try_into().unwrap());

        assert_panics!(facade.system_component_types.get(1, 0));
        assert_panics!(facade.system_actions.has_actions(1, 0));
        assert_eq!(facade.system_component_types.get(2, 0), type_access);
        assert!(facade.system_actions.has_actions(2, 0));
        assert_iter!(facade.runnable_systems.iter(), [SystemLocation::new(2, 0)]);
    }

    #[test]
    fn lock_first_system_without_previous_locked_system() {
        let mut facade = SystemStateFacade::default();
        let type_access = TypeAccess::Write(TypeId::of::<u32>());
        facade.add_system(1, 0, vec![type_access], true);
        facade.add_system(2, 0, vec![type_access], true);

        let locked_system = facade.lock_next_system(LockedSystem::None);

        assert_eq!(locked_system, LockedSystem::Some(SystemLocation::new(1, 0)));
        assert!(!facade.component_types_state.can_be_locked(&[type_access]));
        assert!(!facade.action_state.can_be_locked(true));
        assert_iter!(facade.runnable_systems.iter(), [SystemLocation::new(2, 0)]);
    }

    #[test]
    fn lock_next_system_without_previous_locked_system() {
        let mut facade = SystemStateFacade::default();
        let type1_access = TypeAccess::Write(TypeId::of::<u32>());
        let type2_access = TypeAccess::Write(TypeId::of::<i64>());
        facade.add_system(1, 0, vec![type1_access], true);
        facade.add_system(2, 0, vec![type2_access], true);
        facade.add_system(3, 0, vec![type1_access], false);
        facade.add_system(4, 0, vec![type2_access], false);
        facade.lock_next_system(LockedSystem::None);

        let locked_system = facade.lock_next_system(LockedSystem::None);

        assert_eq!(locked_system, LockedSystem::Some(SystemLocation::new(4, 0)));
        assert!(!facade.component_types_state.can_be_locked(&[type2_access]));
        let locations = [SystemLocation::new(2, 0), SystemLocation::new(3, 0)];
        assert_iter!(facade.runnable_systems.iter(), locations);
    }

    #[test]
    fn lock_next_system_with_previous_locked_system() {
        let mut facade = SystemStateFacade::default();
        let type_access = TypeAccess::Write(TypeId::of::<u32>());
        facade.add_system(1, 0, vec![type_access], true);
        facade.add_system(2, 0, vec![type_access], true);
        let locked_system = facade.lock_next_system(LockedSystem::None);

        let locked_system = facade.lock_next_system(locked_system);

        assert_eq!(locked_system, LockedSystem::Some(SystemLocation::new(2, 0)));
        assert!(!facade.component_types_state.can_be_locked(&[type_access]));
        assert!(!facade.action_state.can_be_locked(true));
        assert_eq!(facade.runnable_systems.iter().next(), None);
    }

    #[test]
    fn lock_next_system_with_no_lockable_system() {
        let mut facade = SystemStateFacade::default();
        let type_access = TypeAccess::Write(TypeId::of::<u32>());
        facade.add_system(1, 0, vec![type_access], true);
        facade.add_system(2, 0, vec![type_access], true);
        facade.lock_next_system(LockedSystem::None);

        let locked_system = facade.lock_next_system(LockedSystem::None);

        assert_eq!(locked_system, LockedSystem::None);
    }

    #[test]
    fn lock_next_system_with_no_more_system_to_lock() {
        let mut facade = SystemStateFacade::default();
        let type_access = TypeAccess::Write(TypeId::of::<u32>());
        facade.add_system(1, 0, vec![type_access], true);
        facade.lock_next_system(LockedSystem::None);

        let locked_system = facade.lock_next_system(LockedSystem::None);

        assert_eq!(locked_system, LockedSystem::Done);
    }

    #[test]
    fn reset() {
        let mut facade = SystemStateFacade::default();
        let type_access = TypeAccess::Write(TypeId::of::<u32>());
        facade.add_system(1, 0, vec![type_access], true);
        facade.lock_next_system(LockedSystem::None);
        facade.lock_next_system(LockedSystem::None);

        facade.reset();

        assert_iter!(facade.runnable_systems.iter(), [SystemLocation::new(1, 0)]);
    }
}
