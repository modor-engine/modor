use crate::storages::archetypes::EntityLocation;
use crate::storages::core::CoreStorage;
use crate::storages::systems::SystemProperties;
use crate::system_params::query::internal::QueryFilterProperties;
use crate::systems::context::SystemContext;
use crate::{QuerySystemParam, QuerySystemParamWithLifetime, SystemParam, SystemParamWithLifetime};
use std::iter::Map;
use std::ops::{Deref, DerefMut};

/// A trait for defining a custom system parameter type.
///
/// **Do not implement manually this trait.**<br>
/// The [`SystemParam`](macro@crate::SystemParam) and
/// [`QuerySystemParam`](macro@crate::QuerySystemParam) derive macros can be used instead to
/// define a custom system parameter.
pub trait CustomSystemParam {
    #[doc(hidden)]
    type Param<'b>: CustomSystemParam + 'b;
    #[doc(hidden)]
    type Tuple: SystemParam;

    #[doc(hidden)]
    fn from_tuple_mut_param(
        tuple: <Self::Tuple as SystemParamWithLifetime<'_>>::Param,
    ) -> Custom<Self::Param<'_>>;
}

/// A trait for defining a custom query system parameter type.
///
/// **Do not implement manually this trait.**<br>
/// The [`QuerySystemParam`](macro@crate::QuerySystemParam) derive macros can be used instead to
/// define a custom query system parameter.
pub trait CustomQuerySystemParam: CustomSystemParam
where
    Self::Tuple: QuerySystemParam,
{
    /// Constant version of the system parameter.
    type ConstParam<'b>: CustomSystemParam + 'b;

    #[doc(hidden)]
    fn from_tuple_const_param_mut_param<'b>(
        tuple: <<Self::Tuple as QuerySystemParamWithLifetime<'b>>::ConstParam as SystemParamWithLifetime<'b>>::Param,
    ) -> <Custom<Self::ConstParam<'b>> as SystemParamWithLifetime<'b>>::Param;

    #[doc(hidden)]
    fn from_tuple_const_param(
        tuple: <Self::Tuple as QuerySystemParamWithLifetime<'_>>::ConstParam,
    ) -> Custom<Self::ConstParam<'_>>;
}

/// A type for using a custom system parameter in a system.
///
/// # Examples
///
/// See [`SystemParam`](macro@crate::SystemParam) and
/// [`QuerySystemParam`](macro@crate::QuerySystemParam)
pub struct Custom<T>
where
    T: CustomSystemParam,
{
    inner: T,
}

impl<T> Custom<T>
where
    T: CustomSystemParam,
{
    #[doc(hidden)]
    pub fn new(param: T) -> Self {
        Self { inner: param }
    }
}

