use crate::storages::components::ComponentTypeIdx;
use crate::storages::core::CoreStorage;
use crate::SystemData;
use std::any::{Any, TypeId};
use std::marker::PhantomData;

/// A trait implemented for all valid entity filters.
///
/// These filters can for example be applied to a [`Query`](crate::Query).
pub trait EntityFilter: 'static {
    #[doc(hidden)]
    fn register(core: &mut CoreStorage);

    #[doc(hidden)]
    fn filtered_component_type_idxs(data: SystemData<'_>) -> Vec<ComponentTypeIdx>;
}

/// A filter for restricting a [`Query`](crate::Query) to entities containing an component
/// of type `C`.
///
/// You can group multiple `With` in a tuple to restrict according to multiple component types.<br>
/// A maximum of 10 filters is supported in tuples.
/// If you need more filters for a query, you can use nested tuples.
///
/// # Examples
///
/// ```rust
/// # use modor::{Query, With, Entity};
/// #
/// struct Position;
/// struct Velocity;
///
/// fn list_movable_entities(query: Query<'_, Entity<'_>, (With<Position>, With<Velocity>)>) {
///     for entity in query.iter() {
///         println!("Entity {} is movable", entity.id());
///     }
/// }
/// ```
pub struct With<C>(PhantomData<C>)
where
    C: Any + Sync + Send;

impl<C> EntityFilter for With<C>
where
    C: Any + Sync + Send,
{
    #[doc(hidden)]
    fn register(core: &mut CoreStorage) {
        core.register_component_type::<C>();
    }

    #[doc(hidden)]
    fn filtered_component_type_idxs(data: SystemData<'_>) -> Vec<ComponentTypeIdx> {
        vec![data
            .components
            .type_idx(TypeId::of::<C>())
            .expect("internal error: missing component type for query filter")]
    }
}

macro_rules! impl_tuple_query_filter {
    ($(($params:ident, $indexes:tt)),*) => {
        impl<$($params),*> EntityFilter for ($($params,)*)
        where
            $($params: EntityFilter,)*
        {
            #[allow(unused_variables)]
            fn register(core: &mut CoreStorage) {
                $($params::register(core);)*
            }

            #[allow(unused_mut, unused_variables)]
            fn filtered_component_type_idxs(data: SystemData<'_>) -> Vec<ComponentTypeIdx> {
                let mut types = Vec::new();
                $(types.extend($params::filtered_component_type_idxs(data));)*
                types
            }
        }
    };
}

impl_tuple_query_filter!();
run_for_tuples_with_idxs!(impl_tuple_query_filter);
