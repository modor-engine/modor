use crate::storages::core::CoreStorage;
use crate::storages::systems::SystemProperties;
use crate::system_params::internal::SystemParamWithLifetime;
use crate::systems::context::SystemContext;
use crate::{Component, Query, QuerySystemParam, Single, SystemParam, True};

#[allow(clippy::use_self)]
impl<'a, S, P> SystemParamWithLifetime<'a> for Option<Single<'_, S, P>>
where
    S: Component<IsSingleton = True>,
    P: 'static + QuerySystemParam,
{
    type Param = Option<Single<'a, S, P>>;
    type Guard = <Single<'a, S, P> as SystemParamWithLifetime<'a>>::Guard;
    type GuardBorrow = <Single<'a, S, P> as SystemParamWithLifetime<'a>>::GuardBorrow;
    type Stream = <Single<'a, S, P> as SystemParamWithLifetime<'a>>::Stream;
}

impl<'c, S, P> SystemParam for Option<Single<'c, S, P>>
where
    S: Component<IsSingleton = True>,
    P: 'static + QuerySystemParam,
{
    type Filter = ();
    type InnerTuple = <Single<'c, S, P> as SystemParam>::InnerTuple;

    fn properties(core: &mut CoreStorage) -> SystemProperties {
        <Single<'_, S, P>>::properties(core)
    }

    fn lock(context: SystemContext<'_>) -> <Self as SystemParamWithLifetime<'_>>::Guard {
        Single::lock(context)
    }

    fn borrow_guard<'a, 'b>(
        guard: &'a mut <Self as SystemParamWithLifetime<'b>>::Guard,
    ) -> <Self as SystemParamWithLifetime<'a>>::GuardBorrow
    where
        'b: 'a,
    {
        Single::borrow_guard(guard)
    }

    fn stream<'a, 'b>(
        guard: &'a mut <Self as SystemParamWithLifetime<'b>>::GuardBorrow,
    ) -> <Self as SystemParamWithLifetime<'a>>::Stream
    where
        'b: 'a,
    {
        Single::stream(guard)
    }

    #[inline]
    fn stream_next<'a, 'b>(
        stream: &'a mut <Self as SystemParamWithLifetime<'b>>::Stream,
    ) -> Option<<Self as SystemParamWithLifetime<'a>>::Param>
    where
        'b: 'a,
    {
        Query::stream_next(&mut stream.inner).map(|query| {
            let item_count = query.iter().len();
            (item_count != 0)
                .then_some(stream.entity_idx)
                .flatten()
                .map(|entity_idx| Single { query, entity_idx })
        })
    }
}
