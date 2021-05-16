use crate::internal::components::interfaces::ComponentInterface;
use crate::internal::core::CoreFacade;
use crate::internal::entity_actions::EntityActionFacade;
use crate::internal::group_actions::GroupActionFacade;
use crate::internal::system::storages::{EntityTypeStorage, SystemStorage};
use crate::internal::system_state::data::LockedSystem;
use crate::internal::system_state::SystemStateFacade;
use crate::SystemData;
use data::SystemDetails;
use scoped_threadpool::Pool;
use std::mem;
use std::num::NonZeroUsize;
use std::sync::Mutex;

pub(crate) mod data;
mod storages;

#[derive(Default)]
pub(super) struct SystemFacade {
    systems: SystemStorage,
    entity_types: EntityTypeStorage,
    state: SystemStateFacade,
    pool: Option<Pool>,
}

impl SystemFacade {
    pub(super) fn set_thread_count(&mut self, count: u32) {
        if count < 2 {
            self.pool = None;
        } else {
            self.pool = Some(Pool::new(count - 1));
        }
    }

    pub(super) fn delete_group(&mut self, group_idx: NonZeroUsize) {
        self.systems.delete(group_idx);
        self.entity_types.delete(group_idx);
        self.state.delete_group(group_idx);
    }

    pub(super) fn add(&mut self, group_idx: Option<NonZeroUsize>, system: SystemDetails) {
        let group_idx = group_idx.map_or(0, NonZeroUsize::get);
        let system_idx = self.systems.add(group_idx, system.wrapper);
        self.entity_types
            .set(group_idx, system_idx, system.entity_type);
        self.state.add_system(
            group_idx,
            system_idx,
            system.component_types,
            system.group_actions,
            system.entity_actions,
        );
    }

    #[allow(clippy::option_if_let_else)]
    pub(super) fn run(
        &mut self,
        core: &CoreFacade,
        components: &ComponentInterface<'_>,
        group_actions: &Mutex<GroupActionFacade>,
        entity_actions: &Mutex<EntityActionFacade>,
    ) {
        let data = SystemData::new(core, components, group_actions, entity_actions);
        if let Some(pool) = &mut self.pool {
            Self::run_systems_in_parallel(
                &data,
                pool,
                &self.systems,
                &self.entity_types,
                &mut self.state,
            );
        } else {
            self.run_systems_sequentially(&data);
        }
    }

    fn run_systems_sequentially(&mut self, data: &SystemData<'_>) {
        for group_idx in 0..self.systems.group_count() {
            for system_idx in 0..self.systems.system_count(group_idx) {
                Self::run_system(
                    group_idx,
                    system_idx,
                    data,
                    &self.entity_types,
                    &self.systems,
                );
            }
        }
    }

    fn run_systems_in_parallel(
        data: &SystemData<'_>,
        pool: &mut Pool,
        systems: &SystemStorage,
        entity_types: &EntityTypeStorage,
        state: &mut SystemStateFacade,
    ) {
        let thread_count = pool.thread_count();
        let state_mutex = Mutex::new(mem::take(state));
        let state_ref = &state_mutex;
        pool.scoped(|scope| {
            for _ in 0..thread_count {
                scope.execute(move || Self::run_thread(state_ref, data, systems, entity_types));
            }
            Self::run_thread(state_ref, data, systems, entity_types);
        });
        *state = state_mutex.into_inner().unwrap();
        state.reset();
    }

    fn run_thread(
        state: &Mutex<SystemStateFacade>,
        data: &SystemData<'_>,
        systems: &SystemStorage,
        entity_types: &EntityTypeStorage,
    ) {
        let mut system_location = LockedSystem::None;
        loop {
            system_location = state.lock().unwrap().lock_next_system(system_location);
            if let LockedSystem::Done = &system_location {
                break;
            } else if let LockedSystem::Some(system_location) = &system_location {
                let group_idx = system_location.group_idx;
                let system_idx = system_location.system_idx;
                Self::run_system(group_idx, system_idx, data, entity_types, systems);
            }
        }
    }

    fn run_system(
        group_idx: usize,
        system_idx: usize,
        data: &SystemData<'_>,
        entity_types: &EntityTypeStorage,
        systems: &SystemStorage,
    ) {
        let filtered_types = entity_types
            .get(group_idx, system_idx)
            .into_iter()
            .collect();
        systems.run(group_idx, system_idx, data, filtered_types);
    }
}

#[cfg(test)]
mod tests_system_facade {
    use super::*;
    use crate::internal::components::ComponentFacade;
    use crate::internal::system_state::data::SystemLocation;
    use crate::{SystemInfo, TypeAccess};
    use std::any::TypeId;
    use std::convert::TryInto;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn set_thread_count_to_zero() {
        let mut facade = SystemFacade::default();

        facade.set_thread_count(0);

        assert!(facade.pool.is_none());
    }

