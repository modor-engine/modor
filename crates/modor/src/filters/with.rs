use super::EntityFilter;
use crate::storages::archetypes::ArchetypeIdx;
use crate::storages::components::ComponentTypeIdx;
use crate::storages::core::CoreStorage;
use crate::storages::systems::SystemIdx;
use crate::systems::context::Storages;
use crate::{Component, QueryEntityFilter};
use std::any::TypeId;
use std::marker::PhantomData;

/// A filter to keep only entities with a component of type `C`.
///
/// You can group multiple [`With`] in a tuple to filter entities with multiple specific component
///  types.<br>
/// A maximum of 10 filters is supported in tuples.
/// If you need more filter conditions, you can use nested tuples.
///
/// # Examples
///
/// ```rust
/// # use modor::*;
/// #
/// #[derive(Component, NoSystem)]
/// struct Position;
///
/// #[derive(Component, NoSystem)]
/// struct Velocity;
///
/// fn list_movable_entities(
///     query: Query<'_, (Entity<'_>, Filter<(With<Position>, With<Velocity>)>)>
/// ) {
///     for (entity, _) in query.iter() {
///         println!("Entity {} has position and velocity", entity.id());
///     }
/// }
/// ```
pub struct With<C>(PhantomData<fn(C)>)
where
    C: Component;

impl<C> EntityFilter for With<C>
where
    C: Component,
{
    fn is_archetype_kept(
        _system_idx: Option<SystemIdx>,
        archetype_idx: ArchetypeIdx,
        storages: Storages<'_>,
    ) -> bool {
        storages
            .archetypes
            .type_ids(archetype_idx)
            .contains(&TypeId::of::<C>())
    }

    fn mutation_component_type_idxs(_core: &mut CoreStorage) -> Vec<ComponentTypeIdx> {
        vec![]
    }
}

impl<C> QueryEntityFilter for With<C> where C: Component {}
