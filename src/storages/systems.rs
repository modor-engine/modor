use crate::storages::actions::ActionIdx;
use crate::storages::archetypes::ArchetypeFilter;
use crate::storages::components::ComponentTypeIdx;
use crate::storages::globals::GlobalIdx;
use crate::storages::system_states::{LockedSystem, SystemStateStorage};
use crate::systems::internal::SystemWrapper;
use crate::{SystemData, SystemInfo};
use scoped_threadpool::Pool;
use std::sync::Mutex;
use typed_index_collections::{TiSlice, TiVec};

#[derive(Default)]
pub(crate) struct SystemStorage {
    wrappers: TiVec<SystemIdx, SystemWrapper>,
    callers: TiVec<SystemIdx, SystemCaller>,
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
        caller: SystemCaller,
        properties: SystemProperties,
        action_idx: ActionIdx,
    ) -> SystemIdx {
        self.states
            .get_mut()
            .expect("internal error: cannot access to system states to register component type")
            .add_system(
                properties.component_types,
                properties.globals,
                properties.can_update,
                action_idx,
            );
        self.wrappers.push(wrapper);
        self.archetype_filters.push(properties.archetype_filter);
        self.callers.push_and_get_key(caller)
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
            &self.callers,
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
                        &self.callers,
                        &self.wrappers,
                    );
                });
            }
            Self::run_thread(
                data,
                &self.states,
                &self.archetype_filters,
                &self.callers,
                &self.wrappers,
            );
        });
    }

    fn run_thread(
        data: SystemData<'_>,
        states: &Mutex<SystemStateStorage>,
        archetype_filters: &TiSlice<SystemIdx, ArchetypeFilter>,
        callers: &TiSlice<SystemIdx, SystemCaller>,
        wrappers: &TiSlice<SystemIdx, SystemWrapper>,
    ) {
        let mut previous_system_idx = None;
        while let LockedSystem::Remaining(system_idx) =
            Self::lock_next_system(data, states, previous_system_idx)
        {
            if let Some(system_idx) = system_idx {
                Self::run_system(system_idx, archetype_filters, callers, wrappers, data);
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
        callers: &TiSlice<SystemIdx, SystemCaller>,
        wrappers: &TiSlice<SystemIdx, SystemWrapper>,
        data: SystemData<'_>,
    ) {
        let caller = callers[system_idx];
        if match caller {
            SystemCaller::Entity(e) => data.components.count(e) == 0,
            SystemCaller::Global(g) => !data.globals.exists(g),
        } {
            return;
        }
        if let SystemCaller::Entity(e) = caller {
            let filtered_component_type_idxs = &[e];
            let archetype_filter = &archetype_filters[system_idx];
            let info = SystemInfo {
                filtered_component_type_idxs,
                archetype_filter,
                item_count: data.item_count(filtered_component_type_idxs, archetype_filter),
            };
            (wrappers[system_idx])(data, info);
        } else {
            let filtered_component_type_idxs = &[];
            let archetype_filter = &archetype_filters[system_idx];
            let info = SystemInfo {
                filtered_component_type_idxs,
                archetype_filter,
                item_count: data.item_count(filtered_component_type_idxs, archetype_filter),
            };
            (wrappers[system_idx])(data, info);
        }
    }
}

idx_type!(pub(crate) SystemIdx);

pub struct SystemProperties {
    pub(crate) component_types: Vec<ComponentTypeAccess>,
    pub(crate) globals: Vec<GlobalAccess>,
    pub(crate) can_update: bool,
    pub(crate) archetype_filter: ArchetypeFilter,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub(crate) struct ComponentTypeAccess {
    pub(crate) access: Access,
    pub(crate) type_idx: ComponentTypeIdx,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub(crate) struct GlobalAccess {
    pub(crate) access: Access,
    pub(crate) idx: GlobalIdx,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum Access {
    Read,
    Write,
}

#[derive(Clone, Copy)]
pub(crate) enum SystemCaller {
    Entity(ComponentTypeIdx),
    Global(GlobalIdx),
}

#[cfg(test)]
mod system_storage_tests {
    use crate::storages::actions::{ActionDependencies, ActionStorage};
    use crate::storages::archetypes::{
        ArchetypeEntityPos, ArchetypeFilter, ArchetypeIdx, ArchetypeStorage, EntityLocation,
    };
    use crate::storages::components::{ComponentStorage, ComponentTypeIdx};
    use crate::storages::entities::EntityStorage;
    use crate::storages::globals::GlobalStorage;
    use crate::storages::systems::{
        Access, ComponentTypeAccess, SystemCaller, SystemProperties, SystemStorage,
    };
    use crate::storages::updates::UpdateStorage;
    use crate::systems::internal::SystemWrapper;
    use crate::{SystemData, SystemInfo};
    use std::any::{Any, TypeId};
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

    fn system_wrapper<C1, C2>(data: SystemData<'_>, _info: SystemInfo<'_>)
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
    fn run_systems_sequentially() {
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
        let caller_type = SystemCaller::Entity(0.into());
        storage.add(wrapper, caller_type, properties, 0.into());
        use_data(true, |d| {
            storage.run(d);
            let mut updates = d.updates.try_lock().unwrap();
            assert_eq!(updates.deleted_entity_drain().count(), 0);
        });
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
        let caller_type1 = SystemCaller::Entity(0.into());
        storage.add(wrapper1, caller_type1, properties1, 0.into());
        let wrapper2: SystemWrapper = system_wrapper::<Component2, Component1>;
        let properties2 = create_properties(vec![]);
        let caller_type2 = SystemCaller::Global(0.into());
        storage.add(wrapper2, caller_type2, properties2, 0.into());
        use_data(true, |d| {
            storage.run(d);
            assert_eq!(d.components.read_components::<Component1>().len(), 0);
            assert_eq!(d.components.read_components::<Component2>().len(), 0);
        });
        use_data(false, |d| {
            storage.run(d);
            let component1_guard = d.components.read_components::<Component1>().clone();
            let component2_guard = d.components.read_components::<Component2>().clone();
            assert_ne!(
                component1_guard[ArchetypeIdx(0)][ArchetypeEntityPos(1)].0,
                component2_guard[ArchetypeIdx(0)][ArchetypeEntityPos(1)].0
            );
        });
    }

    fn use_data<F>(empty_storage: bool, mut f: F)
    where
        F: FnMut(SystemData<'_>),
    {
        let mut components = ComponentStorage::default();
        let mut globals = GlobalStorage::default();
        globals.idx_or_register(TypeId::of::<u32>());
        let type1_idx = components.type_idx_or_create::<Component1>();
        let type2_idx = components.type_idx_or_create::<Component2>();
        if !empty_storage {
            let location = EntityLocation::new(0.into(), 0.into());
            let thread_id = thread::current().id();
            components.add(type1_idx, location, Component1(thread_id), false);
            components.add(type2_idx, location, Component2(thread_id), false);
            globals.replace_or_add(10_u32);
        }
        let mut actions = ActionStorage::default();
        let action_idx = actions.idx_or_create(None, ActionDependencies::Types(vec![]));
        actions.add_system(action_idx);
        actions.add_system(action_idx);
        let data = SystemData {
            entities: &EntityStorage::default(),
            components: &components,
            globals: &globals,
            archetypes: &ArchetypeStorage::default(),
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
            globals: vec![],
            can_update: false,
            archetype_filter: ArchetypeFilter::None,
        }
    }
}