    #[test]
    fn set_thread_count_to_one() {
        let mut facade = SystemFacade::default();

        facade.set_thread_count(1);

        assert!(facade.pool.is_none());
    }

    #[test]
    fn set_thread_count_to_more_than_one() {
        let mut facade = SystemFacade::default();

        facade.set_thread_count(2);

        assert_eq!(facade.pool.as_ref().map(Pool::thread_count), Some(1));
    }

    #[test]
    fn add_first_system_in_global_group() {
        let mut facade = SystemFacade::default();
        let component_types = vec![TypeAccess::Write(TypeId::of::<u32>())];
        let entity_type = Some(TypeId::of::<i64>());
        let system = SystemDetails::new(|_, _| (), component_types, entity_type, true, true);

        facade.add(None, system);

        assert_eq!(facade.systems.group_count(), 1);
        assert_eq!(facade.systems.system_count(0), 1);
        assert_eq!(facade.entity_types.get(0, 0), entity_type);
        let location = LockedSystem::Some(SystemLocation::new(0, 0));
        assert_eq!(facade.state.lock_next_system(LockedSystem::None), location)
    }

    #[test]
    fn add_first_system_in_other_group() {
        let mut facade = SystemFacade::default();
        let component_types = vec![TypeAccess::Write(TypeId::of::<u32>())];
        let entity_type = Some(TypeId::of::<i64>());
        let system = SystemDetails::new(|_, _| (), component_types, entity_type, true, true);

        facade.add(Some(2.try_into().unwrap()), system);

        assert_eq!(facade.systems.group_count(), 3);
        assert_eq!(facade.systems.system_count(2), 1);
        assert_eq!(facade.entity_types.get(2, 0), entity_type);
        let location = LockedSystem::Some(SystemLocation::new(2, 0));
        assert_eq!(facade.state.lock_next_system(LockedSystem::None), location)
    }

    #[test]
    fn add_other_system_in_other_group_with_same_component_types() {
        let mut facade = SystemFacade::default();
        let component_types = vec![TypeAccess::Write(TypeId::of::<i64>())];
        let system1 = SystemDetails::new(|_, _| (), component_types.clone(), None, true, false);
        let entity_type = Some(TypeId::of::<u32>());
        let system2 = SystemDetails::new(|_, _| (), component_types, entity_type, false, true);
        facade.add(Some(2.try_into().unwrap()), system1);

        facade.add(Some(2.try_into().unwrap()), system2);

        assert_eq!(facade.systems.group_count(), 3);
        assert_eq!(facade.systems.system_count(2), 2);
        assert_eq!(facade.entity_types.get(2, 1), entity_type);
        let location = LockedSystem::Some(SystemLocation::new(2, 0));
        assert_eq!(facade.state.lock_next_system(LockedSystem::None), location);
        let location = LockedSystem::None;
        assert_eq!(facade.state.lock_next_system(LockedSystem::None), location);
    }

    #[test]
    fn add_other_system_in_other_group_with_same_group_actions() {
        let mut facade = SystemFacade::default();
        let component_types = vec![TypeAccess::Write(TypeId::of::<i64>())];
        let system1 = SystemDetails::new(|_, _| (), component_types, None, true, false);
        let component_types = vec![TypeAccess::Write(TypeId::of::<u32>())];
        let entity_type = Some(TypeId::of::<u32>());
        let system2 = SystemDetails::new(|_, _| (), component_types, entity_type, true, true);
        facade.add(Some(2.try_into().unwrap()), system1);

        facade.add(Some(2.try_into().unwrap()), system2);

        assert_eq!(facade.systems.group_count(), 3);
        assert_eq!(facade.systems.system_count(2), 2);
        assert_eq!(facade.entity_types.get(2, 1), entity_type);
        let location = LockedSystem::Some(SystemLocation::new(2, 0));
        assert_eq!(facade.state.lock_next_system(LockedSystem::None), location);
        let location = LockedSystem::None;
        assert_eq!(facade.state.lock_next_system(LockedSystem::None), location);
    }

    #[test]
    fn add_other_system_in_other_group_with_different_characteristics() {
        let mut facade = SystemFacade::default();
        let component_types = vec![TypeAccess::Write(TypeId::of::<u32>())];
        let entity_type = Some(TypeId::of::<u32>());
        let system1 = SystemDetails::new(|_, _| (), component_types, entity_type, true, false);
        let component_types = vec![TypeAccess::Write(TypeId::of::<i64>())];
        let system2 = SystemDetails::new(|_, _| (), component_types, None, false, true);
        facade.add(Some(2.try_into().unwrap()), system1);

        facade.add(Some(2.try_into().unwrap()), system2);

        assert_eq!(facade.systems.group_count(), 3);
        assert_eq!(facade.systems.system_count(2), 2);
        assert_eq!(facade.entity_types.get(2, 1), None);
        let location = LockedSystem::Some(SystemLocation::new(2, 0));
        assert_eq!(facade.state.lock_next_system(LockedSystem::None), location);
        let location = LockedSystem::Some(SystemLocation::new(2, 1));
        assert_eq!(facade.state.lock_next_system(LockedSystem::None), location);
    }

