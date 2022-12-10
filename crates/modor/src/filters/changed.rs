use super::EntityFilter;
use crate::storages::archetypes::ArchetypeIdx;
use crate::storages::components::ComponentTypeIdx;
use crate::storages::core::CoreStorage;
use crate::storages::systems::SystemIdx;
use crate::systems::context::Storages;
use std::any::{Any, TypeId};
use std::marker::PhantomData;

// TODO: create ticket for Not<> (special implementation for Not<Mutated<C>> ?)
// TODO: create ticket for deleted/transformed entities (can be accessed as list of IDs from World)

// TODO:
// - Mutated<C>: at least one entity of archetype is accessed mutably or added since last system exec
// - First system execution: no filtering
// - Store whether entity added per archetype
// - Store whether entity accessed mutably per system/archetype/component in RwLock
// - For EntityFilter: fn is_archetype_kept(system_idx: SystemIdx, archetype_idx: ArchetypeIdx, storages: Storages) -> bool

// TODO: add doc + tests + static check test
/// TODO
pub struct Changed<C>(PhantomData<fn(C)>)
where
    C: Any + Sync + Send;

impl<C> EntityFilter for Changed<C>
where
    C: Any + Sync + Send,
{
    fn is_archetype_kept(
        system_idx: Option<SystemIdx>,
        archetype_idx: ArchetypeIdx,
        storages: Storages<'_>,
    ) -> bool {
        system_idx.map_or(true, |system_idx| {
            let component_type_idx = storages
                .components
                .type_idx(TypeId::of::<C>())
                .expect("internal error: read archetype state from not registered component type");
            storages.archetypes.has_new_entity(archetype_idx)
                || storages
                    .archetype_states
                    .read()
                    .expect("internal error: cannot read archetype state")
                    .is_mutated(system_idx, component_type_idx, archetype_idx)
        })
    }

    fn mutation_component_type_idxs(core: &mut CoreStorage) -> Vec<ComponentTypeIdx> {
        vec![core.register_component_type::<C>()]
    }
}
