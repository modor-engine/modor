use crate::storages::components::ComponentTypeIdx;
use crate::storages::system_states::{LockedSystem, SystemProperties, SystemStateStorage};
use crate::systems::internal::SystemWrapper;
use crate::{SystemData, SystemInfo};
use scoped_threadpool::Pool;
use std::any::TypeId;
use std::sync::Mutex;
use typed_index_collections::{TiSlice, TiVec};

#[derive(Default)]
pub(crate) struct SystemStorage {
    wrappers: TiVec<SystemIdx, Option<SystemWrapper>>,
    component_types: TiVec<SystemIdx, Vec<ComponentTypeAccess>>,
    have_entity_actions: TiVec<SystemIdx, bool>,
    entity_main_component_types: TiVec<SystemIdx, TypeId>,
    states: Mutex<SystemStateStorage>,
    pool: Option<Pool>,
}

impl SystemStorage {
    pub(crate) fn thread_count(&self) -> u32 {
        self.pool.as_ref().map_or(0, Pool::thread_count) + 1
    }

    pub(super) fn set_thread_count(&mut self, count: u32) {
        if count < 2 {
            self.pool = None;
        } else {
            self.pool = Some(Pool::new(count - 1));
        }
    }

    pub(super) fn add(
        &mut self,
        wrapper: SystemWrapper,
        entity_main_component_type: TypeId,
        properties: FullSystemProperties,
    ) -> SystemIdx {
        let states = self
            .states
            .get_mut()
            .expect("internal error: cannot access to system states to register component type");
        for component_types in &properties.component_types {
            states.register_component_type(component_types.idx);
        }
        self.wrappers.push(Some(wrapper));
        self.component_types.push(properties.component_types);
        self.have_entity_actions.push(properties.has_entity_actions);
        self.entity_main_component_types
            .push_and_get_key(entity_main_component_type)
    }

    pub(super) fn run(&mut self, data: &SystemData<'_>) {
        if let Some(mut pool) = self.pool.take() {
            self.run_in_parallel(&mut pool, data);
            self.pool = Some(pool);
        } else {
            self.run_sequentially(data);
        }
    }

    fn run_sequentially(&mut self, data: &SystemData<'_>) {
        for system_idx in Self::all_idxs(&self.wrappers) {
            Self::run_system(
                system_idx,
                &self.entity_main_component_types,
                &self.wrappers,
                data,
            );
        }
    }

    fn run_in_parallel(&mut self, pool: &mut Pool, data: &SystemData<'_>) {
        self.states
            .get_mut()
            .expect("internal error: cannot reset states")
            .reset(Self::all_idxs(&self.wrappers));
        let system_properties = SystemProperties {
            component_types: &self.component_types,
            have_entity_actions: &self.have_entity_actions,
        };
        let thread_count = pool.thread_count();
        pool.scoped(|s| {
            for _ in 0..thread_count {
                s.execute(|| {
                    Self::run_thread(
                        data,
                        &self.states,
                        system_properties,
                        &self.entity_main_component_types,
                        &self.wrappers,
                    );
                });
            }
            Self::run_thread(
                data,
                &self.states,
                system_properties,
                &self.entity_main_component_types,
                &self.wrappers,
            );
        });
    }

    fn run_thread(
        data: &SystemData<'_>,
        states: &Mutex<SystemStateStorage>,
        system_properties: SystemProperties<'_>,
        entity_main_component_types: &TiSlice<SystemIdx, TypeId>,
        wrappers: &TiSlice<SystemIdx, Option<SystemWrapper>>,
    ) {
        let mut previous_system_idx = None;
        while let LockedSystem::Remaining(system_idx) =
            Self::lock_next_system(states, previous_system_idx, system_properties)
        {
            if let Some(system_idx) = system_idx {
                Self::run_system(system_idx, entity_main_component_types, wrappers, data);
            }
            previous_system_idx = system_idx;
        }
    }

    fn lock_next_system(
        states: &Mutex<SystemStateStorage>,
        previous_system_idx: Option<SystemIdx>,
        system_properties: SystemProperties<'_>,
    ) -> LockedSystem {
        states
            .lock()
            .expect("internal error: cannot lock states")
            .lock_next_system(previous_system_idx, system_properties)
    }

    fn run_system(
        system_idx: SystemIdx,
        entity_main_component_types: &TiSlice<SystemIdx, TypeId>,
        wrappers: &TiSlice<SystemIdx, Option<SystemWrapper>>,
        data: &SystemData<'_>,
    ) {
        let main_component_type = entity_main_component_types[system_idx];
        if data.components.count(main_component_type) == 0 {
            return;
        }
        let info = SystemInfo {
            filtered_component_types: vec![main_component_type],
        };
        let wrapper = wrappers[system_idx].expect("internal error: call missing system");
        wrapper(data, info);
    }