    #[test]
    fn delete_group() {
        let mut facade = SystemFacade::default();
        let component_types = vec![TypeAccess::Write(TypeId::of::<u32>())];
        let entity_idx = Some(TypeId::of::<i64>());
        let system = SystemDetails::new(|_, _| (), component_types, entity_idx, true, true);
        facade.add(Some(2.try_into().unwrap()), system);

        facade.delete_group(2.try_into().unwrap());

        let core = CoreFacade::default();
        let mut components = ComponentFacade::default();
        let component_interface = components.components();
        let group_actions = Mutex::new(GroupActionFacade::default());
        let entity_actions = Mutex::new(EntityActionFacade::default());
        let data = SystemData::new(&core, &component_interface, &group_actions, &entity_actions);
        assert_panics!(facade.systems.run(2, 0, &data, Vec::new()));
        assert_panics!(facade.entity_types.get(2, 0));
        let location = LockedSystem::Done;
        assert_eq!(facade.state.lock_next_system(LockedSystem::None), location);
    }

    #[allow(clippy::needless_pass_by_value)]
    fn wrapper1(data: &SystemData<'_>, info: SystemInfo) {
        assert_eq!(info.filtered_component_types, [TypeId::of::<i16>()]);
        data.group_actions_mut()
            .mark_group_as_deleted(1.try_into().unwrap());
        data.entity_actions_mut().mark_entity_as_deleted(3);
        thread::sleep(Duration::from_millis(10));
    }

    #[allow(clippy::needless_pass_by_value)]
    fn wrapper2(data: &SystemData<'_>, info: SystemInfo) {
        thread::sleep(Duration::from_millis(10));
        assert_eq!(info.filtered_component_types, []);
        data.group_actions_mut()
            .mark_group_as_deleted(2.try_into().unwrap());
        data.entity_actions_mut().mark_entity_as_deleted(4);
    }

    #[test]
    fn run_systems_sequentially() {
        let mut facade = SystemFacade::default();
        let component_types = vec![TypeAccess::Write(TypeId::of::<u32>())];
        let entity_idx = Some(TypeId::of::<i16>());
        let system = SystemDetails::new(wrapper1, component_types, entity_idx, true, true);
        facade.add(Some(1.try_into().unwrap()), system);
        let component_types = vec![TypeAccess::Write(TypeId::of::<i64>())];
        let system = SystemDetails::new(wrapper2, component_types, None, true, true);
        facade.add(Some(2.try_into().unwrap()), system);
        let core = CoreFacade::default();
        let mut components = ComponentFacade::default();
        let component_interface = components.components();
        let group_actions = Mutex::new(GroupActionFacade::default());
        let entity_actions = Mutex::new(EntityActionFacade::default());

        facade.run(&core, &component_interface, &group_actions, &entity_actions);

        let group_actions = group_actions.try_lock().unwrap();
        assert_iter!(
            group_actions.deleted_group_idxs(),
            &[1.try_into().unwrap(), 2.try_into().unwrap()]
        );
        let entity_actions = entity_actions.try_lock().unwrap();
        assert_iter!(entity_actions.deleted_entity_idxs(), &[4, 3]);
    }

    #[test]
    fn run_systems_in_parallel() {
        let mut facade = SystemFacade::default();
        facade.set_thread_count(2);
        let component_types = vec![TypeAccess::Write(TypeId::of::<u32>())];
        let entity_idx = Some(TypeId::of::<i16>());
        let system = SystemDetails::new(wrapper1, component_types, entity_idx, true, true);
        facade.add(Some(1.try_into().unwrap()), system);
        let component_types = vec![TypeAccess::Write(TypeId::of::<i64>())];
        let system = SystemDetails::new(wrapper2, component_types, None, true, true);
        facade.add(Some(2.try_into().unwrap()), system);
        let core = CoreFacade::default();
        let mut components = ComponentFacade::default();
        let component_interface = components.components();
        let group_actions = Mutex::new(GroupActionFacade::default());
        let entity_actions = Mutex::new(EntityActionFacade::default());

        facade.run(&core, &component_interface, &group_actions, &entity_actions);

        let group_actions = group_actions.try_lock().unwrap();
        assert_iter!(
            group_actions.deleted_group_idxs(),
            &[1.try_into().unwrap(), 2.try_into().unwrap()]
        );
        let entity_actions = entity_actions.try_lock().unwrap();
        assert_iter!(entity_actions.deleted_entity_idxs(), &[4, 3]);
    }
}
