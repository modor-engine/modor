use crate::storages::actions::ActionIdx;
use crate::storages::archetypes::ArchetypeFilter;
use crate::storages::components::ComponentTypeIdx;
use crate::storages::system_states::{LockedSystem, SystemStateStorage};
use crate::systems::internal::SystemWrapper;
use crate::{SystemData, SystemInfo};
use scoped_threadpool::Pool;
use std::sync::Mutex;
use typed_index_collections::{TiSlice, TiVec};

#[derive(Default)]
pub(crate) struct SystemStorage {
    wrappers: TiVec<SystemIdx, SystemWrapper>,
    entity_type_idxs: TiVec<SystemIdx, ComponentTypeIdx>,
    archetype_filters: TiVec<SystemIdx, ArchetypeFilter>,
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
        self.states
            .get_mut()
            .expect("internal error: cannot access to system states to register component type")
            .add_system(
                properties.component_types,
                properties.can_update,
                action_idx,
            );
        self.wrappers.push(wrapper);
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
        Self::run_thread(
            data,
            &self.states,
            &self.archetype_filters,
            &self.entity_type_idxs,
            &self.wrappers,
        );
    }

    fn run_in_parallel(&mut self, pool: &mut Pool, data: SystemData<'_>) {
        let thread_count = pool.thread_count();
        pool.scoped(|s| {
            for _ in 0..thread_count {
                s.execute(|| {
                    Self::run_thread(
                        data,
                        &self.states,
                        &self.archetype_filters,
                        &self.entity_type_idxs,
                        &self.wrappers,
                    );
                });
            }
            Self::run_thread(
                data,
                &self.states,
                &self.archetype_filters,
                &self.entity_type_idxs,
                &self.wrappers,
            );
        });
    }

    fn run_thread(
        data: SystemData<'_>,
        states: &Mutex<SystemStateStorage>,
        archetype_filters: &TiSlice<SystemIdx, ArchetypeFilter>,
        entity_type_idxs: &TiSlice<SystemIdx, ComponentTypeIdx>,
        wrappers: &TiSlice<SystemIdx, SystemWrapper>,
    ) {
        let mut previous_system_idx = None;
        while let LockedSystem::Remaining(system_idx) =
            Self::lock_next_system(data, states, previous_system_idx)
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
    ) -> LockedSystem {
        states
            .lock()
            .expect("internal error: cannot lock states")
            .lock_next_system(previous_system_idx, data.actions)
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
    use crate::storages::actions::{ActionDependencies, ActionStorage};
    use crate::storages::archetypes::{
        ArchetypeEntityPos, ArchetypeFilter, ArchetypeIdx, ArchetypeStorage, EntityLocation,
    };
    use crate::storages::components::{ComponentStorage, ComponentTypeIdx};
    use crate::storages::entities::EntityStorage;
    use crate::storages::systems::{Access, ComponentTypeAccess, SystemProperties, SystemStorage};
    use crate::storages::updates::{EntityUpdate, UpdateStorage};
    use crate::systems::internal::SystemWrapper;
    use crate::{SystemData, SystemInfo};
    use std::any::Any;
    use std::sync::Mutex;
    use std::thread;
    use std::thread::ThreadId;

    #[derive(Clone)]
    struct Component1(ThreadId);

    impl From<ThreadId> for Component1 {
        fn from(id: ThreadId) -> Self {
            Self(id)
        }
    }

    #[derive(Clone)]
    struct Component2(ThreadId);

    impl From<ThreadId> for Component2 {
        fn from(id: ThreadId) -> Self {
            Self(id)
        }
    }

    fn system_wrapper<C1, C2>(data: SystemData, _info: SystemInfo)
    where
        C1: From<ThreadId> + Any,
        C2: Any,
    {
        loop {
            if let Some(mut components) = data.components.try_write_components() {
                let thread_id = C1::from(thread::current().id());
                components[ArchetypeIdx(0)].push(thread_id);
                break;
            }
        }
        loop {
            if let Some(components) = data.components.try_write_components::<C2>() {
                if components[ArchetypeIdx(0)].len() > 1 {
                    break;
                }
            }
        }
    }

    #[test]
    fn configure_thread_count() {
        let mut storage = SystemStorage::default();
        storage.set_thread_count(0);
        assert_eq!(storage.thread_count(), 1);
        storage.set_thread_count(1);
        assert_eq!(storage.thread_count(), 1);
        storage.set_thread_count(3);
        assert_eq!(storage.thread_count(), 3);
    }

    #[test]
    fn run_system_sequentially_with_existing_entity() {
        let mut storage = SystemStorage::default();
        storage.set_thread_count(1);
        let wrapper: SystemWrapper = |d, i| {
            assert_eq!(i.filtered_component_type_idxs, [0.into()]);
            assert_eq!(i.archetype_filter, &ArchetypeFilter::None);
            assert_eq!(i.item_count, 1);
            d.updates.try_lock().unwrap().delete_entity(2.into());
        };
        let component_type_access = create_type_access(Access::Read, 1.into());
        let properties = create_properties(vec![component_type_access]);
        storage.add(wrapper, 0.into(), properties, 0.into());
        use_data(false, |d| {
            storage.run(d);
            let mut updates = d.updates.try_lock().unwrap();
            assert_eq!(updates.drain_entity_updates().count(), 0);
        });
        use_data(true, |d| {
            storage.run(d);
            let mut updates = d.updates.try_lock().unwrap();
            let entity_updates: Vec<_> = updates.drain_entity_updates().collect();
            assert_eq!(entity_updates[0].0, 2.into());
            assert!(matches!(entity_updates[0].1, EntityUpdate::Deletion));
        });
    }

    #[test]
    fn run_system_in_parallel_with_existing_entity() {
        let mut storage = SystemStorage::default();
        storage.set_thread_count(2);
        let wrapper1: SystemWrapper = system_wrapper::<Component1, Component2>;
        let wrapper2: SystemWrapper = system_wrapper::<Component2, Component1>;
        let properties1 = create_properties(vec![]);
        let properties2 = create_properties(vec![]);
        storage.add(wrapper1, 0.into(), properties1, 0.into());
        storage.add(wrapper2, 1.into(), properties2, 0.into());
        use_data(false, |d| {
            storage.run(d);
            assert_eq!(d.components.read_components::<Component1>().len(), 0);
            assert_eq!(d.components.read_components::<Component2>().len(), 0);
        });
        use_data(true, |d| {
            storage.run(d);
            let component1_guard = d.components.read_components::<Component1>().clone();
            let component2_guard = d.components.read_components::<Component2>().clone();
            assert_ne!(
                component1_guard[ArchetypeIdx(0)][ArchetypeEntityPos(1)].0,
                component2_guard[ArchetypeIdx(0)][ArchetypeEntityPos(1)].0
            );
        });
    }

    fn use_data<F>(create_components: bool, mut f: F)
    where
        F: FnMut(SystemData),
    {
        let mut components = ComponentStorage::default();
        let type1_idx = components.type_idx_or_create::<Component1>();
        let type2_idx = components.type_idx_or_create::<Component2>();
        let location = EntityLocation::new(0.into(), 0.into());
        if create_components {
            components.add(type1_idx, location, Component1(thread::current().id()));
            components.add(type2_idx, location, Component2(thread::current().id()));
        }
        let mut actions = ActionStorage::default();
        let action_idx = actions.idx_or_create(None, ActionDependencies::Types(vec![]));
        actions.add_system(action_idx);
        actions.add_system(action_idx);
        let data = SystemData {
            entities: &EntityStorage::default(),
            components: &components,
            archetypes: &ArchetypeStorage::default(),
            actions: &actions,
            updates: &Mutex::new(UpdateStorage::default()),
        };
        f(data)
    }

    fn create_type_access(access: Access, type_idx: ComponentTypeIdx) -> ComponentTypeAccess {
        ComponentTypeAccess { access, type_idx }
    }

    fn create_properties(component_types: Vec<ComponentTypeAccess>) -> SystemProperties {
        SystemProperties {
            component_types,
            can_update: false,
            archetype_filter: ArchetypeFilter::None,
        }
    }
}
