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

    #[allow(unused_variables)]
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
