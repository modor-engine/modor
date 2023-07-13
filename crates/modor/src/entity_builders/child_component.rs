use crate::entity_builders::internal::BuiltEntityPart;
use crate::entity_builders::BuiltEntity;
use crate::storages::core::CoreStorage;
use crate::storages::entities::EntityIdx;
use crate::{ComponentSystems, EntityBuilder};

/// A builder for defining child of an entity.
///
/// [`EntityBuilder`](EntityBuilder) needs to be used to instantiate this builder.
pub struct EntityChildComponentBuilder<C> {
    pub(crate) component: C,
}

impl<C> BuiltEntityPart for EntityChildComponentBuilder<C>
where
    C: ComponentSystems,
{
    fn create_other_entities(self, core: &mut CoreStorage, parent_idx: Option<EntityIdx>) {
        EntityBuilder::new()
            .component(self.component)
            .build(core, parent_idx);
    }
}

impl<P, C> EntityBuilder<P, EntityChildComponentBuilder<C>> {
    /// Updates the unique component of the previously created child entity.
    pub fn with(mut self, updater: impl FnOnce(&mut C)) -> Self {
        updater(&mut self.last.component);
        self
    }
}
