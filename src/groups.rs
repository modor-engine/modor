use crate::internal::main::MainFacade;
use crate::internal::system::data::SystemInfo;
use crate::{Entity, EntityBuilder, SystemBuilder};
use std::num::NonZeroUsize;

pub struct GroupBuilder<'a> {
    ecs: &'a mut MainFacade,
    group_idx: NonZeroUsize,
}

impl<'a> GroupBuilder<'a> {
    pub fn on_update(&mut self, system: SystemBuilder) -> &mut Self {
        let system = SystemInfo::new(
            system.wrapper,
            system.component_types,
            None,
            system.group_actions,
        );
        self.ecs.add_system(Some(self.group_idx), system);
        self
    }

    pub fn with_entity<C>(&mut self, params: C::Params) -> &mut Self
    where
        C: Entity,
    {
        let entity_idx = self.ecs.create_entity(self.group_idx);
        let mut entity_builder = EntityBuilder::new(self.ecs, entity_idx, self.group_idx);
        C::build(&mut entity_builder, params);
        self
    }

    pub(crate) fn new(ecs: &'a mut MainFacade, group_idx: NonZeroUsize) -> Self {
        Self { ecs, group_idx }
    }
}
