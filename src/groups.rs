use crate::internal::main::MainFacade;
use crate::internal::system::data::SystemDetails;
use crate::{EntityBuilder, EntityMainComponent, SystemBuilder};
use std::num::NonZeroUsize;

pub struct GroupBuilder<'a> {
    main: &'a mut MainFacade,
    group_idx: NonZeroUsize,
}

impl<'a> GroupBuilder<'a> {
    pub fn on_update(&mut self, system: SystemBuilder) -> &mut Self {
        let system = SystemDetails::new(
            system.wrapper,
            system.component_types,
            None,
            system.group_actions,
            system.entity_actions,
        );
        self.main.add_system(Some(self.group_idx), system);
        self
    }

    pub fn with_entity<M>(&mut self, params: M::Params) -> &mut Self
    where
        M: EntityMainComponent,
    {
        let entity_idx = self.main.create_entity(self.group_idx);
        let mut entity_builder = EntityBuilder::new(self.main, entity_idx, self.group_idx);
        M::build(&mut entity_builder, params);
        self
    }

    pub(crate) fn new(main: &'a mut MainFacade, group_idx: NonZeroUsize) -> Self {
        Self { main, group_idx }
    }
}
