use crate::internal::main::MainFacade;
use crate::internal::system::data::SystemDetails;
use crate::{GroupBuilder, SystemBuilder, SystemData, SystemInfo, SystemOnceBuilder};

#[derive(Default)]
pub struct Application(MainFacade);

impl Application {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_group(mut self, build_group_fn: impl FnOnce(&mut GroupBuilder<'_>)) -> Self {
        let group_idx = self.0.create_group();
        let mut group_builder = GroupBuilder::new(&mut self.0, group_idx);
        build_group_fn(&mut group_builder);
        self
    }

    pub fn with_thread_count(mut self, count: u32) -> Self {
        self.0.set_thread_count(count);
        self
    }

    pub fn on_update(mut self, system: SystemBuilder) -> Self {
        let system =
            SystemDetails::new(system.wrapper, system.component_types, None, system.actions);
        self.0.add_system(None, system);
        self
    }

    pub fn update(&mut self) {
        self.0.run_systems();
        self.0.apply_system_actions();
    }

    pub fn run<S>(&mut self, system: SystemOnceBuilder<S>)
    where
        S: FnMut(&SystemData<'_>, SystemInfo),
    {
        self.0.run_system_once(system);
        self.0.apply_system_actions();
    }
}

#[cfg(test)]
mod application_tests {
    use super::*;

    assert_impl_all!(Application: Send);
    assert_not_impl_any!(Application: Clone);
}