impl<T> Deref for Custom<T>
where
    T: CustomSystemParam,
{
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T> DerefMut for Custom<T>
where
    T: CustomSystemParam,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl<'a, T> SystemParamWithLifetime<'a> for Custom<T>
where
    T: CustomSystemParam,
{
    type Param = Custom<T::Param<'a>>;
    type Guard = <T::Tuple as SystemParamWithLifetime<'a>>::Guard;
    type GuardBorrow = <T::Tuple as SystemParamWithLifetime<'a>>::GuardBorrow;
    type Stream = <T::Tuple as SystemParamWithLifetime<'a>>::Stream;
}

impl<T> SystemParam for Custom<T>
where
    T: CustomSystemParam,
{
    type Filter = <T::Tuple as SystemParam>::Filter;
    type InnerTuple = T::Tuple;

    fn properties(core: &mut CoreStorage) -> SystemProperties {
        T::Tuple::properties(core)
    }

    fn lock(context: SystemContext<'_>) -> <Self as SystemParamWithLifetime<'_>>::Guard {
        T::Tuple::lock(context)
    }

    fn borrow_guard<'a, 'b>(
        guard: &'a mut <Self as SystemParamWithLifetime<'b>>::Guard,
    ) -> <Self as SystemParamWithLifetime<'a>>::GuardBorrow
    where
        'b: 'a,
    {
        T::Tuple::borrow_guard(guard)
    }

    fn stream<'a, 'b>(
        guard: &'a mut <Self as SystemParamWithLifetime<'b>>::GuardBorrow,
    ) -> <Self as SystemParamWithLifetime<'a>>::Stream
    where
        'b: 'a,
    {
        T::Tuple::stream(guard)
    }

    #[inline]
    fn stream_next<'a, 'b>(
        stream: &'a mut <Self as SystemParamWithLifetime<'b>>::Stream,
    ) -> Option<<Self as SystemParamWithLifetime<'a>>::Param>
    where
        'b: 'a,
    {
        T::Tuple::stream_next(stream).map(|t| T::from_tuple_mut_param(t))
    }
}

impl<'a, T> QuerySystemParamWithLifetime<'a> for Custom<T>
where
    T: CustomQuerySystemParam,
    T::Tuple: QuerySystemParam,
{
    type ConstParam = Custom<T::ConstParam<'a>>;
    type Iter = Map<
        <T::Tuple as QuerySystemParamWithLifetime<'a>>::Iter,
        fn(
            <<T::Tuple as QuerySystemParamWithLifetime<'a>>::ConstParam as SystemParamWithLifetime<'a>>::Param,
        ) -> <Custom< T::ConstParam<'a>> as SystemParamWithLifetime<'a>>::Param,
    >;
    type IterMut = Map<
        <T::Tuple as QuerySystemParamWithLifetime<'a>>::IterMut,
        fn(<T::Tuple as SystemParamWithLifetime<'a>>::Param) -> Custom<T::Param<'a>>,
    >;
}

impl<T> QuerySystemParam for Custom<T>
where
    T: CustomQuerySystemParam,
    T::Tuple: QuerySystemParam,
{
    fn query_iter<'a, 'b>(
        guard: &'a <Self as SystemParamWithLifetime<'b>>::GuardBorrow,
        filter: Option<QueryFilterProperties>,
    ) -> <Self as QuerySystemParamWithLifetime<'a>>::Iter
    where
        'b: 'a,
    {
        T::Tuple::query_iter(guard, filter).map(|t| T::from_tuple_const_param_mut_param(t))
    }

    fn query_iter_mut<'a, 'b>(
        guard: &'a mut <Self as SystemParamWithLifetime<'b>>::GuardBorrow,
        filter: Option<QueryFilterProperties>,
    ) -> <Self as QuerySystemParamWithLifetime<'a>>::IterMut
    where
        'b: 'a,
    {
        T::Tuple::query_iter_mut(guard, filter).map(|t| T::from_tuple_mut_param(t))
    }

    #[inline]
    fn get<'a, 'b>(
        guard: &'a <Self as SystemParamWithLifetime<'b>>::GuardBorrow,
        location: EntityLocation,
    ) -> Option<<Self as QuerySystemParamWithLifetime<'a>>::ConstParam>
    where
        'b: 'a,
    {
        T::Tuple::get(guard, location).map(|t| T::from_tuple_const_param(t))
    }

    #[inline]
    fn get_mut<'a, 'b>(
        guard: &'a mut <Self as SystemParamWithLifetime<'b>>::GuardBorrow,
        location: EntityLocation,
    ) -> Option<<Self as SystemParamWithLifetime<'a>>::Param>
    where
        'b: 'a,
    {
        T::Tuple::get_mut(guard, location).map(|t| T::from_tuple_mut_param(t))
    }

    #[inline]
    fn get_both_mut<'a, 'b>(
        guard: &'a mut <Self as SystemParamWithLifetime<'b>>::GuardBorrow,
        location1: EntityLocation,
        location2: EntityLocation,
    ) -> (
        Option<<Self as SystemParamWithLifetime<'a>>::Param>,
        Option<<Self as SystemParamWithLifetime<'a>>::Param>,
    )
    where
        'b: 'a,
    {
        let (t1, t2) = T::Tuple::get_both_mut(guard, location1, location2);
        (
            t1.map(|t| T::from_tuple_mut_param(t)),
            t2.map(|t| T::from_tuple_mut_param(t)),
        )
    }
}
