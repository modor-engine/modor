use crate::storages::archetypes::{ArchetypeIdx, ArchetypeStorage, EntityLocation};
use crate::storages::core::CoreStorage;
use crate::storages::entities::EntityIdx;

pub trait BuiltEntityPart: Sized {
    fn create_archetype(
        &mut self,
        core: &mut CoreStorage,
        archetype_idx: ArchetypeIdx,
    ) -> ArchetypeIdx;

    fn add_components(&mut self, core: &mut CoreStorage, location: EntityLocation);

    fn create_other_entities(self, core: &mut CoreStorage, parent_idx: Option<EntityIdx>);

    fn build(mut self, core: &mut CoreStorage, parent_idx: Option<EntityIdx>) -> EntityIdx {
        let archetype_idx = self.create_archetype(core, ArchetypeStorage::DEFAULT_IDX);
        let (entity_idx, location) = core.create_entity(archetype_idx, parent_idx);
        self.add_components(core, location);
        self.create_other_entities(core, Some(entity_idx));
        trace!("entity created with ID {}", entity_idx.0);
        entity_idx
    }
}
