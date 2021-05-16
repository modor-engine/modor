use crate::internal::main::MainFacade;
use crate::internal::system::data::SystemDetails;
use crate::{GroupBuilder, SystemBuilder};

#[derive(Default)]
pub struct Application(MainFacade);

impl Application {
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
}
