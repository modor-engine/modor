use crate::storages::archetypes::ArchetypeIdx;
use crate::storages::components::ComponentTypeIdx;
use crate::storages::core::CoreStorage;
use crate::storages::systems::SystemIdx;
use crate::systems::context::Storages;
use std::any::Any;

/// A trait implemented for all valid entity filters.
///
/// These filters can for example be applied to a [`Query`](crate::Query).
pub trait EntityFilter: Any {
    #[doc(hidden)]
    fn is_archetype_kept(
        system_idx: Option<SystemIdx>,
        archetype_idx: ArchetypeIdx,
        storages: Storages<'_>,
    ) -> bool;

    #[doc(hidden)]
    #[allow(unused_variables)]
    fn mutation_component_type_idxs(core: &mut CoreStorage) -> Vec<ComponentTypeIdx> {
        vec![]
    }
}

pub(crate) mod and;
pub(crate) mod changed;
pub(crate) mod not;
pub(crate) mod or;
pub(crate) mod with;
