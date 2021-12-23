use crate::storages::core::CoreStorage;
use crate::storages::systems::SystemProperties;
use crate::system_params::internal::{
    QuerySystemParamWithLifetime, SystemParamIterInfo, SystemParamWithLifetime,
};
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
    iter_info: SystemParamIterInfo,
    phantom: PhantomData<F>,
}

impl<'a, P, F> Query<'a, P, F>
where
    P: 'static + QuerySystemParam,
    F: QueryFilter,
{
    fn new(
        data: &'a SystemData<'a>,
        guard: <P as SystemParamWithLifetime<'a>>::GuardBorrow,
    ) -> Self {
        Self {
            guard,
            iter_info: P::iter_info(
                data,
                &SystemInfo {
                    filtered_component_types: F::filtered_component_types(),
                },
            ),
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
        P::query_iter(&self.guard, &self.iter_info)
    }

    /// Returns an iterator on query results.
    pub fn iter_mut(&mut self) -> <P as QuerySystemParamWithLifetime<'_>>::IterMut {
        P::query_iter_mut(&mut self.guard, &self.iter_info)
    }
}

impl<'a, P, F> SystemParamWithLifetime<'a> for Query<'_, P, F>
where
    P: 'static + QuerySystemParam,
    F: QueryFilter,
{
    type Param = Query<'a, P, F>;
    type Guard = &'a SystemData<'a>;
    type GuardBorrow = &'a SystemData<'a>;
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
        P::properties(core)
    }

    fn iter_info(_data: &SystemData<'_>, _info: &SystemInfo) -> SystemParamIterInfo {
        SystemParamIterInfo::None
    }

    fn lock<'a>(data: &'a SystemData<'_>) -> <Self as SystemParamWithLifetime<'a>>::Guard {
        data
    }

    fn borrow_guard<'a, 'b>(
        guard: &'a mut <Self as SystemParamWithLifetime<'b>>::Guard,
    ) -> <Self as SystemParamWithLifetime<'a>>::GuardBorrow
    where
        'b: 'a,
    {
        guard
    }

    fn stream<'a, 'b>(
        guard: &'a mut <Self as SystemParamWithLifetime<'b>>::GuardBorrow,
        info: &'a SystemParamIterInfo,
    ) -> <Self as SystemParamWithLifetime<'a>>::Stream
    where
        'b: 'a,
    {
        QueryStream::new(info, guard)
    }

    #[inline]
    fn stream_next<'a, 'b>(
        stream: &'a mut <Self as SystemParamWithLifetime<'b>>::Stream,
    ) -> Option<<Self as SystemParamWithLifetime<'a>>::Param>
    where
        'b: 'a,
    {
        stream
            .entity_positions
            .next()
            .map(|_| Query::new(stream.data, P::borrow_guard(&mut stream.guard)))
    }
}

/// A trait implemented for all valid filters that can be applied to a [`Query`](crate::Query).
pub trait QueryFilter: 'static {
    #[doc(hidden)]
    fn register(core: &mut CoreStorage);

    #[doc(hidden)]
    fn filtered_component_types() -> Vec<TypeId>;
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
    fn register(core: &mut CoreStorage) {
        core.register_component_type::<C>();
    }

    #[doc(hidden)]
    fn filtered_component_types() -> Vec<TypeId> {
        vec![TypeId::of::<C>()]
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

            #[allow(unused_mut)]
            fn filtered_component_types() -> Vec<TypeId> {
                let mut types = Vec::new();
                $(types.extend($params::filtered_component_types());)*
                types
            }
        }
    };
}

impl_tuple_query_filter!();
run_for_tuples_with_idxs!(impl_tuple_query_filter);

mod internal {
    use crate::system_params::internal::SystemParamIterInfo;
    use crate::system_params::{SystemParam, SystemParamWithLifetime};
    use crate::SystemData;
    use std::ops::Range;

