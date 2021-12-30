use crate::queries::internal::{QueryGuard, QueryGuardBorrow};
use crate::storages::archetypes::ArchetypeFilter;
use crate::storages::components::ComponentTypeIdx;
use crate::storages::core::CoreStorage;
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
    phantom: PhantomData<F>,
}

impl<'a, P, F> Query<'a, P, F>
where
    P: 'static + QuerySystemParam,
    F: QueryFilter,
{
    fn new(guard: <P as SystemParamWithLifetime<'a>>::GuardBorrow) -> Self {
        Self {
            guard,
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
}

impl<'a, P, F> SystemParamWithLifetime<'a> for Query<'_, P, F>
where
    P: 'static + QuerySystemParam,
    F: QueryFilter,
{
    type Param = Query<'a, P, F>;
    type Guard = QueryGuard<'a, F>;
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
            has_entity_actions: param_properties.has_entity_actions,
            archetype_filter: ArchetypeFilter::None,
        }
    }

    fn lock<'a>(
        data: &'a SystemData<'_>,
        info: &'a SystemInfo,
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
        stream
            .entity_positions
            .next()
            .map(|_| Query::new(P::borrow_guard(&mut stream.guard)))
    }
}

/// A trait implemented for all valid filters that can be applied to a [`Query`](crate::Query).
pub trait QueryFilter: 'static {
    #[doc(hidden)]
    fn register(core: &mut CoreStorage);

    #[doc(hidden)]
    fn filtered_component_type_idxs(data: &SystemData<'_>) -> Vec<ComponentTypeIdx>;
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
    #[doc(hidden)]
    fn register(core: &mut CoreStorage) {
        core.register_component_type::<C>();
    }

    #[doc(hidden)]
    fn filtered_component_type_idxs(data: &SystemData<'_>) -> Vec<ComponentTypeIdx> {
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
            fn filtered_component_type_idxs(data: &SystemData<'_>) -> Vec<ComponentTypeIdx> {
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
    use crate::storages::archetypes::ArchetypeFilter;
    use crate::system_params::{SystemParam, SystemParamWithLifetime};
    use crate::{QueryFilter, SystemData, SystemInfo};
    use std::marker::PhantomData;
    use std::ops::Range;

    pub struct QueryGuard<'a, F> {
        data: &'a SystemData<'a>,
        info: &'a SystemInfo,
        phantom: PhantomData<F>,
    }

    impl<'a, F> QueryGuard<'a, F>
    where
        F: QueryFilter,
    {
        pub(crate) fn new(data: &'a SystemData<'_>, info: &'a SystemInfo) -> Self {
            Self {
                data,
                info,
                phantom: PhantomData,
            }
        }

        pub(crate) fn borrow(&mut self) -> QueryGuardBorrow<'_> {
            QueryGuardBorrow {
                data: self.data,
                param_info: SystemInfo {
                    filtered_component_type_idxs: F::filtered_component_type_idxs(self.data),
                    archetype_filter: ArchetypeFilter::All,
                },
                item_count: self.data.item_count(self.info),
            }
        }
    }

    pub struct QueryGuardBorrow<'a> {
        pub(crate) data: &'a SystemData<'a>,
        pub(crate) param_info: SystemInfo,
        pub(crate) item_count: usize,
    }

    pub struct QueryStream<'a, P>
    where
        P: SystemParam,
    {
        pub(crate) entity_positions: Range<usize>,
        pub(crate) guard: <P as SystemParamWithLifetime<'a>>::Guard,
    }

    impl<'a, P> QueryStream<'a, P>
    where
        P: SystemParam,
    {
        pub(crate) fn new(guard: &'a QueryGuardBorrow<'_>) -> Self {
            QueryStream {
                entity_positions: 0..guard.item_count,
                guard: P::lock(guard.data, &guard.param_info),
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
        let info = SystemInfo {
            filtered_component_type_idxs: vec![2.into()],
            archetype_filter: ArchetypeFilter::All,
        };
        let mut guard = <&mut u32>::lock(&data, &info);
        let guard_borrow = <&mut u32>::borrow_guard(&mut guard);
        let query = Query::<&mut u32, With<i64>>::new(guard_borrow);

        let mut iter = query.iter();

        assert_eq!(iter.next(), Some(&20));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn iter_mut() {
        let mut core = CoreStorage::default();
        create_entity(&mut core, 10_u32, 11_i8);
        create_entity(&mut core, 20_u32, 21_i64);
        create_entity(&mut core, 30_u8, 31_i8);
        let data = core.system_data();
        let info = SystemInfo {
            filtered_component_type_idxs: vec![2.into()],
            archetype_filter: ArchetypeFilter::All,
        };
        let mut guard = <&mut u32>::lock(&data, &info);
        let guard_borrow = <&mut u32>::borrow_guard(&mut guard);
        let mut query = Query::<&mut u32, With<i64>>::new(guard_borrow);

        let mut iter = query.iter_mut();

        assert_eq!(iter.next(), Some(&mut 20));
        assert_eq!(iter.next(), None);
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
    use crate::storages::archetypes::ArchetypeStorage;
    use crate::storages::components::ComponentStorage;
    use crate::SystemData;
    use std::panic::{RefUnwindSafe, UnwindSafe};
    use std::sync::Mutex;

    assert_impl_all!(With<u32>: Sync, Send, UnwindSafe, RefUnwindSafe, Unpin);

    macro_rules! test_tuple_register {
        ($($params:ident),*) => {{
            let mut core = CoreStorage::default();

            <($(With<$params>,)*) as QueryFilter>::register(&mut core);

            $(assert!(core.components().type_idx(TypeId::of::<$params>()).is_some());)*
        }};
    }

    #[test]
    fn register_single_type() {
        let mut core = CoreStorage::default();

        With::<u32>::register(&mut core);

        assert!(core.components().type_idx(TypeId::of::<u32>()).is_some());
    }

    #[test]
    fn register_empty_tuple() {
        test_tuple_register!();
    }

    #[test]
    fn register_1_item_tuple() {
        test_tuple_register!(u8);
    }

    #[test]
    fn register_2_items_tuple() {
        test_tuple_register!(u8, u16);
    }

    #[test]
    fn register_3_items_tuple() {
        test_tuple_register!(u8, u16, u32);
    }

    #[test]
    fn register_4_items_tuple() {
        test_tuple_register!(u8, u16, u32, u64);
    }

    #[test]
    fn register_5_items_tuple() {
        test_tuple_register!(u8, u16, u32, u64, u128);
    }

    #[test]
    fn register_6_items_tuple() {
        test_tuple_register!(u8, u16, u32, u64, u128, i8);
    }

    #[test]
    fn register_7_items_tuple() {
        test_tuple_register!(u8, u16, u32, u64, u128, i8, i16);
    }

    #[test]
    fn register_8_items_tuple() {
        test_tuple_register!(u8, u16, u32, u64, u128, i8, i16, i32);
    }

    #[test]
    fn register_9_items_tuple() {
        test_tuple_register!(u8, u16, u32, u64, u128, i8, i16, i32, i64);
    }

    #[test]
    fn register_10_items_tuple() {
        test_tuple_register!(u8, u16, u32, u64, u128, i8, i16, i32, i64, i128);
    }

    macro_rules! test_tuple_filtered_component_types {
        (($($params:ident),*), ($($indexes:literal),*)) => {{
            #[allow(unused_mut)]
            let mut components = ComponentStorage::default();
            $(components.type_idx_or_create::<$params>();)*
            let data = SystemData {
                components: &components,
                archetypes: &ArchetypeStorage::default(),
                entity_actions: &Mutex::default(),
            };

            let types = <($(With<$params>,)*) as QueryFilter>::filtered_component_type_idxs(&data);

            assert_eq!(types, vec![$($indexes.into()),*]);
        }};
    }

    #[test]
    fn retrieve_filtered_component_types_for_single_type() {
        let mut components = ComponentStorage::default();
        components.type_idx_or_create::<u32>();
        let data = SystemData {
            components: &components,
            archetypes: &ArchetypeStorage::default(),
            entity_actions: &Mutex::default(),
        };

        let types = With::<u32>::filtered_component_type_idxs(&data);

        assert_eq!(types, vec![0.into()]);
    }

    #[test]
    fn retrieve_filtered_component_types_for_empty_tuple() {
        test_tuple_filtered_component_types!((), ());
    }

    #[test]
    fn retrieve_filtered_component_types_for_1_item_tuple() {
        test_tuple_filtered_component_types!((u8), (0));
    }

    #[test]
    fn retrieve_filtered_component_types_for_2_items_tuple() {
        test_tuple_filtered_component_types!((u8, u16), (0, 1));
    }

    #[test]
    fn retrieve_filtered_component_types_for_3_items_tuple() {
        test_tuple_filtered_component_types!((u8, u16, u32), (0, 1, 2));
    }

    #[test]
    fn retrieve_filtered_component_types_for_4_items_tuple() {
        test_tuple_filtered_component_types!((u8, u16, u32, u64), (0, 1, 2, 3));
    }

    #[test]
    fn retrieve_filtered_component_types_for_5_items_tuple() {
        test_tuple_filtered_component_types!((u8, u16, u32, u64, u128), (0, 1, 2, 3, 4));
    }

    #[test]
    fn retrieve_filtered_component_types_for_6_items_tuple() {
        test_tuple_filtered_component_types!((u8, u16, u32, u64, u128, i8), (0, 1, 2, 3, 4, 5));
    }

    #[test]
    fn retrieve_filtered_component_types_for_7_items_tuple() {
        test_tuple_filtered_component_types!(
            (u8, u16, u32, u64, u128, i8, i16),
            (0, 1, 2, 3, 4, 5, 6)
        );
    }

    #[test]
    fn retrieve_filtered_component_types_for_8_items_tuple() {
        test_tuple_filtered_component_types!(
            (u8, u16, u32, u64, u128, i8, i16, i32),
            (0, 1, 2, 3, 4, 5, 6, 7)
        );
    }

    #[test]
    fn retrieve_filtered_component_types_for_9_items_tuple() {
        test_tuple_filtered_component_types!(
            (u8, u16, u32, u64, u128, i8, i16, i32, i64),
            (0, 1, 2, 3, 4, 5, 6, 7, 8)
        );
    }

    #[test]
    fn retrieve_filtered_component_types_for_10_items_tuple() {
        test_tuple_filtered_component_types!(
            (u8, u16, u32, u64, u128, i8, i16, i32, i64, i128),
            (0, 1, 2, 3, 4, 5, 6, 7, 8, 9)
        );
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
        assert_eq!(properties.archetype_filter, ArchetypeFilter::None);
    }

    #[test]
    fn lock() {
        let mut core = CoreStorage::default();
        let archetype1_idx = ArchetypeStorage::DEFAULT_IDX;
        let (type_idx, archetype2_idx) = core.add_component_type::<u32>(archetype1_idx);
        core.add_component_type::<i64>(archetype1_idx);
        let location = core.create_entity(archetype2_idx);
        core.add_component(10_u32, type_idx, location);
        let data = core.system_data();
        let info = SystemInfo {
            filtered_component_type_idxs: vec![0.into()],
            archetype_filter: ArchetypeFilter::All,
        };

        let mut guard = Query::<&u32, With<i64>>::lock(&data, &info);
        let guard_borrow = Query::<&u32, With<i64>>::borrow_guard(&mut guard);

        assert!(ptr::eq(guard_borrow.data, &data));
        let archetype_filter = guard_borrow.param_info.archetype_filter;
        assert_eq!(archetype_filter, ArchetypeFilter::All);
        let filtered_type_idxs = guard_borrow.param_info.filtered_component_type_idxs;
        assert_eq!(filtered_type_idxs, vec![1.into()]);
        assert_eq!(guard_borrow.item_count, 1);
    }

    #[test]
    fn retrieve_stream() {
        let mut core = CoreStorage::default();
        core.add_component_type::<u32>(ArchetypeStorage::DEFAULT_IDX);
        let mut guard_borrow = QueryGuardBorrow {
            data: &core.system_data(),
            param_info: SystemInfo {
                filtered_component_type_idxs: vec![],
                archetype_filter: ArchetypeFilter::All,
            },
            item_count: 3,
        };

        let mut stream = Query::<&u32>::stream(&mut guard_borrow);

        assert!(Query::<&u32>::stream_next(&mut stream).is_some());
        assert!(Query::<&u32>::stream_next(&mut stream).is_some());
        assert!(Query::<&u32>::stream_next(&mut stream).is_some());
        assert!(Query::<&u32>::stream_next(&mut stream).is_none());
    }
}
