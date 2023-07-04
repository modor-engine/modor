use crate::entity_builders::internal::BuiltEntityPart;
use crate::entity_builders::BuiltEntity;
use crate::storages::archetypes::{ArchetypeIdx, EntityLocation};
use crate::storages::core::CoreStorage;
use crate::storages::entities::EntityIdx;
use crate::{Component, ComponentSystems, EntityBuilder};

/// A builder for defining child of an entity.
///
/// [`EntityBuilder`](EntityBuilder) needs to be used to instantiate this builder.
pub struct EntityChildComponentBuilder<C, P> {
    pub(crate) component: C,
    pub(crate) previous: P,
}

impl<C, P> EntityChildComponentBuilder<C, P> {
    /// Updates the unique component of the previously created child entity.
    pub fn with(mut self, updater: impl FnOnce(&mut C)) -> Self {
        updater(&mut self.component);
        self
    }
}

impl<C, P> BuiltEntityPart for EntityChildComponentBuilder<C, P>
where
    C: ComponentSystems,
    P: BuiltEntity,
{
    fn create_archetype(
        &mut self,
        core: &mut CoreStorage,
        archetype_idx: ArchetypeIdx,
    ) -> ArchetypeIdx {
        self.previous.create_archetype(core, archetype_idx)
    }

    fn add_components(&mut self, core: &mut CoreStorage, location: EntityLocation) {
        self.previous.add_components(core, location);
    }

    fn create_other_entities(self, core: &mut CoreStorage, parent_idx: Option<EntityIdx>) {
        self.previous.create_other_entities(core, parent_idx);
        EntityBuilder::new()
            .component(self.component)
            .build(core, parent_idx);
    }

    fn update_component<C2>(&mut self, updater: impl FnMut(&mut C2))
    where
        C2: Component,
    {
        self.previous.update_component(updater);
    }
}
