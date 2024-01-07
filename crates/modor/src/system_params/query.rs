use crate::query::internal::{QueryGuard, QueryGuardBorrow};
use crate::storages::archetypes::EntityLocation;
use crate::storages::core::CoreStorage;
use crate::storages::entities::EntityIdx;
use crate::storages::systems::SystemProperties;
use crate::system_params::query::internal::{QueryFilterProperties, QueryStream};
use crate::systems::context::SystemContext;
use crate::{
    EntityFilter, QueryFilter, QuerySystemParam, QuerySystemParamWithLifetime, SystemParam,
    SystemParamWithLifetime,
};

/// A system parameter for iterating on entities.
///
/// # Examples
///
/// ```rust
/// # use modor::*;
/// #
///
/// fn print_position(query: Query<'_, Entity<'_>>) {
///     for entity in query.iter() {
///         println!("Entity found with ID {}", entity.id());
///     }
/// }
/// ```
pub struct Query<'a, P>
where
    P: 'static + QuerySystemParam,
{
    pub(super) context: SystemContext<'a>,
    guard: <P as SystemParamWithLifetime<'a>>::GuardBorrow,
    filter: Option<QueryFilterProperties>,
}

impl<'a, P> Query<'a, P>
where
    P: 'static + QuerySystemParam,
{
    fn new(
        guard: <P as SystemParamWithLifetime<'a>>::GuardBorrow,
        context: SystemContext<'a>,
    ) -> Self {
        Self {
            context,
            guard,
            filter: None,
        }
    }
}

impl<P> Query<'_, P>
where
    P: 'static + QuerySystemParam,
{
    /// Adds a dynamic filter to the query iterators.
    ///
    /// This filter has an effect only on [`Query::iter`] and [`Query::iter_mut`] methods.
    ///
    /// Note that if this method is called twice, the second call replaces the previous filter.
    pub fn set_iter_filter(&mut self, filter: QueryFilter) {
        self.filter = Some(QueryFilterProperties {
            filter,
            item_count: self.context.storages.item_count(
                self.context.system_idx,
                <P::Filter>::is_archetype_kept,
                None,
                Some(filter),
            ),
        });
    }

    #[allow(clippy::iter_without_into_iter)] // TODO: create issue
    /// Returns an iterator on constant query results.
    pub fn iter(&self) -> <P as QuerySystemParamWithLifetime<'_>>::Iter {
        P::query_iter(&self.guard, self.filter)
    }

    #[allow(clippy::iter_without_into_iter)] // TODO: create issue
    /// Returns an iterator on query results.
    pub fn iter_mut(&mut self) -> <P as QuerySystemParamWithLifetime<'_>>::IterMut {
        P::query_iter_mut(&mut self.guard, self.filter)
    }

    /// Returns the constant query result for the entity with ID `entity_id`.
    ///
    /// `None` is returned if `entity_id` does not exist or does not match the query.
    pub fn get(
        &self,
        entity_id: usize,
    ) -> Option<<P as QuerySystemParamWithLifetime<'_>>::ConstParam> {
        self.location(entity_id.into())
            .and_then(|l| P::get(&self.guard, l))
    }

    /// Returns the query result for the entity with ID `entity_id`.
    ///
    /// `None` is returned if `entity_id` does not exist or does not match the query.
    pub fn get_mut(
        &mut self,
        entity_id: usize,
    ) -> Option<<P as SystemParamWithLifetime<'_>>::Param> {
        self.location(entity_id.into())
            .and_then(|l| P::get_mut(&mut self.guard, l))
    }

    /// Returns the query results for entities with IDs `entity1_id` and `entity2_id`.
    ///
    /// `None` is returned for each entity that does not exist or does not match the query.
    ///
    /// If `entity1_id` and `entity2_id` are equal, the result is returned only in the first part
    /// of the returned tuple, and the second part contains `None`.
    pub fn get_both_mut(
        &mut self,
        entity1_id: usize,
        entity2_id: usize,
    ) -> (
        Option<<P as SystemParamWithLifetime<'_>>::Param>,
        Option<<P as SystemParamWithLifetime<'_>>::Param>,
    ) {
        if entity1_id == entity2_id {
            (self.get_mut(entity1_id), None)
        } else {
            let location1 = self.location(entity1_id.into());
            let location2 = self.location(entity2_id.into());
            match (location1, location2) {
                (Some(l1), Some(l2)) => P::get_both_mut(&mut self.guard, l1, l2),
                (Some(l1), None) => (P::get_mut(&mut self.guard, l1), None),
                (None, Some(l2)) => (None, P::get_mut(&mut self.guard, l2)),
                (None, None) => (None, None),
            }
        }
    }

    fn location(&self, entity_idx: EntityIdx) -> Option<EntityLocation> {
        self.context
            .storages
            .entities
            .location(entity_idx)
            .and_then(|l| {
                <P::Filter>::is_archetype_kept(
                    self.context.system_idx,
                    l.idx,
                    self.context.storages,
                )
                .then_some(l)
            })
    }
}

