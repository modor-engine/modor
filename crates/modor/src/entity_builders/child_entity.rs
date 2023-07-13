use crate::entity_builders::internal::BuiltEntityPart;
use crate::storages::core::CoreStorage;
use crate::storages::entities::EntityIdx;
use crate::{BuildableEntity, BuiltEntitySource};

/// A builder for defining child of an entity.
///
/// [`EntityBuilder`](crate::EntityBuilder) needs to be used to instantiate this builder.
pub struct EntityChildEntityBuilder<E> {
    pub(crate) child: E,
}

impl<E> BuiltEntityPart for EntityChildEntityBuilder<E>
where
    E: BuildableEntity<BuiltEntitySource>,
{
    fn create_other_entities(self, core: &mut CoreStorage, parent_idx: Option<EntityIdx>) {
        self.child.build_entity(core, parent_idx);
    }
}