    pub struct QueryStream<'a, P>
    where
        P: SystemParam,
    {
        pub(crate) data: &'a SystemData<'a>,
        pub(crate) entity_positions: Range<usize>,
        pub(crate) guard: <P as SystemParamWithLifetime<'a>>::Guard,
    }

    impl<'a, P> QueryStream<'a, P>
    where
        P: SystemParam,
    {
        pub(crate) fn new(info: &'a SystemParamIterInfo, data: &'a SystemData<'_>) -> Self {
            QueryStream {
                data,
                entity_positions: 0..info.item_count(),
                guard: P::lock(data),
            }
        }
    }
}

#[cfg(test)]
mod query_tests {
    use super::*;
    use crate::storages::archetypes::ArchetypeStorage;
    use crate::storages::core::CoreStorage;
    use std::panic::{RefUnwindSafe, UnwindSafe};

    assert_impl_all!(Query<'_, ()>: Sync, Send, UnwindSafe, RefUnwindSafe, Unpin);

    #[test]
    fn iter() {
        let mut core = CoreStorage::default();
        create_entity(&mut core, 10_u32, 11_i8);
        create_entity(&mut core, 20_u32, 21_i64);
        create_entity(&mut core, 30_u8, 31_i8);
        let data = core.system_data();
        let mut guard = core.components().write_components::<u32>();
        let query = Query::<&mut u32, With<i64>>::new(&data, &mut *guard);

        let iter = query.iter();

        assert_iter!(iter, [&20]);
    }

    #[test]
    fn iter_mut() {
        let mut core = CoreStorage::default();
        create_entity(&mut core, 10_u32, 11_i8);
        create_entity(&mut core, 20_u32, 21_i64);
        create_entity(&mut core, 30_u8, 31_i8);
        let data = core.system_data();
        let mut guard = core.components().write_components::<u32>();
        let mut query = Query::<&mut u32, With<i64>>::new(&data, &mut *guard);

        let iter = query.iter_mut();

        assert_iter!(iter, [&20]);
    }

    fn create_entity<C1, C2>(core: &mut CoreStorage, component1: C1, component2: C2)
    where
        C1: Any + Sync + Send,
        C2: Any + Sync + Send,
    {
        let archetype1_idx = ArchetypeStorage::DEFAULT_IDX;
        let (type1_idx, archetype2_idx) = core.add_component_type::<C1>(archetype1_idx);
        let (type2_idx, archetype3_idx) = core.add_component_type::<C2>(archetype2_idx);
        let location = core.create_entity(archetype3_idx);
        core.add_component(component1, type1_idx, location);
        core.add_component(component2, type2_idx, location);
    }
}

#[cfg(test)]
mod with_tests {
    use super::*;
    use std::panic::{RefUnwindSafe, UnwindSafe};

    assert_impl_all!(With<u32>: Sync, Send, UnwindSafe, RefUnwindSafe, Unpin);

    macro_rules! test_tuple_register {
        ($($params:ident),*) => {{
            let mut core = CoreStorage::default();

            <($(With<$params>,)*) as QueryFilter>::register(&mut core);

            $(assert!(core.components().type_idx(TypeId::of::<$params>()).is_some());)*
        }};
    }

    #[test]
    fn register_types() {
        let mut core = CoreStorage::default();

        With::<u32>::register(&mut core);

        assert!(core.components().type_idx(TypeId::of::<u32>()).is_some());

        test_tuple_register!();
        test_tuple_register!(u8);
        test_tuple_register!(u8, u16);
        test_tuple_register!(u8, u16, u32);
        test_tuple_register!(u8, u16, u32, u64);
        test_tuple_register!(u8, u16, u32, u64, u128);
        test_tuple_register!(u8, u16, u32, u64, u128, i8);
        test_tuple_register!(u8, u16, u32, u64, u128, i8, i16);
        test_tuple_register!(u8, u16, u32, u64, u128, i8, i16, i32);
        test_tuple_register!(u8, u16, u32, u64, u128, i8, i16, i32, i64);
        test_tuple_register!(u8, u16, u32, u64, u128, i8, i16, i32, i64, i128);
    }

