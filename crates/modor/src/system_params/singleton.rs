use crate::storages::core::CoreStorage;
use crate::storages::entities::EntityIdx;
use crate::storages::systems::SystemProperties;
use crate::system_params::query::internal::{QueryGuard, QueryGuardBorrow};
use crate::system_params::singleton::internal::SingleStream;
use crate::systems::context::SystemContext;
use crate::{
    Component, Entity, Filter, Query, QuerySystemParam, QuerySystemParamWithLifetime, SystemParam,
    SystemParamWithLifetime, True, With,
};

/// A [`Single`] to retrieve the singleton component of type `S` as immutable reference.
pub type SingleRef<'a, 'b, S> = Single<'a, S, &'b S>;

/// A [`Single`] to retrieve the singleton component of type `S` as mutable reference.
pub type SingleMut<'a, 'b, S> = Single<'a, S, &'b mut S>;

/// A system parameter for accessing `P` parameters of an entity with a singleton of type `S`.
///
/// This system parameter can be seen as a shortcut for `Query<(P, Filter<With<S>>)>`.<br>
/// If the singleton does not exist or does not match `P`, the system is not executed.<br>
/// If you want to execute the system even if the singleton does not exist or match, you can use
/// instead a system parameter of type `Option<Single<'_, S, P>>`.
///
/// # Examples
///
/// ```rust
/// # use modor::*;
/// #
/// #[derive(SingletonComponent, NoSystem)]
/// struct Score(u32);
///
/// fn increment_score(mut score: Single<'_, Score, &mut Score>) {
///     score.get_mut().0 += 1;
/// }
/// ```
pub struct Single<'a, S, P>
where
    S: Component<IsSingleton = True>,
    P: 'static + QuerySystemParam,
{
    pub(crate) query: Query<'a, (P, Filter<With<S>>)>,
    pub(crate) entity_idx: EntityIdx,
}

impl<'a, S, P> Single<'a, S, P>
where
    S: Component<IsSingleton = True>,
    P: 'static + QuerySystemParam,
{
    /// Returns information about the matching entity.
    pub fn entity(&self) -> Entity<'_> {
        Entity {
            entity_idx: self.entity_idx,
            context: self.query.context,
        }
    }

    /// Returns the constant result of the matching entity.
    pub fn get(&self) -> <P as QuerySystemParamWithLifetime<'_>>::ConstParam {
        self.query
            .get(self.entity_idx.into())
            .expect("internal error: singleton not accessible immutably")
            .0
    }

    /// Returns the result of the matching entity.
    pub fn get_mut(&mut self) -> <P as SystemParamWithLifetime<'_>>::Param {
        self.query
            .get_mut(self.entity_idx.into())
            .expect("internal error: singleton not accessible mutably")
            .0
    }
}

impl<'a, S, P> SystemParamWithLifetime<'a> for Single<'_, S, P>
where
    S: Component<IsSingleton = True>,
    P: 'static + QuerySystemParam,
{
    type Param = Single<'a, S, P>;
    type Guard = QueryGuard<'a, (P, Filter<With<S>>)>;
    type GuardBorrow = QueryGuardBorrow<'a>;
    type Stream = SingleStream<'a, S, P>;
}

impl<S, P> SystemParam for Single<'_, S, P>
where
    S: Component<IsSingleton = True>,
    P: 'static + QuerySystemParam,
{
    type Filter = ();
    type InnerTuple = (P, Filter<With<S>>);

    fn properties(core: &mut CoreStorage) -> SystemProperties {
        core.register_component_type::<S>();
        Query::<(P, Filter<With<S>>)>::properties(core)
    }

    fn lock(context: SystemContext<'_>) -> <Self as SystemParamWithLifetime<'_>>::Guard {
        Query::lock(context)
    }

    fn borrow_guard<'a, 'b>(
        guard: &'a mut <Self as SystemParamWithLifetime<'b>>::Guard,
    ) -> <Self as SystemParamWithLifetime<'a>>::GuardBorrow
    where
        'b: 'a,
    {
        Query::borrow_guard(guard)
    }

    fn stream<'a, 'b>(
        guard: &'a mut <Self as SystemParamWithLifetime<'b>>::GuardBorrow,
    ) -> <Self as SystemParamWithLifetime<'a>>::Stream
    where
        'b: 'a,
    {
        let context = guard.context;
        let type_idx = context.component_type_idx::<S>();
        SingleStream {
            inner: Query::stream(guard),
            entity_idx: context
                .storages
                .components
                .singleton_location(type_idx)
                .map(|l| context.storages.archetypes.entity_idxs(l.idx)[l.pos]),
        }
    }

    #[inline]
    fn stream_next<'a, 'b>(
        stream: &'a mut <Self as SystemParamWithLifetime<'b>>::Stream,
    ) -> Option<<Self as SystemParamWithLifetime<'a>>::Param>
    where
        'b: 'a,
    {
        Query::stream_next(&mut stream.inner)
            .filter(|query| query.iter().len() != 0)
            .map(|query| Single {
                query,
                entity_idx: stream
                    .entity_idx
                    .expect("internal error: cannot retrieve singleton ID"),
            })
    }
}

pub(super) mod internal {
    use crate::storages::entities::EntityIdx;
    use crate::system_params::query::internal::QueryStream;
    use crate::{Component, Filter, SystemParam, True, With};

    pub struct SingleStream<'a, S, P>
    where
        S: Component<IsSingleton = True>,
        P: SystemParam,
    {
        pub(crate) inner: QueryStream<'a, (P, Filter<With<S>>)>,
        pub(crate) entity_idx: Option<EntityIdx>,
    }
}
