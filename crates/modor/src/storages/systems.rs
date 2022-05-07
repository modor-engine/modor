use crate::storages::actions::ActionIdx;
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
    filtered_component_type_idxs: TiVec<SystemIdx, Vec<ComponentTypeIdx>>,
    states: Mutex<SystemStateStorage>,
    pool: Option<Pool>,
}

impl SystemStorage {
    pub(crate) fn thread_count(&self) -> u32 {
        self.pool.as_ref().map_or(0, Pool::thread_count) + 1
    }

    pub(super) fn set_thread_count(&mut self, count: u32) {
        #[cfg(not(target_arch = "wasm32"))]
        {
            if count < 2 {
                self.pool = None;
            } else {
                self.pool = Some(Pool::new(count - 1));
            }
        }
        #[cfg(target_arch = "wasm32")]
        {
            self.pool = None;
        }
    }

    pub(super) fn add(
        &mut self,
        wrapper: SystemWrapper,
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
        self.filtered_component_type_idxs
            .push_and_get_key(properties.filtered_component_type_idxs)
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
            &self.filtered_component_type_idxs,
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
                        &self.filtered_component_type_idxs,
                        &self.wrappers,
                    );
                });
            }
            Self::run_thread(
                data,
                &self.states,
                &self.filtered_component_type_idxs,
                &self.wrappers,
            );
        });
    }

    fn run_thread(
        data: SystemData<'_>,
        states: &Mutex<SystemStateStorage>,
        filtered_component_type_idxs: &TiSlice<SystemIdx, Vec<ComponentTypeIdx>>,
        wrappers: &TiSlice<SystemIdx, SystemWrapper>,
    ) {
        let mut previous_system_idx = None;
        while let LockedSystem::Remaining(system_idx) =
            Self::lock_next_system(data, states, previous_system_idx)
        {
            if let Some(system_idx) = system_idx {
                Self::run_system(system_idx, filtered_component_type_idxs, wrappers, data);
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
        filtered_component_type_idxs: &TiSlice<SystemIdx, Vec<ComponentTypeIdx>>,
        wrappers: &TiSlice<SystemIdx, SystemWrapper>,
        data: SystemData<'_>,
    ) {
        let filtered_type_idxs = &filtered_component_type_idxs[system_idx];
        let info = SystemInfo {
            filtered_component_type_idxs: filtered_type_idxs,
            item_count: data.item_count(filtered_type_idxs),
        };
        (wrappers[system_idx])(data, info);
    }
}

idx_type!(pub(crate) SystemIdx);

pub struct SystemProperties {
    pub(crate) component_types: Vec<ComponentTypeAccess>,
    pub(crate) can_update: bool,
    pub(crate) filtered_component_type_idxs: Vec<ComponentTypeIdx>,
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
        ArchetypeEntityPos, ArchetypeIdx, ArchetypeStorage, EntityLocation,
    };
    use crate::storages::components::{ComponentStorage, ComponentTypeIdx};
    use crate::storages::entities::EntityStorage;
    use crate::storages::systems::{
        Access, ComponentTypeAccess, SystemIdx, SystemProperties, SystemStorage,
    };
    use crate::storages::updates::UpdateStorage;
    use crate::systems::internal::SystemWrapper;
    use crate::{SystemData, SystemInfo};
    use std::any::{Any, TypeId};
    use std::sync::Mutex;
    use std::thread;
    use std::thread::ThreadId;

    impl SystemStorage {
        pub(crate) fn filtered_component_idxs(&self, system_idx: SystemIdx) -> &[ComponentTypeIdx] {
            &self.filtered_component_type_idxs[system_idx]
        }
    }

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

    fn system_wrapper<C1, C2>(data: SystemData<'_>, _info: SystemInfo<'_>)
    where
        C1: From<ThreadId> + Any,
        C2: Any,
    {
        loop {
            if let Some(mut components) = data.components.try_write_components() {
                let thread_id = C1::from(thread::current().id());
                let type_idx = data.components.type_idx(TypeId::of::<C1>()).unwrap();
                let archetype_idx = *data
                    .components
                    .sorted_archetype_idxs(type_idx)
                    .first()
                    .unwrap();
                components[archetype_idx].push(thread_id);
                break;
            }
        }
        loop {
            if let Some(components) = data.components.try_write_components::<C2>() {
                let type_idx = data.components.type_idx(TypeId::of::<C2>()).unwrap();
                let archetype_idx = *data
                    .components
                    .sorted_archetype_idxs(type_idx)
                    .first()
                    .unwrap();
                if components[archetype_idx].len() > 1 {
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
    fn run_systems_sequentially() {
        let mut storage = SystemStorage::default();
        storage.set_thread_count(1);
        let wrapper: SystemWrapper = |d, i| {
            assert_eq!(i.filtered_component_type_idxs, [0.into()]);
            assert_eq!(i.item_count, 1);
            d.updates.try_lock().unwrap().delete_entity(2.into());
        };
        let component_type_access = create_type_access(Access::Read, 1.into());
        let properties = create_properties(vec![component_type_access]);
        storage.add(wrapper, properties, 0.into());
        use_data(false, |d| {
            storage.run(d);
            let mut updates = d.updates.try_lock().unwrap();
            let deleted_entity_idxs: Vec<_> = updates.deleted_entity_drain().collect();
            assert_eq!(deleted_entity_idxs, [2.into()]);
        });
    }

    #[test]
    fn run_systems_in_parallel() {
        let mut storage = SystemStorage::default();
        storage.set_thread_count(2);
        let wrapper1: SystemWrapper = system_wrapper::<Component1, Component2>;
        let properties1 = create_properties(vec![]);
        storage.add(wrapper1, properties1, 0.into());
        let wrapper2: SystemWrapper = system_wrapper::<Component2, Component1>;
        let properties2 = create_properties(vec![]);
        storage.add(wrapper2, properties2, 0.into());
        use_data(false, |d| {
            storage.run(d);
            let component1_guard = d.components.read_components::<Component1>().clone();
            let component2_guard = d.components.read_components::<Component2>().clone();
            assert_ne!(
                component1_guard[ArchetypeIdx(1)][ArchetypeEntityPos(1)].0,
                component2_guard[ArchetypeIdx(2)][ArchetypeEntityPos(1)].0
            );
        });
    }

    fn use_data<F>(empty_storage: bool, mut f: F)
    where
        F: FnMut(SystemData<'_>),
    {
        let mut archetypes = ArchetypeStorage::default();
        let mut components = ComponentStorage::default();
        let type1_idx = components.type_idx_or_create::<Component1>();
        let type2_idx = components.type_idx_or_create::<Component2>();
        if !empty_storage {
            let archetype1_idx = archetypes
                .add_component(ArchetypeStorage::DEFAULT_IDX, type1_idx)
                .unwrap();
            let archetype2_idx = archetypes
                .add_component(ArchetypeStorage::DEFAULT_IDX, type2_idx)
                .unwrap();
            archetypes.add_entity(0.into(), archetype1_idx);
            archetypes.add_entity(1.into(), archetype2_idx);
            let thread_id = thread::current().id();
            let location1 = EntityLocation::new(archetype1_idx, 0.into());
            components.add(type1_idx, location1, Component1(thread_id), false);
            let location2 = EntityLocation::new(archetype2_idx, 0.into());
            components.add(type2_idx, location2, Component2(thread_id), false);
        }
        let mut actions = ActionStorage::default();
        let action_idx = actions.idx_or_create(None, ActionDependencies::Types(vec![]));
        actions.add_system(action_idx);
        actions.add_system(action_idx);
        let data = SystemData {
            entities: &EntityStorage::default(),
            components: &components,
            archetypes: &archetypes,
            actions: &actions,
            updates: &Mutex::new(UpdateStorage::default()),
        };
        f(data);
    }

    fn create_type_access(access: Access, type_idx: ComponentTypeIdx) -> ComponentTypeAccess {
        ComponentTypeAccess { access, type_idx }
    }

    fn create_properties(component_types: Vec<ComponentTypeAccess>) -> SystemProperties {
        SystemProperties {
            component_types,
            can_update: false,
            filtered_component_type_idxs: vec![0.into()],
        }
    }
}
