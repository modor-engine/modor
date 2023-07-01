use crate::entity_builders::internal::BuiltEntityPart;
use crate::entity_builders::BuiltEntity;
use crate::storages::archetypes::{ArchetypeIdx, EntityLocation};
use crate::storages::core::CoreStorage;
use crate::storages::entities::EntityIdx;

/// A builder for defining inheritance of an entity.
///
/// [`EntityBuilder`](crate::EntityBuilder) needs to be used to instantiate this builder.
pub struct EntityInheritedBuilder<E, P> {
    pub(crate) entity: E,
    pub(crate) previous: P,
}

impl<E, P> BuiltEntityPart for EntityInheritedBuilder<E, P>
where
    E: BuiltEntity,
    P: BuiltEntity,
{
    fn create_archetype(
        &mut self,
        core: &mut CoreStorage,
        archetype_idx: ArchetypeIdx,
    ) -> ArchetypeIdx {
        let archetype_idx = self.previous.create_archetype(core, archetype_idx);
        self.entity.create_archetype(core, archetype_idx)
    }

    fn add_components(&mut self, core: &mut CoreStorage, location: EntityLocation) {
        self.previous.add_components(core, location);
        self.entity.add_components(core, location);
    }

    fn create_other_entities(self, core: &mut CoreStorage, parent_idx: Option<EntityIdx>) {
        self.previous.create_other_entities(core, parent_idx);
        self.entity.create_other_entities(core, parent_idx);
    }
}
