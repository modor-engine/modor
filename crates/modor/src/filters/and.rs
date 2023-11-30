use super::EntityFilter;
use crate::storages::archetypes::ArchetypeIdx;
use crate::storages::components::ComponentTypeIdx;
use crate::storages::core::CoreStorage;
use crate::storages::systems::SystemIdx;
use crate::systems::context::Storages;
use crate::{utils, QueryEntityFilter};

macro_rules! impl_tuple_filter {
    ($(($params:ident, $indexes:tt)),*) => {
        #[allow(unused_mut, unused_variables)]
        impl<$($params),*> EntityFilter for ($($params,)*)
        where
            $($params: EntityFilter,)*
        {
            fn is_archetype_kept(
                system_idx: Option<SystemIdx>,
                archetype_idx: ArchetypeIdx,
                storages: Storages<'_>,
            ) -> bool {
                true $(&& $params::is_archetype_kept(system_idx, archetype_idx, storages))*
            }

            fn mutation_component_type_idxs(core: &mut CoreStorage) -> Vec<ComponentTypeIdx> {
                utils::merge([$($params::mutation_component_type_idxs(core)),*])
            }
        }

        impl<$($params),*> QueryEntityFilter for ($($params,)*)
        where
            $($params: QueryEntityFilter,)*
        {
        }
    };
}

impl_tuple_filter!();
run_for_tuples_with_idxs!(impl_tuple_filter);
