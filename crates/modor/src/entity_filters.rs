use std::any::{Any, TypeId};
use std::marker::PhantomData;

/// A trait implemented for all valid entity filters.
///
/// These filters can for example be applied to a [`Query`](crate::Query).
pub trait EntityFilter: Any {
    #[doc(hidden)]
    fn is_archetype_kept(archetype_type_ids: &[TypeId]) -> bool;
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
/// # use modor::{Query, With, Entity, Filter};
/// #
/// struct Position;
/// struct Velocity;
///
/// fn list_movable_entities(query: Query<'_, (Entity<'_>, Filter<(With<Position>, With<Velocity>)>)>) {
///     for (entity, _) in query.iter() {
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
    fn is_archetype_kept(archetype_type_ids: &[TypeId]) -> bool {
        archetype_type_ids.contains(&TypeId::of::<C>())
    }
}

macro_rules! impl_tuple_query_filter {
    ($(($params:ident, $indexes:tt)),*) => {
        #[allow(unused_mut, unused_variables)]
        impl<$($params),*> EntityFilter for ($($params,)*)
        where
            $($params: EntityFilter,)*
        {
            fn is_archetype_kept(archetype_type_ids: &[TypeId]) -> bool {
                true $(&& $params::is_archetype_kept(archetype_type_ids))*
            }
        }
    };
}

impl_tuple_query_filter!();
run_for_tuples_with_idxs!(impl_tuple_query_filter);