    macro_rules! test_tuple_filtered_component_types {
        ($($params:ident),*) => {{
            let types = <($(With<$params>,)*) as QueryFilter>::filtered_component_types();

            assert_eq!(types, vec![$(TypeId::of::<$params>()),*]);
        }};
    }

    #[test]
    fn retrieve_filtered_component_types() {
        let types = With::<u32>::filtered_component_types();

        assert_eq!(types, vec![TypeId::of::<u32>()]);

        test_tuple_filtered_component_types!();
        test_tuple_filtered_component_types!(u8);
        test_tuple_filtered_component_types!(u8, u16);
        test_tuple_filtered_component_types!(u8, u16, u32);
        test_tuple_filtered_component_types!(u8, u16, u32, u64);
        test_tuple_filtered_component_types!(u8, u16, u32, u64, u128);
        test_tuple_filtered_component_types!(u8, u16, u32, u64, u128, i8);
        test_tuple_filtered_component_types!(u8, u16, u32, u64, u128, i8, i16);
        test_tuple_filtered_component_types!(u8, u16, u32, u64, u128, i8, i16, i32);
        test_tuple_filtered_component_types!(u8, u16, u32, u64, u128, i8, i16, i32, i64);
        test_tuple_filtered_component_types!(u8, u16, u32, u64, u128, i8, i16, i32, i64, i128);
    }
}

#[cfg(test)]
mod query_system_param_tests {
    use super::*;
    use crate::storages::archetypes::ArchetypeStorage;
    use crate::storages::core::CoreStorage;
    use crate::storages::systems::Access;
    use std::ptr;

    #[test]
    fn retrieve_properties() {
        let mut core = CoreStorage::default();

        let properties = Query::<&u32, With<i64>>::properties(&mut core);

        assert_eq!(properties.component_types.len(), 1);
        assert_eq!(properties.component_types[0].access, Access::Read);
        assert_eq!(properties.component_types[0].type_idx, 1.into());
        assert!(!properties.has_entity_actions);
    }

    #[test]
    fn retrieve_iter_info() {
        let core = CoreStorage::default();
        let info = SystemInfo {
            filtered_component_types: vec![],
        };

        let iter_info = Query::<&u32>::iter_info(&core.system_data(), &info);

        assert_eq!(iter_info, SystemParamIterInfo::None);
    }

    #[test]
    fn lock() {
        let core = CoreStorage::default();
        let data = core.system_data();

        let mut guard = Query::<&u32>::lock(&data);
        let guard_borrow = Query::<&u32>::borrow_guard(&mut guard);

        assert!(ptr::eq(guard_borrow, &data));
    }

    #[test]
    fn retrieve_stream_when_no_iteration() {
        let mut core = CoreStorage::default();
        core.add_component_type::<u32>(ArchetypeStorage::DEFAULT_IDX);
        let mut guard_borrow = &core.system_data();
        let iter_info = SystemParamIterInfo::None;

        let mut stream = Query::<&u32>::stream(&mut guard_borrow, &iter_info);

        assert!(Query::<&u32>::stream_next(&mut stream).is_some());
        assert!(Query::<&u32>::stream_next(&mut stream).is_none());
    }

    #[test]
    fn retrieve_stream_when_iteration_on_entities() {
        let mut core = CoreStorage::default();
        core.add_component_type::<u32>(ArchetypeStorage::DEFAULT_IDX);
        let mut guard_borrow = &core.system_data();
        let iter_info = SystemParamIterInfo::new_union(vec![(0.into(), 1), (2.into(), 2)]);

        let mut stream = Query::<&u32>::stream(&mut guard_borrow, &iter_info);

        assert!(Query::<&u32>::stream_next(&mut stream).is_some());
        assert!(Query::<&u32>::stream_next(&mut stream).is_some());
        assert!(Query::<&u32>::stream_next(&mut stream).is_some());
        assert!(Query::<&u32>::stream_next(&mut stream).is_none());
    }
}
