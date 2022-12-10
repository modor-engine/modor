use crate::storages::archetypes::ArchetypeIdx;
use crate::storages::components::ComponentTypeIdx;
use crate::storages::core::CoreStorage;
use crate::storages::systems::SystemIdx;
use crate::systems::context::Storages;
use crate::utils;
use std::any::{Any, TypeId};
use std::marker::PhantomData;

/// A trait implemented for all valid entity filters.
///
/// These filters can for example be applied to a [`Query`](crate::Query).
pub trait EntityFilter: Any {
    #[doc(hidden)]
    fn is_archetype_kept(
        system_idx: Option<SystemIdx>,
        archetype_idx: ArchetypeIdx,
        storages: Storages<'_>,
    ) -> bool;

    #[doc(hidden)]
    #[allow(unused_variables)]
    fn mutation_component_type_idxs(core: &mut CoreStorage) -> Vec<ComponentTypeIdx> {
        vec![]
    }
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
    fn is_archetype_kept(
        system_idx: Option<SystemIdx>,
        archetype_idx: ArchetypeIdx,
        storages: Storages<'_>,
    ) -> bool {
        !<With<C>>::is_archetype_kept(system_idx, archetype_idx, storages)
    }
}

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
            storages
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

macro_rules! impl_tuple_query_filter {
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
    };
}

impl_tuple_query_filter!();
run_for_tuples_with_idxs!(impl_tuple_query_filter);
