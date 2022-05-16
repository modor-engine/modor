use crate::queries::internal::{QueryGuard, QueryGuardBorrow};
use crate::storages::archetypes::EntityLocation;
use crate::storages::components::ComponentTypeIdx;
use crate::storages::core::CoreStorage;
use crate::storages::entities::EntityIdx;
use crate::storages::systems::SystemProperties;
use crate::system_params::internal::{QuerySystemParamWithLifetime, SystemParamWithLifetime};
use crate::system_params::queries::internal::QueryStream;
use crate::{QuerySystemParam, SystemData, SystemInfo, SystemParam};
use std::any::{Any, TypeId};
use std::marker::PhantomData;

/// A system parameter for iterating on entities.
///
/// # Examples
///
/// ```rust
/// # use modor::{Entity, Query};
/// #
/// #[derive(Debug)]
/// struct Position(f32, f32);
///
/// fn print_position(query: Query<'_, (Entity<'_>, &Position)>) {
///     for (entity, position) in query.iter() {
///         println!("Entity with ID {} has position {:?}", entity.id(), position)
///     }
/// }
/// ```
pub struct Query<'a, P, F = ()>
where
    P: 'static + QuerySystemParam,
    F: QueryFilter,
{
    guard: <P as SystemParamWithLifetime<'a>>::GuardBorrow,
    filtered_component_type_idxs: &'a [ComponentTypeIdx],
    data: SystemData<'a>,
    phantom: PhantomData<F>,
}

impl<'a, P, F> Query<'a, P, F>
where
    P: 'static + QuerySystemParam,
    F: QueryFilter,
{
    fn new(
        guard: <P as SystemParamWithLifetime<'a>>::GuardBorrow,
        filtered_component_type_idxs: &'a [ComponentTypeIdx],
        data: SystemData<'a>,
    ) -> Self {
        Self {
            guard,
            filtered_component_type_idxs,
            data,
            phantom: PhantomData,
        }
    }
}

impl<P, F> Query<'_, P, F>
where
    P: 'static + QuerySystemParam,
    F: QueryFilter,
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
        self.data.entities.location(entity_idx).and_then(|l| {
            self.data
                .archetypes
                .has_types(l.idx, self.filtered_component_type_idxs)
                .then(|| l)
        })
    }

    fn first_parent(&self, entity_idx: EntityIdx) -> Option<EntityIdx> {
        let parent_idx = self.data.entities.parent_idx(entity_idx);
        parent_idx.and_then(|p| {
            if self.get(p.into()).is_some() {
                Some(p)
            } else {
                self.first_parent(p)
            }
        })
    }
}

impl<'a, P, F> SystemParamWithLifetime<'a> for Query<'_, P, F>
where
    P: 'static + QuerySystemParam,
    F: QueryFilter,
{
    type Param = Query<'a, P, F>;
    type Guard = QueryGuard<'a, P, F>;
    type GuardBorrow = QueryGuardBorrow<'a>;
    type Stream = QueryStream<'a, P>;
}

impl<P, F> SystemParam for Query<'_, P, F>
where
    P: 'static + QuerySystemParam,
    F: QueryFilter,
{
    type Tuple = (Self,);
    type InnerTuple = P::Tuple;

    fn properties(core: &mut CoreStorage) -> SystemProperties {
        F::register(core);
        let param_properties = P::properties(core);
        SystemProperties {
            component_types: param_properties.component_types,
            can_update: param_properties.can_update,
            filtered_component_type_idxs: vec![],
        }
    }

    fn lock<'a>(
        data: SystemData<'a>,
        info: SystemInfo<'a>,
    ) -> <Self as SystemParamWithLifetime<'a>>::Guard {
        QueryGuard::new(data, info)
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
        stream.item_positions.next().map(|_| {
            Query::new(
                P::borrow_guard(&mut stream.guard),
                stream.filtered_component_type_idxs,
                stream.data,
            )
        })
    }
}

/// A trait implemented for all valid filters that can be applied to a [`Query`](crate::Query).
pub trait QueryFilter: 'static {
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

impl<C> QueryFilter for With<C>
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
        impl<$($params),*> QueryFilter for ($($params,)*)
        where
            $($params: QueryFilter,)*
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

mod internal {
    use crate::storages::components::ComponentTypeIdx;
    use crate::system_params::{SystemParam, SystemParamWithLifetime};
    use crate::{utils, QueryFilter, QuerySystemParam, SystemData, SystemInfo};
    use std::marker::PhantomData;
    use std::ops::Range;

    pub struct QueryGuard<'a, P, F> {
        data: SystemData<'a>,
        item_count: usize,
        filtered_component_type_idxs: Vec<ComponentTypeIdx>,
        phantom: PhantomData<(P, F)>,
    }

    impl<'a, P, F> QueryGuard<'a, P, F>
    where
        P: QuerySystemParam,
        F: QueryFilter,
    {
        pub(crate) fn new(data: SystemData<'a>, info: SystemInfo<'a>) -> Self {
            Self {
                data,
                item_count: info.item_count,
                filtered_component_type_idxs: utils::merge([
                    P::filtered_component_type_idxs(data),
                    F::filtered_component_type_idxs(data),
                ]),
                phantom: PhantomData,
            }
        }

        pub(crate) fn borrow(&mut self) -> QueryGuardBorrow<'_> {
            QueryGuardBorrow {
                data: self.data,
                param_info: SystemInfo {
                    filtered_component_type_idxs: &self.filtered_component_type_idxs,
                    item_count: self.data.item_count(&self.filtered_component_type_idxs),
                },
                item_count: self.item_count,
                filtered_component_type_idxs: &self.filtered_component_type_idxs,
            }
        }
    }

    pub struct QueryGuardBorrow<'a> {
        pub(crate) data: SystemData<'a>,
        pub(crate) param_info: SystemInfo<'a>,
        pub(crate) item_count: usize,
        pub(crate) filtered_component_type_idxs: &'a [ComponentTypeIdx],
    }

    pub struct QueryStream<'a, P>
    where
        P: SystemParam,
    {
        pub(crate) item_positions: Range<usize>,
        pub(crate) data: SystemData<'a>,
        pub(crate) guard: <P as SystemParamWithLifetime<'a>>::Guard,
        pub(crate) filtered_component_type_idxs: &'a [ComponentTypeIdx],
    }

    impl<'a, P> QueryStream<'a, P>
    where
        P: SystemParam,
    {
        pub(crate) fn new(guard: &'a QueryGuardBorrow<'_>) -> Self {
            QueryStream {
                item_positions: 0..guard.item_count,
                data: guard.data,
                guard: P::lock(guard.data, guard.param_info),
                filtered_component_type_idxs: guard.filtered_component_type_idxs,
            }
        }
    }
}
