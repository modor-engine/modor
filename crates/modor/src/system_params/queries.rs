use crate::queries::internal::{QueryGuard, QueryGuardBorrow};
use crate::storages::archetypes::EntityLocation;
use crate::storages::core::CoreStorage;
use crate::storages::entities::EntityIdx;
use crate::storages::systems::SystemProperties;
use crate::system_params::internal::{QuerySystemParamWithLifetime, SystemParamWithLifetime};
use crate::system_params::queries::internal::QueryStream;
use crate::systems::context::SystemContext;
use crate::{EntityFilter, QuerySystemParam, SystemParam};

/// A system parameter for iterating on entities.
///
/// # Examples
///
/// ```rust
/// # use modor::{Entity, Query};
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
    guard: <P as SystemParamWithLifetime<'a>>::GuardBorrow,
    context: SystemContext<'a>,
}

impl<'a, P> Query<'a, P>
where
    P: 'static + QuerySystemParam,
{
    fn new(
        guard: <P as SystemParamWithLifetime<'a>>::GuardBorrow,
        context: SystemContext<'a>,
    ) -> Self {
        Self { guard, context }
    }
}

impl<P> Query<'_, P>
where
    P: 'static + QuerySystemParam,
{
    /// Returns an iterator on constant query results.
    pub fn iter(&self) -> <P as QuerySystemParamWithLifetime<'_>>::Iter {
        P::query_iter(&self.guard)
    }

    /// Returns an iterator on query results.
    pub fn iter_mut(&mut self) -> <P as QuerySystemParamWithLifetime<'_>>::IterMut {
        P::query_iter_mut(&mut self.guard)
    }

    /// Returns the constant query result for the entity with ID `entity_id`.
    ///
    /// `None` is returned if `entity_id` does not exist or does not match the query.
    #[inline]
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
    #[inline]
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
    #[inline]
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

    /// Returns the constant query results for entity with ID `entity_id` and its first parent that
    /// matches the query.
    ///
    /// For example, the entity has a direct parent that does not match the query,
    /// but has a grand parent that matches. It means the second part of the returned value
    /// is the query result corresponding to the grand parent.
    ///
    /// `None` is returned for the entity if it does not exist or does not match the query.<br>
    /// `None` is returned for the first matching parent if it is not found.
    #[inline]
    pub fn get_with_first_parent(
        &self,
        entity_id: usize,
    ) -> (
        Option<<P as QuerySystemParamWithLifetime<'_>>::ConstParam>,
        Option<<P as QuerySystemParamWithLifetime<'_>>::ConstParam>,
    ) {
        (
            self.get(entity_id),
            self.first_parent(entity_id.into())
                .and_then(|p| self.get(p.into())),
        )
    }

    /// Returns the query results for entity with ID `entity_id` and its first parent that
    /// matches the query.
    ///
    /// For example, the entity has a direct parent that does not match the query,
    /// but has a grand parent that matches. It means the second part of the returned value
    /// is the query result corresponding to the grand parent.
    ///
    /// `None` is returned for the entity if it does not exist or does not match the query.<br>
    /// `None` is returned for the first matching parent if it is not found.
    #[inline]
    pub fn get_with_first_parent_mut(
        &mut self,
        entity_id: usize,
    ) -> (
        Option<<P as SystemParamWithLifetime<'_>>::Param>,
        Option<<P as SystemParamWithLifetime<'_>>::Param>,
    ) {
        if let Some(first_parent_idx) = self.first_parent(entity_id.into()) {
            self.get_both_mut(entity_id, first_parent_idx.into())
        } else {
            (self.get_mut(entity_id), None)
        }
    }

    fn location(&self, entity_idx: EntityIdx) -> Option<EntityLocation> {
        self.context
            .storages
            .entities
            .location(entity_idx)
            .and_then(|l| {
                <P::Filter>::is_archetype_kept(self.context.storages.archetypes.type_ids(l.idx))
                    .then_some(l)
            })
    }

    fn first_parent(&self, entity_idx: EntityIdx) -> Option<EntityIdx> {
        let parent_idx = self.context.storages.entities.parent_idx(entity_idx);
        parent_idx.and_then(|p| {
            if self.get(p.into()).is_some() {
                Some(p)
            } else {
                self.first_parent(p)
            }
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
    type InnerTuple = P::InnerTuple;

    fn properties(core: &mut CoreStorage) -> SystemProperties {
        let param_properties = P::properties(core);
        SystemProperties {
            component_types: param_properties.component_types,
            can_update: param_properties.can_update,
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

mod internal {
    use crate::system_params::{SystemParam, SystemParamWithLifetime};
    use crate::systems::context::SystemContext;
    use crate::{EntityFilter, QuerySystemParam};
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
                    archetype_filter_fn: <P::Filter>::is_archetype_kept,
                    entity_type_idx: None,
                    item_count: self
                        .context
                        .storages
                        .item_count(<P::Filter>::is_archetype_kept, None),
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
}