    fn all_idxs(
        wrappers: &TiVec<SystemIdx, Option<SystemWrapper>>,
    ) -> impl Iterator<Item = SystemIdx> + '_ {
        wrappers
            .iter_enumerated()
            .filter(|(_, w)| w.is_some())
            .map(|(s, _)| s)
    }
}

idx_type!(pub(crate) SystemIdx);

pub(super) struct FullSystemProperties {
    pub(super) component_types: Vec<ComponentTypeAccess>,
    pub(super) has_entity_actions: bool,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) struct ComponentTypeAccess {
    pub(super) idx: ComponentTypeIdx,
    pub(super) access: Access,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum Access {
    Read,
    Write,
}

#[cfg(test)]
mod system_storage_tests {
    use super::*;
    use crate::storages::archetypes::{
        ArchetypeEntityPos, ArchetypeIdx, ArchetypeStorage, EntityLocationInArchetype,
    };
    use crate::storages::components::ComponentStorage;
    use crate::storages::entity_actions::{EntityActionStorage, EntityState};
    use std::thread;
    use std::thread::ThreadId;
    use std::time::Duration;

    #[derive(Clone)]
    struct Component1(ThreadId);

    #[derive(Clone)]
    struct Component2(ThreadId);

    #[test]
    fn set_thread_count_to_0() {
        let mut storage = SystemStorage::default();

        storage.set_thread_count(0);

        assert_eq!(storage.thread_count(), 1);
        assert_eq!(storage.pool.map(|p| p.thread_count()), None);
    }

    #[test]
    fn set_thread_count_to_1() {
        let mut storage = SystemStorage::default();

        storage.set_thread_count(1);

        assert_eq!(storage.thread_count(), 1);
        assert_eq!(storage.pool.map(|p| p.thread_count()), None);
    }

    #[test]
    fn set_thread_count_to_more_than_1() {
        let mut storage = SystemStorage::default();

        storage.set_thread_count(3);

        assert_eq!(storage.thread_count(), 3);
        assert_eq!(storage.pool.map(|p| p.thread_count()), Some(2));
    }

    #[test]
    fn add_system() {
        let mut storage = SystemStorage::default();
        let wrapper: SystemWrapper = |_, _| ();
        let entity_main_component_type = TypeId::of::<u32>();
        let component_type_access = create_type_access(1.into(), Access::Read);
        let properties = create_properties(vec![component_type_access], true);

        let system_idx = storage.add(wrapper, entity_main_component_type, properties);

        assert_eq!(system_idx, 0.into());
        let component_state = storage.states.get_mut().unwrap();
        assert_eq!(component_state.last_component_type_idx(), Some(1.into()));
        assert!(storage.wrappers[system_idx].is_some());
        let component_types = ti_vec![vec![component_type_access]];
        assert_eq!(storage.component_types, component_types);
        assert_eq!(storage.have_entity_actions, ti_vec![true]);
        let filtered_component_types = ti_vec![TypeId::of::<u32>()];
        assert_eq!(
            storage.entity_main_component_types,
            filtered_component_types
        );
    }

    #[test]
    fn run_system_sequentially_without_existing_entity() {
        let mut storage = SystemStorage::default();
        storage.set_thread_count(1);
        let wrapper: SystemWrapper =
            |d, _| d.entity_actions.try_lock().unwrap().delete_entity(2.into());
        let entity_main_component_type = TypeId::of::<u32>();
        let component_type_access = create_type_access(1.into(), Access::Read);
        let properties = create_properties(vec![component_type_access], true);
        storage.add(wrapper, entity_main_component_type, properties);
        let data = SystemData {
            components: &ComponentStorage::default(),
            archetypes: &ArchetypeStorage::default(),
            entity_actions: &Mutex::new(EntityActionStorage::default()),
        };

        storage.run(&data);

        let mut entity_actions = data.entity_actions.try_lock().unwrap();
        assert_eq!(entity_actions.drain_entity_states().count(), 0);
    }

    #[test]
    fn run_system_in_parallel_without_existing_entity() {
        let mut storage = SystemStorage::default();
        storage.set_thread_count(2);
        let wrapper: SystemWrapper =
            |d, _| d.entity_actions.lock().unwrap().delete_entity(2.into());
        let entity_main_component_type = TypeId::of::<u32>();
        let component_type_access = create_type_access(1.into(), Access::Read);
        let properties = create_properties(vec![component_type_access], false);
        storage.add(wrapper, entity_main_component_type, properties);
        let wrapper: SystemWrapper =
            |d, _| d.entity_actions.lock().unwrap().delete_entity(3.into());
        let entity_main_component_type = TypeId::of::<i64>();
        let component_type_access = create_type_access(3.into(), Access::Write);
        let properties = create_properties(vec![component_type_access], true);
        storage.add(wrapper, entity_main_component_type, properties);
        let data = SystemData {
            components: &ComponentStorage::default(),
            archetypes: &ArchetypeStorage::default(),
            entity_actions: &Mutex::new(EntityActionStorage::default()),
        };

        storage.run(&data);

        let mut entity_actions = data.entity_actions.try_lock().unwrap();
        assert_eq!(entity_actions.drain_entity_states().count(), 0);
    }

    #[test]
    fn run_system_sequentially_with_existing_entity() {
        let mut storage = SystemStorage::default();
        storage.set_thread_count(1);
        let wrapper: SystemWrapper = |d, i| {
            assert_eq!(i.filtered_component_types, [TypeId::of::<u32>()]);
            d.entity_actions.try_lock().unwrap().delete_entity(2.into());
        };
        let entity_main_component_type = TypeId::of::<u32>();
        let component_type_access = create_type_access(1.into(), Access::Read);
        let properties = create_properties(vec![component_type_access], true);
        storage.add(wrapper, entity_main_component_type, properties);
        let mut components = ComponentStorage::default();
        let type_idx = components.type_idx_or_create::<u32>();
        let location = EntityLocationInArchetype::new(0.into(), 0.into());
        components.add(type_idx, location, 10_u32);
        let data = SystemData {
            components: &components,
            archetypes: &ArchetypeStorage::default(),
            entity_actions: &Mutex::new(EntityActionStorage::default()),
        };

        storage.run(&data);

        let mut entity_actions = data.entity_actions.try_lock().unwrap();
        let entity_states: Vec<_> = entity_actions.drain_entity_states().collect();
        assert_eq!(entity_states[0].0, 2.into());
        assert!(matches!(entity_states[0].1, EntityState::Deleted));
    }

    #[test]
    fn run_system_in_parallel_with_existing_entity() {
        let mut storage = SystemStorage::default();
        storage.set_thread_count(2);
        let wrapper1: SystemWrapper = |d, _| {
            let thread_id = Component1(thread::current().id());
            d.components.write_components()[ArchetypeIdx(0)].push(thread_id);
            thread::sleep(Duration::from_millis(1));
        };
        let wrapper2: SystemWrapper = |d, _| {
            let thread_id = Component2(thread::current().id());
            d.components.write_components()[ArchetypeIdx(0)].push(thread_id);
            thread::sleep(Duration::from_millis(1));
        };
        let entity_main_component_type1 = TypeId::of::<Component1>();
        let entity_main_component_type2 = TypeId::of::<Component2>();
        let properties1 = create_properties(vec![], false);
        let properties2 = create_properties(vec![], true);
        storage.add(wrapper1, entity_main_component_type1, properties1);
        storage.add(wrapper2, entity_main_component_type2, properties2);
        let mut components = ComponentStorage::default();
        let type1_idx = components.type_idx_or_create::<Component1>();
        let type2_idx = components.type_idx_or_create::<Component2>();
        let location = EntityLocationInArchetype::new(0.into(), 0.into());
        components.add(type1_idx, location, Component1(thread::current().id()));
        components.add(type2_idx, location, Component2(thread::current().id()));
        let data = SystemData {
            components: &components,
            archetypes: &ArchetypeStorage::default(),
            entity_actions: &Mutex::new(EntityActionStorage::default()),
        };

        storage.run(&data);

        let component1_guard = components.read_components::<Component1>().clone();
        let component2_guard = components.read_components::<Component2>().clone();
        assert_ne!(
            component1_guard[ArchetypeIdx(0)][ArchetypeEntityPos(1)].0,
            component2_guard[ArchetypeIdx(0)][ArchetypeEntityPos(1)].0
        );
    }

    fn create_type_access(type_idx: ComponentTypeIdx, access: Access) -> ComponentTypeAccess {
        ComponentTypeAccess {
            idx: type_idx,
            access,
        }
    }

    fn create_properties(
        component_types: Vec<ComponentTypeAccess>,
        has_entity_actions: bool,
    ) -> FullSystemProperties {
        FullSystemProperties {
            component_types,
            has_entity_actions,
        }
    }
}
