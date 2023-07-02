use crate::entity_builders::internal::BuiltEntityPart;
use crate::entity_builders::BuiltEntity;
use crate::storages::archetypes::{ArchetypeIdx, EntityLocation};
use crate::storages::components::ComponentTypeIdx;
use crate::storages::core::CoreStorage;
use crate::storages::entities::EntityIdx;
use crate::{ComponentSystems, SystemRunner, True};
use std::any;
use std::any::TypeId;

/// A builder for defining component of an entity.
///
/// [`EntityBuilder`](crate::EntityBuilder) needs to be used to instantiate this builder.
pub struct EntityComponentBuilder<C, P> {
    pub(crate) component: Option<C>,
    pub(crate) type_idx: Option<ComponentTypeIdx>,
    pub(crate) previous: P,
}

impl<C, P> EntityComponentBuilder<C, P>
where
    C: ComponentSystems,
{
    /// Updates the component that has just been added to the entity.
    ///
    /// If the component is optional, then the update is performed only if the component exists.
    pub fn with(mut self, updater: impl FnOnce(&mut C)) -> Self {
        if let Some(component) = &mut self.component {
            updater(component);
        }
        self
    }
}

impl<C, P> BuiltEntityPart for EntityComponentBuilder<C, P>
where
    C: ComponentSystems,
    P: BuiltEntity,
{
    fn create_archetype(
        &mut self,
        core: &mut CoreStorage,
        archetype_idx: ArchetypeIdx,
    ) -> ArchetypeIdx {
        let archetype_idx = self.previous.create_archetype(core, archetype_idx);
        if !core.components().has_systems_loaded::<C>() {
            let component_type_idx = core.set_systems_as_loaded::<C>();
            C::on_update(SystemRunner {
                core,
                component_action_type: TypeId::of::<C::Action>(),
                component_type_idx,
                action_idxs: vec![],
            });
        };
        if self.component.is_some() {
            let (type_idx, archetype_idx) = core.add_component_type::<C>(archetype_idx);
            self.type_idx = Some(type_idx);
            archetype_idx
        } else {
            archetype_idx
        }
    }

    fn add_components(&mut self, core: &mut CoreStorage, location: EntityLocation) {
        self.previous.add_components(core, location);
        if let (Some(component), Some(type_idx)) = (self.component.take(), self.type_idx) {
            core.add_component(
                component,
                type_idx,
                location,
                TypeId::of::<C::IsSingleton>() == TypeId::of::<True>(),
            );
            trace!(
                "component `{}` added to entity with ID {}",
                any::type_name::<C>(),
                core.archetypes().entity_idxs(location.idx)[location.pos].0
            );
        } else {
            trace!(
                "component `{}` not added to entity with ID {} as condition is false",
                any::type_name::<C>(),
                core.archetypes().entity_idxs(location.idx)[location.pos].0
            );
        }
    }

    fn create_other_entities(self, core: &mut CoreStorage, parent_idx: Option<EntityIdx>) {
        self.previous.create_other_entities(core, parent_idx);
    }
}
