use super::EntityFilter;
use crate::storages::archetypes::ArchetypeIdx;
use crate::storages::components::ComponentTypeIdx;
use crate::storages::core::CoreStorage;
use crate::storages::systems::SystemIdx;
use crate::systems::context::Storages;
use crate::{utils, QueryEntityFilter};
use std::marker::PhantomData;

/// A filter to keep only entities matching at least one of the sub-filters.
///
/// Tuple entity filters if you want instead to keep entities matching all sub-filters.<br>
/// A maximum of 10 filters is supported in tuples.
/// If you need more filter conditions, you can use nested tuples.
///
/// # Examples
///
/// ```rust
/// # use modor::*;
/// #
/// #[derive(Component, NoSystem)]
/// struct MainCharacter;
///
/// #[derive(Component, NoSystem)]
/// struct EnemyCharacter;
///
/// fn list_characters(
///     query: Query<'_, (Entity<'_>, Filter<Or<(With<MainCharacter>, With<EnemyCharacter>)>>)>
/// ) {
///     for (entity, _) in query.iter() {
///         println!("Entity {} is a character", entity.id());
///     }
/// }
/// ```
pub struct Or<F>(PhantomData<fn(F)>)
where
    F: EntityFilter;

macro_rules! impl_tuple_filter {
    ($(($params:ident, $indexes:tt)),*) => {
        #[allow(unused_mut, unused_variables)]
        impl<$($params),*> EntityFilter for Or<($($params,)*)>
        where
            $($params: EntityFilter,)*
        {
            fn is_archetype_kept(
                system_idx: Option<SystemIdx>,
                archetype_idx: ArchetypeIdx,
                storages: Storages<'_>,
            ) -> bool {
                false $(|| $params::is_archetype_kept(system_idx, archetype_idx, storages))*
            }

            fn mutation_component_type_idxs(core: &mut CoreStorage) -> Vec<ComponentTypeIdx> {
                utils::merge([$($params::mutation_component_type_idxs(core)),*])
            }
        }

        impl<$($params),*> QueryEntityFilter for Or<($($params,)*)>
        where
            $($params: QueryEntityFilter,)*
        {
        }
    };
}

impl_tuple_filter!();
run_for_tuples_with_idxs!(impl_tuple_filter);