impl<'a, P> SystemParamWithLifetime<'a> for Query<'_, P>
where
    P: 'static + QuerySystemParam,
{
    type Param = Query<'a, P>;
    type Guard = QueryGuard<'a, P>;
    type GuardBorrow = QueryGuardBorrow<'a>;
    type Stream = QueryStream<'a, P>;
}

impl<P> SystemParam for Query<'_, P>
where
    P: 'static + QuerySystemParam,
{
    type Filter = ();
    type InnerTuple = (P,);

    fn properties(core: &mut CoreStorage) -> SystemProperties {
        let param_properties = P::properties(core);
        SystemProperties {
            component_types: param_properties.component_types,
            can_update: param_properties.can_update,
            mutation_component_type_idxs: param_properties.mutation_component_type_idxs,
        }
    }

    fn lock(context: SystemContext<'_>) -> <Self as SystemParamWithLifetime<'_>>::Guard {
        QueryGuard::new(context)
    }

    fn borrow_guard<'a, 'b>(
        guard: &'a mut <Self as SystemParamWithLifetime<'b>>::Guard,
    ) -> <Self as SystemParamWithLifetime<'a>>::GuardBorrow
    where
        'b: 'a,
    {
        guard.borrow()
    }

    fn stream<'a, 'b>(
        guard: &'a mut <Self as SystemParamWithLifetime<'b>>::GuardBorrow,
    ) -> <Self as SystemParamWithLifetime<'a>>::Stream
    where
        'b: 'a,
    {
        QueryStream::new(guard)
    }

    #[inline]
    fn stream_next<'a, 'b>(
        stream: &'a mut <Self as SystemParamWithLifetime<'b>>::Stream,
    ) -> Option<<Self as SystemParamWithLifetime<'a>>::Param>
    where
        'b: 'a,
    {
        stream
            .item_positions
            .next()
            .map(|_| Query::new(P::borrow_guard(&mut stream.guard), stream.context))
    }
}

pub(crate) mod internal {
    use crate::system_params::{SystemParam, SystemParamWithLifetime};
    use crate::systems::context::SystemContext;
    use crate::{EntityFilter, QueryFilter, QuerySystemParam};
    use std::marker::PhantomData;
    use std::ops::Range;

    pub struct QueryGuard<'a, P> {
        context: SystemContext<'a>,
        item_count: usize,
        phantom: PhantomData<P>,
    }

    impl<'a, P> QueryGuard<'a, P>
    where
        P: QuerySystemParam,
    {
        pub(crate) fn new(context: SystemContext<'a>) -> Self {
            Self {
                context,
                item_count: context.item_count,
                phantom: PhantomData,
            }
        }

        pub(crate) fn borrow(&mut self) -> QueryGuardBorrow<'_> {
            QueryGuardBorrow {
                context: self.context,
                param_context: SystemContext {
                    system_idx: self.context.system_idx,
                    archetype_filter_fn: <P::Filter>::is_archetype_kept,
                    component_type_idx: None,
                    item_count: self.context.storages.item_count(
                        self.context.system_idx,
                        <P::Filter>::is_archetype_kept,
                        None,
                        None,
                    ),
                    storages: self.context.storages,
                },
                item_count: self.item_count,
            }
        }
    }

    pub struct QueryGuardBorrow<'a> {
        pub(crate) context: SystemContext<'a>,
        pub(crate) param_context: SystemContext<'a>,
        pub(crate) item_count: usize,
    }

    pub struct QueryStream<'a, P>
    where
        P: SystemParam,
    {
        pub(crate) item_positions: Range<usize>,
        pub(crate) context: SystemContext<'a>,
        pub(crate) guard: <P as SystemParamWithLifetime<'a>>::Guard,
    }

    impl<'a, P> QueryStream<'a, P>
    where
        P: SystemParam,
    {
        pub(crate) fn new(guard: &'a QueryGuardBorrow<'_>) -> Self {
            QueryStream {
                item_positions: 0..guard.item_count,
                context: guard.context,
                guard: P::lock(guard.param_context),
            }
        }
    }

    #[derive(Debug, Clone, Copy)]
    pub struct QueryFilterProperties {
        pub(crate) filter: QueryFilter,
        pub(crate) item_count: usize,
    }
}
