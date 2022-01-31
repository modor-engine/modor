use crate::storages::actions::ActionIdx;
use crate::storages::archetypes::ArchetypeFilter;
use crate::storages::components::ComponentTypeIdx;
use crate::storages::system_states::{AllSystemProperties, LockedSystem, SystemStateStorage};
use crate::systems::internal::SystemWrapper;
use crate::{SystemData, SystemInfo};
use scoped_threadpool::Pool;
use std::sync::Mutex;
use typed_index_collections::{TiSlice, TiVec};

#[derive(Default)]
pub(crate) struct SystemStorage {
    wrappers: TiVec<SystemIdx, SystemWrapper>,
    component_types: TiVec<SystemIdx, Vec<ComponentTypeAccess>>,
    can_update: TiVec<SystemIdx, bool>,
    entity_type_idxs: TiVec<SystemIdx, ComponentTypeIdx>,
    archetype_filters: TiVec<SystemIdx, ArchetypeFilter>,
    action_idxs: TiVec<SystemIdx, ActionIdx>,
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
        entity_type_idx: ComponentTypeIdx,
        properties: SystemProperties,
        action_idx: ActionIdx,
    ) -> SystemIdx {
        let states = self
            .states
            .get_mut()
            .expect("internal error: cannot access to system states to register component type");
        for component_types in &properties.component_types {
            states.register_component_type(component_types.type_idx);
        }
        self.action_idxs.push(action_idx);
        self.wrappers.push(wrapper);
        self.component_types.push(properties.component_types);
        self.can_update.push(properties.can_update);
        self.archetype_filters.push(properties.archetype_filter);
        self.entity_type_idxs.push_and_get_key(entity_type_idx)
    }

    pub(super) fn run(&mut self, data: SystemData<'_>) {
        self.states
            .get_mut()
            .expect("internal error: cannot reset states")
            .reset(self.wrappers.keys(), data.actions.system_counts());
        if let Some(mut pool) = self.pool.take() {
            self.run_in_parallel(&mut pool, data);
            self.pool = Some(pool);
        } else {
            self.run_sequentially(data);
        }
    }

    fn run_sequentially(&mut self, data: SystemData<'_>) {
        let system_properties = AllSystemProperties {
            component_types: &self.component_types,
            can_update: &self.can_update,
            action_idxs: &self.action_idxs,
        };
        Self::run_thread(
            data,
            &self.states,
            system_properties,
            &self.archetype_filters,
            &self.entity_type_idxs,
            &self.wrappers,
        );
    }

    fn run_in_parallel(&mut self, pool: &mut Pool, data: SystemData<'_>) {
        let system_properties = AllSystemProperties {
            component_types: &self.component_types,
            can_update: &self.can_update,
            action_idxs: &self.action_idxs,
        };
        let thread_count = pool.thread_count();
        pool.scoped(|s| {
            for _ in 0..thread_count {
                s.execute(|| {
                    Self::run_thread(
                        data,
                        &self.states,
                        system_properties,
                        &self.archetype_filters,
                        &self.entity_type_idxs,
                        &self.wrappers,
                    );
                });
            }
            Self::run_thread(
                data,
                &self.states,
                system_properties,
                &self.archetype_filters,
                &self.entity_type_idxs,
                &self.wrappers,
            );
        });
    }

    fn run_thread(
        data: SystemData<'_>,
        states: &Mutex<SystemStateStorage>,
        system_properties: AllSystemProperties<'_>,
        archetype_filters: &TiSlice<SystemIdx, ArchetypeFilter>,
        entity_type_idxs: &TiSlice<SystemIdx, ComponentTypeIdx>,
        wrappers: &TiSlice<SystemIdx, SystemWrapper>,
    ) {
        let mut previous_system_idx = None;
        while let LockedSystem::Remaining(system_idx) =
            Self::lock_next_system(data, states, previous_system_idx, system_properties)
        {
            if let Some(system_idx) = system_idx {
                Self::run_system(
                    system_idx,
                    archetype_filters,
                    entity_type_idxs,
                    wrappers,
                    data,
                );
            }
            previous_system_idx = system_idx;
        }
    }

    fn lock_next_system(
        data: SystemData<'_>,
        states: &Mutex<SystemStateStorage>,
        previous_system_idx: Option<SystemIdx>,
        system_properties: AllSystemProperties<'_>,
    ) -> LockedSystem {
        states
            .lock()
            .expect("internal error: cannot lock states")
            .lock_next_system(previous_system_idx, system_properties, data.actions)
    }

    fn run_system(
        system_idx: SystemIdx,
        archetype_filters: &TiSlice<SystemIdx, ArchetypeFilter>,
        entity_type_idxs: &TiSlice<SystemIdx, ComponentTypeIdx>,
        wrappers: &TiSlice<SystemIdx, SystemWrapper>,
        data: SystemData<'_>,
    ) {
        let entity_type_idx = entity_type_idxs[system_idx];
        if data.components.count(entity_type_idx) == 0 {
            return;
        }
        let filtered_component_type_idxs = &[entity_type_idx];
        let archetype_filter = &archetype_filters[system_idx];
        let info = SystemInfo {
            filtered_component_type_idxs,
            archetype_filter,
            item_count: data.item_count(filtered_component_type_idxs, archetype_filter),
        };
        (wrappers[system_idx])(data, info);
    }
}

