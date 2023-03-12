use super::with::With;
use super::EntityFilter;
use crate::storages::archetypes::ArchetypeIdx;
use crate::storages::systems::SystemIdx;
use crate::systems::context::Storages;
use crate::Component;
use std::marker::PhantomData;

/// A filter to keep only entities without a component of type `C`.
///
/// You can group multiple `With` in a tuple to filter entities without multiple specific component
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
/// fn list_not_movable_entities(
///     query: Query<'_, (Entity<'_>, Filter<Or<(Without<Position>, Without<Velocity>)>>)>
/// ) {
///     for (entity, _) in query.iter() {
///         println!("Entity {} has missing position or velocity", entity.id());
///     }
/// }
/// ```
pub struct Without<C>(PhantomData<fn(C)>)
where
    C: Component;

impl<C> EntityFilter for Without<C>
where
    C: Component,
{
    fn is_archetype_kept(
        system_idx: Option<SystemIdx>,
        archetype_idx: ArchetypeIdx,
        storages: Storages<'_>,
    ) -> bool {
        !<With<C>>::is_archetype_kept(system_idx, archetype_idx, storages)
    }
}
