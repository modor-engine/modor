use crate::entity_builders::internal::BuiltEntityPart;
use crate::storages::archetypes::{ArchetypeIdx, EntityLocation};
use crate::storages::core::CoreStorage;
use crate::storages::entities::EntityIdx;
use crate::{BuiltEntity, Component, True};
use std::any;
use std::any::TypeId;
use std::marker::PhantomData;

/// A builder for defining dependency of an entity.
///
/// [`EntityBuilder`](crate::EntityBuilder) needs to be used to instantiate this builder.
pub struct EntityDependencyBuilder<C, E, F, P> {
    pub(crate) builder: F,
    pub(crate) previous: P,
    pub(crate) phantom: PhantomData<fn(C) -> E>,
}

impl<C, E, F, P> BuiltEntityPart for EntityDependencyBuilder<C, E, F, P>
where
    C: Component<IsSingleton = True>,
    E: BuiltEntity,
    F: FnOnce() -> E,
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
        let singleton_exists = core
            .components()
            .type_idx(TypeId::of::<C>())
            .and_then(|c| core.components().singleton_location(c))
            .is_some();
        if singleton_exists {
            trace!(
                "dependency entity for singleton of type `{}` not created as already existing",
                any::type_name::<F>(),
            );
        } else {
            (self.builder)().build(core, None);
            trace!(
                "dependency entity for singleton of type `{}` created",
                any::type_name::<F>(),
            );
        }
    }

    fn update_component<C2>(&mut self, updater: impl FnMut(&mut C2))
    where
        C2: Component,
    {
        self.previous.update_component(updater);
    }
}