idx_type!(pub(crate) SystemIdx);

pub struct SystemProperties {
    pub(crate) component_types: Vec<ComponentTypeAccess>,
    pub(crate) can_update: bool,
    pub(crate) archetype_filter: ArchetypeFilter,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub(crate) struct ComponentTypeAccess {
    pub(crate) access: Access,
    pub(crate) type_idx: ComponentTypeIdx,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum Access {
    Read,
    Write,
}

#[cfg(test)]
mod system_storage_tests {
    use crate::storages::actions::{ActionDefinition, ActionDependencies, ActionStorage};
    use crate::storages::archetypes::{
        ArchetypeEntityPos, ArchetypeFilter, ArchetypeIdx, ArchetypeStorage,
        EntityLocation,
    };
    use crate::storages::components::{ComponentStorage, ComponentTypeIdx};
    use crate::storages::entities::EntityStorage;
    use crate::storages::systems::{Access, ComponentTypeAccess, SystemProperties, SystemStorage};
    use crate::storages::updates::{EntityUpdate, UpdateStorage};
    use crate::systems::internal::SystemWrapper;
    use crate::SystemData;
    use std::sync::Mutex;
    use std::thread;
    use std::thread::ThreadId;

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
        let component_type_access = create_type_access(Access::Read, 1.into());
        let properties = create_properties(vec![component_type_access], true);

        let system_idx = storage.add(wrapper, 0.into(), properties, 1.into());

        assert_eq!(system_idx, 0.into());
        let component_state = storage.states.get_mut().unwrap();
        assert_eq!(component_state.last_component_type_idx(), Some(1.into()));
        assert_eq!(storage.wrappers.len(), 1);
        let component_types = ti_vec![vec![component_type_access]];
        assert_eq!(storage.component_types, component_types);
        assert_eq!(storage.can_update, ti_vec![true]);
        let filtered_component_types = ti_vec![0.into()];
        assert_eq!(storage.entity_type_idxs, filtered_component_types);
        assert_eq!(storage.action_idxs, ti_vec![1.into()]);
    }

    #[test]
    fn run_system_sequentially_without_existing_entity() {
        let mut storage = SystemStorage::default();
        storage.set_thread_count(1);
        let wrapper: SystemWrapper = |d, _| d.updates.try_lock().unwrap().delete_entity(2.into());
        let component_type_access = create_type_access(Access::Read, 1.into());
        let properties = create_properties(vec![component_type_access], true);
        storage.add(wrapper, 0.into(), properties, 0.into());
        let mut actions = ActionStorage::default();
        let action_idx = actions.idx_or_create(ActionDefinition {
            type_: None,
            dependency_types: ActionDependencies::Types(vec![]),
        });
        actions.add_system(action_idx);
        let data = SystemData {
            entities: &EntityStorage::default(),
            components: &ComponentStorage::default(),
            archetypes: &ArchetypeStorage::default(),
            actions: &actions,
            updates: &Mutex::new(UpdateStorage::default()),
        };

        storage.run(data);

        let mut updates = data.updates.try_lock().unwrap();
        assert_eq!(updates.drain_entity_updates().count(), 0);
    }

    #[test]
    fn run_system_in_parallel_without_existing_entity() {
        let mut storage = SystemStorage::default();
        storage.set_thread_count(2);
        let wrapper: SystemWrapper = |d, _| d.updates.lock().unwrap().delete_entity(2.into());
        let component_type_access = create_type_access(Access::Read, 1.into());
        let properties = create_properties(vec![component_type_access], false);
        storage.add(wrapper, 0.into(), properties, 0.into());
        let wrapper: SystemWrapper = |d, _| d.updates.lock().unwrap().delete_entity(3.into());
        let component_type_access = create_type_access(Access::Write, 3.into());
        let properties = create_properties(vec![component_type_access], true);
        storage.add(wrapper, 1.into(), properties, 0.into());
        let mut actions = ActionStorage::default();
        let action_idx = actions.idx_or_create(ActionDefinition {
            type_: None,
            dependency_types: ActionDependencies::Types(vec![]),
        });
        actions.add_system(action_idx);
        actions.add_system(action_idx);
        let data = SystemData {
            entities: &EntityStorage::default(),
            components: &ComponentStorage::default(),
            archetypes: &ArchetypeStorage::default(),
            actions: &actions,
            updates: &Mutex::new(UpdateStorage::default()),
        };

        storage.run(data);

        let mut updates = data.updates.try_lock().unwrap();
        assert_eq!(updates.drain_entity_updates().count(), 0);
    }

    #[test]
    fn run_system_sequentially_with_existing_entity() {
        let mut storage = SystemStorage::default();
        storage.set_thread_count(1);
        let wrapper: SystemWrapper = |d, i| {
            assert_eq!(i.filtered_component_type_idxs, [0.into()]);
            d.updates.try_lock().unwrap().delete_entity(2.into());
        };
        let component_type_access = create_type_access(Access::Read, 1.into());
        let properties = create_properties(vec![component_type_access], true);
        storage.add(wrapper, 0.into(), properties, 0.into());
        let mut components = ComponentStorage::default();
        let type_idx = components.type_idx_or_create::<u32>();
        let location = EntityLocation::new(0.into(), 0.into());
        components.add(type_idx, location, 10_u32);
        let mut actions = ActionStorage::default();
        let action_idx = actions.idx_or_create(ActionDefinition {
            type_: None,
            dependency_types: ActionDependencies::Types(vec![]),
        });
        actions.add_system(action_idx);
        let data = SystemData {
            entities: &EntityStorage::default(),
            components: &components,
            archetypes: &ArchetypeStorage::default(),
            actions: &actions,
            updates: &Mutex::new(UpdateStorage::default()),
        };

        storage.run(data);

        let mut updates = data.updates.try_lock().unwrap();
        let entity_updates: Vec<_> = updates.drain_entity_updates().collect();
        assert_eq!(entity_updates[0].0, 2.into());
        assert!(matches!(entity_updates[0].1, EntityUpdate::Deleted));
    }

    #[test]
    fn run_system_in_parallel_with_existing_entity() {
        let mut storage = SystemStorage::default();
        storage.set_thread_count(2);
        let wrapper1: SystemWrapper = |d, _| {
            loop {
                if let Some(mut components) = d.components.try_write_components() {
                    let thread_id = Component1(thread::current().id());
                    components[ArchetypeIdx(0)].push(thread_id);
                    break;
                }
            }
            loop {
                if let Some(components) = d.components.try_write_components::<Component2>() {
                    if components[ArchetypeIdx(0)].len() > 1 {
                        break;
                    }
                }
            }
        };
        let wrapper2: SystemWrapper = |d, _| {
            loop {
                if let Some(mut components) = d.components.try_write_components() {
                    let thread_id = Component2(thread::current().id());
                    components[ArchetypeIdx(0)].push(thread_id);
                    break;
                }
            }
            loop {
                if let Some(components) = d.components.try_write_components::<Component1>() {
                    if components[ArchetypeIdx(0)].len() > 1 {
                        break;
                    }
                }
            }
        };
        let properties1 = create_properties(vec![], false);
        let properties2 = create_properties(vec![], true);
        storage.add(wrapper1, 0.into(), properties1, 0.into());
        storage.add(wrapper2, 1.into(), properties2, 0.into());
        let mut components = ComponentStorage::default();
        let type1_idx = components.type_idx_or_create::<Component1>();
        let type2_idx = components.type_idx_or_create::<Component2>();
        let location = EntityLocation::new(0.into(), 0.into());
        components.add(type1_idx, location, Component1(thread::current().id()));
        components.add(type2_idx, location, Component2(thread::current().id()));
        let mut actions = ActionStorage::default();
        let action_idx = actions.idx_or_create(ActionDefinition {
            type_: None,
            dependency_types: ActionDependencies::Types(vec![]),
        });
        actions.add_system(action_idx);
        actions.add_system(action_idx);
        let data = SystemData {
            entities: &EntityStorage::default(),
            components: &components,
            archetypes: &ArchetypeStorage::default(),
            actions: &actions,
            updates: &Mutex::new(UpdateStorage::default()),
        };

        storage.run(data);

        let component1_guard = components.read_components::<Component1>().clone();
        let component2_guard = components.read_components::<Component2>().clone();
        assert_ne!(
            component1_guard[ArchetypeIdx(0)][ArchetypeEntityPos(1)].0,
            component2_guard[ArchetypeIdx(0)][ArchetypeEntityPos(1)].0
        );
    }

    fn create_type_access(access: Access, type_idx: ComponentTypeIdx) -> ComponentTypeAccess {
        ComponentTypeAccess { access, type_idx }
    }

    fn create_properties(
        component_types: Vec<ComponentTypeAccess>,
        can_update: bool,
    ) -> SystemProperties {
        SystemProperties {
            component_types,
            can_update,
            archetype_filter: ArchetypeFilter::None,
        }
    }
}
