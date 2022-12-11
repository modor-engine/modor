use super::EntityFilter;
use crate::storages::archetypes::ArchetypeIdx;
use crate::storages::components::ComponentTypeIdx;
use crate::storages::core::CoreStorage;
use crate::storages::systems::SystemIdx;
use crate::systems::context::Storages;
use std::any::{Any, TypeId};
use std::marker::PhantomData;

/// A filter to keep only entities with a component type `C` changed since last execution of the
/// system.
///
/// At first execution of the system, all entities are kept.<br>
/// After first execution, an entity can match if the component of type `C` has been accessed
/// mutably with a system or a [`Query`](crate::Query). An entity can also match in some cases if
/// some of its components have been added or deleted.
///
/// The filter is applied at archetype level (an archetype regroup entities with the same set of
/// component types), which means that some unchanged entities might be kept by this filter.<br>
/// However it is ensured that all entities with added or changed component of type `C` are kept.
///
/// # Examples
///
/// This filter can be used to track all entities in an optimized way (i.e. iterate only on
/// necessary entities):
///
/// ```rust
/// # use modor::*;
/// #
/// # fn main() {}
///
/// #[derive(Clone, Copy)]
/// struct Position(f32, f32);
///
/// struct PositionStorage {
///     positions: Vec<Option<Position>>,
/// }
///
/// #[entity]
/// impl PositionStorage {
///     #[run]
///     fn delete_entities(&mut self, world: World<'_>) {
///         for entity_id in world
///             .deleted_entity_ids()
///             // don't forget entities that might not match `update_entities` query anymore:
///             .chain(world.transformed_entity_ids())
///         {
///             if let Some(position) = self.positions.get_mut(entity_id) {
///                 *position = None;
///             }
///         }
///     }
///
///     #[run_after_previous]
///     fn update_entities(
///         &mut self,
///         query: Query<'_, (Entity<'_>, &Position, Filter<Changed<Position>>)>
///     ) {
///         for (entity, &position, _) in query.iter() {
///             for _ in self.positions.len()..=entity.id() {
///                 self.positions.push(None);
///             }
///             self.positions[entity.id()] = Some(position);
///         }
///     }
/// }
/// ```
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
