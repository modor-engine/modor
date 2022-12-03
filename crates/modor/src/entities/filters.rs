use std::any::{Any, TypeId};
use std::marker::PhantomData;

/// A trait implemented for all valid entity filters.
///
/// These filters can for example be applied to a [`Query`](crate::Query).
pub trait EntityFilter: Any {
    #[doc(hidden)]
    fn is_archetype_kept(component_types: &[TypeId]) -> bool;
}

/// An entity filter to keep only entities with a component of type `C`.
///
/// You can group multiple `With` in a tuple to filter entities with multiple specific component
///  types.<br>
/// A maximum of 10 filters is supported in tuples.
/// If you need more filter conditions, you can use nested tuples.
///
/// # Examples
///
/// ```rust
/// # use modor::{Query, With, Entity, Filter};
/// #
/// struct Position;
/// struct Velocity;
///
/// fn list_movable_entities(
///     query: Query<'_, (Entity<'_>, Filter<(With<Position>, With<Velocity>)>)>
/// ) {
///     for (entity, _) in query.iter() {
///         println!("Entity {} is movable", entity.id());
///     }
/// }
/// ```
pub struct With<C>(PhantomData<fn(C)>)
where
    C: Any + Sync + Send;

impl<C> EntityFilter for With<C>
where
    C: Any + Sync + Send,
{
    fn is_archetype_kept(component_types: &[TypeId]) -> bool {
        component_types.contains(&TypeId::of::<C>())
    }
}

/// An entity filter to keep only entities without a component of type `C`.
///
/// You can group multiple `With` in a tuple to filter entities without multiple specific component
///  types.<br>
/// A maximum of 10 filters is supported in tuples.
/// If you need more filter conditions, you can use nested tuples.
///
/// # Examples
///
/// ```rust
/// # use modor::{Query, Without, Or, Entity, Filter};
/// #
/// struct Position;
/// struct Velocity;
///
/// fn list_not_movable_entities(
///     query: Query<'_, (Entity<'_>, Filter<Or<(Without<Position>, Without<Velocity>)>>)>
/// ) {
///     for (entity, _) in query.iter() {
///         println!("Entity {} is not movable", entity.id());
///     }
/// }
/// ```
pub struct Without<C>(PhantomData<fn(C)>)
where
    C: Any + Sync + Send;

impl<C> EntityFilter for Without<C>
where
    C: Any + Sync + Send,
{
    fn is_archetype_kept(component_types: &[TypeId]) -> bool {
        !component_types.contains(&TypeId::of::<C>())
    }
}

// TODO: add doc + tests + Or compatibility + static check test
/// TODO
pub struct Mutated<C>(PhantomData<fn(C)>)
where
    C: Any + Sync + Send;

impl<C> EntityFilter for Mutated<C>
where
    C: Any + Sync + Send,
{
    fn is_archetype_kept(_component_types: &[TypeId]) -> bool {
        true
    }
}

/// An entity filter to keep only entities matching at least one of the sub-filters.
///
/// Tuple entity filters if you want instead to keep entities matching all sub-filters.<br>
/// A maximum of 10 filters is supported in tuples.
/// If you need more filter conditions, you can use nested tuples.
///
/// # Examples
///
/// ```rust
/// # use modor::{Query, With, Entity, Filter, Or};
/// #
/// struct MainCharacter;
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

// TODO: should it be kept ?
impl<C> EntityFilter for Or<With<C>>
where
    C: Any + Sync + Send,
{
    fn is_archetype_kept(component_types: &[TypeId]) -> bool {
        With::<C>::is_archetype_kept(component_types)
    }
}

// TODO: should it be kept ?
impl<C> EntityFilter for Or<Without<C>>
where
    C: Any + Sync + Send,
{
    fn is_archetype_kept(component_types: &[TypeId]) -> bool {
        Without::<C>::is_archetype_kept(component_types)
    }
}

macro_rules! impl_tuple_query_filter {
    ($(($params:ident, $indexes:tt)),*) => {
        #[allow(unused_mut, unused_variables)]
        impl<$($params),*> EntityFilter for ($($params,)*)
        where
            $($params: EntityFilter,)*
        {
            fn is_archetype_kept(component_types: &[TypeId]) -> bool {
                true $(&& $params::is_archetype_kept(component_types))*
            }
        }

        #[allow(unused_mut, unused_variables)]
        impl<$($params),*> EntityFilter for Or<($($params,)*)>
        where
            $($params: EntityFilter,)*
        {
            fn is_archetype_kept(component_types: &[TypeId]) -> bool {
                false $(|| $params::is_archetype_kept(component_types))*
            }
        }
    };
}

impl_tuple_query_filter!();
run_for_tuples_with_idxs!(impl_tuple_query_filter);
