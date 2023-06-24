use super::EntityFilter;
use crate::storages::archetypes::ArchetypeIdx;
use crate::storages::systems::SystemIdx;
use crate::systems::context::Storages;
use std::marker::PhantomData;

/// A filter to keep only entities that do not match another filter `F`.
///
/// # Examples
///
/// ```rust
/// # use modor::*;
/// #
/// #[derive(Component, NoSystem)]
/// struct Position;
///
/// fn list_not_movable_entities(
///     query: Query<'_, (Entity<'_>, Filter<Not<With<Position>>>)>
/// ) {
///     for (entity, _) in query.iter() {
///         println!("Entity {} has missing position", entity.id());
///     }
/// }
/// ```
pub struct Not<F>(PhantomData<fn(F)>)
where
    F: EntityFilter;

impl<F> EntityFilter for Not<F>
where
    F: EntityFilter,
{
    fn is_archetype_kept(
        system_idx: Option<SystemIdx>,
        archetype_idx: ArchetypeIdx,
        storages: Storages<'_>,
    ) -> bool {
        !F::is_archetype_kept(system_idx, archetype_idx, storages)
    }
}
