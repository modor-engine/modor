use crate::storages::actions::ActionIdx;
use crate::storages::components::ComponentTypeIdx;
use crate::storages::system_states::{LockedSystem, SystemStateStorage};
use crate::systems::context::{Storages, SystemContext};
use crate::{platform, ArchetypeFilterFn, SystemWrapper};
use scoped_threadpool::Pool;
use std::sync::Mutex;
use typed_index_collections::TiVec;

#[derive(Default)]
pub(crate) struct SystemStorage {
    properties: TiVec<SystemIdx, FullSystemProperties>,
    states: Mutex<SystemStateStorage>,
    pool: Option<Pool>,
}

impl SystemStorage {
    pub(crate) fn thread_count(&self) -> u32 {
        self.pool.as_ref().map_or(0, Pool::thread_count) + 1
    }

    #[allow(unused_variables)]
    pub(super) fn set_thread_count(&mut self, count: u32) {
        if count < 2 {
            self.pool = None;
        } else {
            self.pool = platform::create_pool(count - 1);
        }
    }

    pub(super) fn add(
        &mut self,
        properties: FullSystemProperties,
        action_idx: ActionIdx,
    ) -> SystemIdx {
        self.states
            .get_mut()
            .expect("internal error: cannot access to system states to register component type")
            .add_system(&properties.component_types, action_idx);
        self.properties.push_and_get_key(properties)
    }

    pub(super) fn run(&mut self, storages: Storages<'_>) {
        self.states
            .get_mut()
            .expect("internal error: cannot reset states")
            .reset(self.properties.keys(), storages.actions.system_counts());
        if let Some(mut pool) = self.pool.take() {
            self.run_in_parallel(&mut pool, storages);
            self.pool = Some(pool);
        } else {
            self.run_sequentially(storages);
        }
    }

    fn run_sequentially(&mut self, storages: Storages<'_>) {
        Self::run_thread(storages, &self.states, &self.properties);
    }

    fn run_in_parallel(&mut self, pool: &mut Pool, storages: Storages<'_>) {
        let thread_count = pool.thread_count();
        pool.scoped(|s| {
            for _ in 0..thread_count {
                s.execute(|| {
                    Self::run_thread(storages, &self.states, &self.properties);
                });
            }
            Self::run_thread(storages, &self.states, &self.properties);
        });
    }

    fn run_thread(
        storages: Storages<'_>,
        states: &Mutex<SystemStateStorage>,
        properties: &TiVec<SystemIdx, FullSystemProperties>,
    ) {
        let mut previous_system_idx = None;
        while let LockedSystem::Remaining(system_idx) =
            Self::lock_next_system(storages, states, previous_system_idx, properties)
        {
            if let Some(system_idx) = system_idx {
                Self::run_system(system_idx, properties, storages);
            }
            previous_system_idx = system_idx;
        }
    }

    fn lock_next_system(
        storages: Storages<'_>,
        states: &Mutex<SystemStateStorage>,
        previous_system_idx: Option<SystemIdx>,
        properties: &TiVec<SystemIdx, FullSystemProperties>,
    ) -> LockedSystem {
        states
            .lock()
            .expect("internal error: cannot lock states")
            .lock_next_system(previous_system_idx, storages.actions, properties)
    }

    fn run_system(
        system_idx: SystemIdx,
        properties: &TiVec<SystemIdx, FullSystemProperties>,
        storages: Storages<'_>,
    ) {
        let system = &properties[system_idx];
        let context = SystemContext {
            system_idx: Some(system_idx),
            archetype_filter_fn: system.archetype_filter_fn,
            component_type_idx: system.component_type_idx,
            item_count: storages.item_count(
                Some(system_idx),
                system.archetype_filter_fn,
                system.component_type_idx,
            ),
            storages,
        };
        (system.wrapper)(context);
        storages
            .archetype_states
            .write()
            .expect("internal error: cannot lock archetype state")
            .reset_system(system_idx);
        trace!("system `{}` run", system.label);
    }
}

idx_type!(pub SystemIdx);

pub struct SystemProperties {
    pub(crate) component_types: Vec<ComponentTypeAccess>,
    pub(crate) can_update: bool,
    pub(crate) mutation_component_type_idxs: Vec<ComponentTypeIdx>,
}

pub(crate) struct FullSystemProperties {
    pub(crate) wrapper: SystemWrapper,
    pub(crate) component_types: Vec<ComponentTypeAccess>,
    pub(crate) can_update: bool,
    pub(crate) mutation_component_type_idxs: Vec<ComponentTypeIdx>,
    pub(crate) archetype_filter_fn: ArchetypeFilterFn,
    pub(crate) component_type_idx: Option<ComponentTypeIdx>,
    pub(crate) label: &'static str,
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
