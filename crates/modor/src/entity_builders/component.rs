use crate::entity_builders::internal::BuiltEntityPart;
use crate::storages::archetypes::{ArchetypeIdx, EntityLocation};
use crate::storages::components::ComponentTypeIdx;
use crate::storages::core::CoreStorage;
use crate::{Component, ComponentSystems, EntityBuilder, SystemRunner, True};
use std::any;
use std::any::{Any, TypeId};

/// A builder for defining component of an entity.
///
/// [`EntityBuilder`](EntityBuilder) needs to be used to instantiate this builder.
pub struct EntityComponentBuilder<C> {
    pub(crate) component: Option<C>,
    pub(crate) type_idx: Option<ComponentTypeIdx>,
}

impl<C> BuiltEntityPart for EntityComponentBuilder<C>
where
    C: ComponentSystems,
{
    fn create_archetype(
        &mut self,
        core: &mut CoreStorage,
        archetype_idx: ArchetypeIdx,
    ) -> ArchetypeIdx {
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
        if let (Some(component), Some(type_idx)) = (self.component.take(), self.type_idx) {
            core.add_component(
                component,
                type_idx,
                location,
                TypeId::of::<C::IsSingleton>() == TypeId::of::<True>(),
            );
            trace!(
                "component `{}` added to entity with ID {}", // no-coverage
                any::type_name::<C>(),                       // no-coverage
                core.archetypes().entity_idxs(location.idx)[location.pos].0  // no-coverage
            );
        } else {
            trace!(
                "component `{}` not added to entity with ID {} as condition is false", // no-coverage
                any::type_name::<C>(), // no-coverage
                core.archetypes().entity_idxs(location.idx)[location.pos].0  // no-coverage
            );
        }
    }

    fn update_component<C2>(&mut self, mut updater: impl FnMut(&mut C2))
    where
        C2: Component,
    {
        if let Some(component) = &mut self.component {
            if let Some(component) = (component as &mut dyn Any).downcast_mut() {
                updater(component);
            }
        }
    }
}

impl<P, C> EntityBuilder<P, EntityComponentBuilder<C>> {
    /// Updates the previously added component.
    ///
    /// If the component is optional, then the update is performed only if the component exists.
    pub fn with(mut self, updater: impl FnOnce(&mut C)) -> Self {
        if let Some(component) = &mut self.last.component {
            updater(component);
        }
        self
    }
}
