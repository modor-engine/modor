use crate::storages::archetypes::ArchetypeIdx;
use crate::storages::components::ComponentTypeIdx;
use crate::storages::core::CoreStorage;
use crate::storages::systems::SystemIdx;
use crate::systems::building::ArchetypeFilterFn;
use crate::systems::context::Storages;
use std::any::Any;

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
    fn mutation_component_type_idxs(core: &mut CoreStorage) -> Vec<ComponentTypeIdx>;
}

/// A trait implemented for all valid entity filters that can be dynamically applied to
/// [`Query`](crate::Query).
pub trait QueryEntityFilter: EntityFilter {}

/// A filter that can be used dynamically with [`Query`](crate::Query).
///
/// # Example
///
/// ```rust
/// # use modor::*;
/// #
/// #[derive(Component, NoSystem)]
/// struct MyComponent;
///
/// fn my_system(mut query: Query<'_, Entity<'_>>) {
///     query.set_iter_filter(QueryFilter::new::<With<MyComponent>>());
///     for entity in query.iter() {
///         println!("MyComponent found in entity with ID {}", entity.id());
///     }
/// }
/// ```
#[derive(Debug, Clone, Copy)]
pub struct QueryFilter {
    pub(crate) archetype_filter_fn: ArchetypeFilterFn,
}

impl QueryFilter {
    /// Creates a new dynamic query filter.
    pub fn new<F>() -> Self
    where
        F: QueryEntityFilter,
    {
        Self {
            archetype_filter_fn: F::is_archetype_kept,
        }
    }
}

pub(crate) mod and;
pub(crate) mod changed;
pub(crate) mod not;
pub(crate) mod or;
pub(crate) mod with;
