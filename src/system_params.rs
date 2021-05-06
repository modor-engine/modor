use crate::internal::components::interfaces::Components;
use crate::{
    ArchetypeInfo, Group, OptionComponentIter, OptionComponentMutIter, Query, QueryMut,
    QueryMutIterator, SystemData, SystemInfo, TypeAccess,
};
use std::any::{Any, TypeId};
use std::iter::{self, Map, Repeat, Zip};
use std::slice::{Iter, IterMut};
use std::sync::{RwLockReadGuard, RwLockWriteGuard};

// TODO: define type to create/delete entities/groups in systems

pub trait SystemParam<'a, 'b>: Sized {
    const HAS_MANDATORY_COMPONENT: bool;
    const HAS_GROUP_ACTIONS: bool;
    type Lock: 'b;
    type Iterator: Iterator<Item = Self>;

    fn component_types() -> Vec<TypeAccess>;

    fn mandatory_component_types() -> Vec<TypeId>;

    fn lock(data: &'b SystemData<'_>) -> Self::Lock;

    fn iter(
        data: &'b SystemData<'_>,
        info: &SystemInfo,
        lock: &'a mut Self::Lock,
        archetype: ArchetypeInfo,
    ) -> Self::Iterator;

    fn get(info: &SystemInfo, lock: &'a mut Self::Lock) -> Self;
}

impl<'a, 'b: 'a, C> SystemParam<'a, 'b> for &'a C
where
    C: Any,
{
    const HAS_MANDATORY_COMPONENT: bool = true;
    const HAS_GROUP_ACTIONS: bool = false;
    type Lock = Option<RwLockReadGuard<'b, Components>>;
    type Iterator = Iter<'a, C>;

    fn component_types() -> Vec<TypeAccess> {
        vec![TypeAccess::Read(TypeId::of::<C>())]
    }

    fn mandatory_component_types() -> Vec<TypeId> {
        vec![TypeId::of::<C>()]
    }

    fn lock(data: &'b SystemData<'_>) -> Self::Lock {
        data.read_components::<C>()
    }

    fn iter(
        data: &'b SystemData<'_>,
        _info: &SystemInfo,
        lock: &'a mut Self::Lock,
        archetype: ArchetypeInfo,
    ) -> Self::Iterator {
        data.component_iter(lock.as_ref().unwrap(), archetype.idx)
            .unwrap()
    }

    fn get(_info: &SystemInfo, _lock: &'a mut Self::Lock) -> Self {
        panic!("single component retrieved")
    }
}

impl<'a, 'b: 'a, C> SystemParam<'a, 'b> for &'a mut C
where
    C: Any,
{
    const HAS_MANDATORY_COMPONENT: bool = true;
    const HAS_GROUP_ACTIONS: bool = false;
    type Lock = Option<RwLockWriteGuard<'b, Components>>;
    type Iterator = IterMut<'a, C>;

    fn component_types() -> Vec<TypeAccess> {
        vec![TypeAccess::Write(TypeId::of::<C>())]
    }

    fn mandatory_component_types() -> Vec<TypeId> {
        vec![TypeId::of::<C>()]
    }

    fn lock(data: &'b SystemData<'_>) -> Self::Lock {
        data.write_components::<C>()
    }

    fn iter(
        data: &'b SystemData<'_>,
        _info: &SystemInfo,
        lock: &'a mut Self::Lock,
        archetype: ArchetypeInfo,
    ) -> Self::Iterator {
        data.component_iter_mut(lock.as_mut().unwrap(), archetype.idx)
            .unwrap()
    }

    fn get(_info: &SystemInfo, _lock: &'a mut Self::Lock) -> Self {
        panic!("single component retrieved")
    }
}

#[allow(clippy::use_self)]
impl<'a, 'b: 'a, C> SystemParam<'a, 'b> for Option<&'a C>
where
    C: Any,
{
    const HAS_MANDATORY_COMPONENT: bool = false;
    const HAS_GROUP_ACTIONS: bool = false;
    type Lock = Option<RwLockReadGuard<'b, Components>>;
    type Iterator = OptionComponentIter<'a, C>;

    fn component_types() -> Vec<TypeAccess> {
        vec![TypeAccess::Read(TypeId::of::<C>())]
    }

    fn mandatory_component_types() -> Vec<TypeId> {
        Vec::new()
    }

    fn lock(data: &'b SystemData<'_>) -> Self::Lock {
        data.read_components::<C>()
    }

    fn iter(
        data: &'b SystemData<'_>,
        _info: &SystemInfo,
        lock: &'a mut Self::Lock,
        archetype: ArchetypeInfo,
    ) -> Self::Iterator {
        OptionComponentIter::new(
            lock.as_ref()
                .and_then(|l| data.component_iter(l, archetype.idx)),
        )
    }

    fn get(_info: &SystemInfo, _lock: &'a mut Self::Lock) -> Self {
        panic!("single component retrieved")
    }
}

#[allow(clippy::use_self)]
impl<'a, 'b: 'a, C> SystemParam<'a, 'b> for Option<&'a mut C>
where
    C: Any,
{
    const HAS_MANDATORY_COMPONENT: bool = false;
    const HAS_GROUP_ACTIONS: bool = false;
    type Lock = Option<RwLockWriteGuard<'b, Components>>;
    type Iterator = OptionComponentMutIter<'a, C>;

    fn component_types() -> Vec<TypeAccess> {
        vec![TypeAccess::Write(TypeId::of::<C>())]
    }

    fn mandatory_component_types() -> Vec<TypeId> {
        Vec::new()
    }

    fn lock(data: &'b SystemData<'_>) -> Self::Lock {
        data.write_components::<C>()
    }

    fn iter(
        data: &'b SystemData<'_>,
        _info: &SystemInfo,
        lock: &'a mut Self::Lock,
        archetype: ArchetypeInfo,
    ) -> Self::Iterator {
        OptionComponentMutIter::new(
            lock.as_mut()
                .and_then(|l| data.component_iter_mut(l, archetype.idx)),
        )
    }

    fn get(_info: &SystemInfo, _lock: &'a mut Self::Lock) -> Self {
        panic!("single component retrieved")
    }
}

impl<'a, 'b: 'a, T> SystemParam<'a, 'b> for Query<'a, T>
where
    T: ConstSystemParam + TupleSystemParam + SystemParam<'a, 'b>,
{
    const HAS_MANDATORY_COMPONENT: bool = false;
    const HAS_GROUP_ACTIONS: bool = T::HAS_GROUP_ACTIONS;
    type Lock = &'b SystemData<'b>;
    type Iterator = Repeat<Self>;

    fn component_types() -> Vec<TypeAccess> {
        T::component_types()
    }

    fn mandatory_component_types() -> Vec<TypeId> {
        Vec::new()
    }

    fn lock(data: &'b SystemData<'_>) -> Self::Lock {
        data
    }

    fn iter(
        _data: &SystemData<'_>,
        info: &SystemInfo,
        lock: &'a mut Self::Lock,
        _archetype: ArchetypeInfo,
    ) -> Self::Iterator {
        iter::repeat(Self::new(lock.clone(), info.group_idx))
    }

    fn get(info: &SystemInfo, lock: &'a mut Self::Lock) -> Self {
        Self::new(lock.clone(), info.group_idx)
    }
}

impl<'a, 'b: 'a, T> SystemParam<'a, 'b> for QueryMut<'a, T>
where
    T: TupleSystemParam + SystemParam<'a, 'b>,
{
    const HAS_MANDATORY_COMPONENT: bool = false;
    const HAS_GROUP_ACTIONS: bool = T::HAS_GROUP_ACTIONS;
    type Lock = &'b SystemData<'b>;
    type Iterator = QueryMutIterator<'a, T>;

    fn component_types() -> Vec<TypeAccess> {
        T::component_types()
    }

    fn mandatory_component_types() -> Vec<TypeId> {
        Vec::new()
    }

    fn lock(data: &'b SystemData<'_>) -> Self::Lock {
        data
    }

    fn iter(
        _data: &SystemData<'_>,
        info: &SystemInfo,
        lock: &'a mut Self::Lock,
        _archetype: ArchetypeInfo,
    ) -> Self::Iterator {
        QueryMutIterator::new(Self::new(lock.clone(), info.group_idx))
    }

    fn get(info: &SystemInfo, lock: &'a mut Self::Lock) -> Self {
        Self::new(lock.clone(), info.group_idx)
    }
}

impl<'a, 'b: 'a> SystemParam<'a, 'b> for Group<'a> {
    const HAS_MANDATORY_COMPONENT: bool = false;
    const HAS_GROUP_ACTIONS: bool = true;
    type Lock = &'b SystemData<'b>;
    type Iterator = Repeat<Self>;

    fn component_types() -> Vec<TypeAccess> {
        Vec::new()
    }

    fn mandatory_component_types() -> Vec<TypeId> {
        Vec::new()
    }

    fn lock(data: &'b SystemData<'_>) -> Self::Lock {
        data
    }

    fn iter(
        _data: &SystemData<'_>,
        _info: &SystemInfo,
        lock: &'a mut Self::Lock,
        archetype: ArchetypeInfo,
    ) -> Self::Iterator {
        iter::repeat(Self::new(archetype.group_idx, lock.clone()))
    }

    fn get(_info: &SystemInfo, _lock: &'a mut Self::Lock) -> Self {
        panic!("group retrieved for no entity")
    }
}

impl<'a, 'b: 'a> SystemParam<'a, 'b> for () {
    const HAS_MANDATORY_COMPONENT: bool = false;
    const HAS_GROUP_ACTIONS: bool = false;
    type Lock = ();
    type Iterator = Repeat<()>;

    fn component_types() -> Vec<TypeAccess> {
        Vec::new()
    }

    fn mandatory_component_types() -> Vec<TypeId> {
        Vec::new()
    }

    fn lock(_data: &'b SystemData<'_>) -> Self::Lock {}

    fn iter(
        _data: &'b SystemData<'_>,
        _info: &SystemInfo,
        _lock: &'a mut Self::Lock,
        _archetype: ArchetypeInfo,
    ) -> Self::Iterator {
        iter::repeat(())
    }

    fn get(_info: &SystemInfo, _lock: &'a mut Self::Lock) -> Self {}
}

macro_rules! impl_system_param_for_tuple {
    ($(($params:ident, $index:tt)),+) => {
        impl<'a, 'b: 'a, $($params),+> SystemParam<'a, 'b> for ($($params,)+)
        where
            $($params: SystemParam<'a, 'b>,)+
        {
            const HAS_MANDATORY_COMPONENT: bool = $($params::HAS_MANDATORY_COMPONENT)||+;
            const HAS_GROUP_ACTIONS: bool = $($params::HAS_GROUP_ACTIONS)||+;
            type Lock = ($($params::Lock,)+);
            #[allow(clippy::type_complexity)]
            type Iterator = impl_system_param_for_tuple!(@iterator_type $($params),+);

            fn component_types() -> Vec<TypeAccess> {
                let mut types = Vec::new();
                $(types.extend($params::component_types().into_iter());)+
                types
            }

            fn mandatory_component_types() -> Vec<TypeId> {
                let mut types = Vec::new();
                $(types.extend($params::mandatory_component_types().into_iter());)+
                types
            }

            fn lock(data: &'b SystemData<'_>) -> Self::Lock {
                ($($params::lock(data),)+)
            }

            fn iter(
                data: &'b SystemData<'_>,
                info: &SystemInfo,
                lock: &'a mut Self::Lock,
                archetype: ArchetypeInfo,
            ) -> Self::Iterator {
                impl_system_param_for_tuple!(
                    @iteration data, lock, archetype, info, $($params, $index),+
                )
            }

            fn get(info: &SystemInfo, lock: &'a mut Self::Lock) -> Self {
                (
                    $($params::get(info, &mut lock.$index),)+
                )
            }
        }
    };
    (
        @iteration
        $data:ident,
        $lock:ident,
        $archetype:ident,
        $info:ident,
        $param:ident,
        $index:tt
    ) => {
        A::iter($data, $info, &mut $lock.$index, $archetype).map(|item| (item,))
    };
    (
        @iteration
        $data:ident,
        $lock:ident,
        $archetype:ident,
        $info:ident,
        $($params:ident, $index:tt),+
    ) => {
        itertools::izip!($($params::iter($data, $info, &mut $lock.$index, $archetype),)+)
    };
    (@iterator_type $param1:ident, $param2:ident) => {
        Zip<$param1::Iterator, $param2::Iterator>
    };
    (@iterator_type $($params:ident),+) => {
        Map<
            impl_system_param_for_tuple!(@iterator_zip $($params),+),
            fn(impl_system_param_for_tuple!(@iterator_tuple $($params),+)) -> ($($params,)+)
        >
    };
    (@iterator_zip $($params:ident),+) => {
        impl_system_param_for_tuple!(@iterator_zip @reverse ($($params),+), ())
    };
    (@iterator_zip @reverse ($param:ident $(, $params:ident)*), ($($reversed_params:ident),*)) => {
        impl_system_param_for_tuple!(@iterator_zip @reverse ($($params),*), ($param $(,$reversed_params)*))
    };
    (@iterator_zip @reverse (), ($($reversed_params:ident),+)) => {
        impl_system_param_for_tuple!(@iterator_zip @generate $($reversed_params),+)
    };
    (@iterator_zip @generate $param:ident $(, $params:ident)+) => {
        Zip<impl_system_param_for_tuple!(@iterator_zip @generate $($params),+), $param::Iterator>
    };
    (@iterator_zip @generate $param:ident) => {
        $param::Iterator
    };
    (@iterator_tuple $($params:ident),+) => {
        impl_system_param_for_tuple!(@iterator_tuple @reverse ($($params),+), ())
    };
    (@iterator_tuple @reverse ($param:ident $(, $params:ident)*), ($($reversed_params:ident),*)) => {
        impl_system_param_for_tuple!(@iterator_tuple @reverse ($($params),*), ($param $(,$reversed_params)*))
    };
    (@iterator_tuple @reverse (), ($($reversed_params:ident),+)) => {
        impl_system_param_for_tuple!(@iterator_tuple @generate $($reversed_params),+)
    };
    (@iterator_tuple @generate $param:ident $(, $params:ident)+) => {
        (impl_system_param_for_tuple!(@iterator_tuple @generate $($params),+), $param)
    };
    (@iterator_tuple @generate $param:ident) => {
        $param
    };
}

run_for_tuples_with_idxs!(impl_system_param_for_tuple);

pub trait TupleSystemParam {}

macro_rules! impl_tuple_system_param {
    ($($param:ident),*) => {
        impl<'a, 'b, $($param),*> TupleSystemParam for ($($param,)*)
        where
            $($param: SystemParam<'a, 'b>,)*
        {
        }
    };
}

impl_tuple_system_param!();
run_for_tuples!(impl_tuple_system_param);

pub trait NotMandatoryComponentSystemParam {}

impl<T> NotMandatoryComponentSystemParam for Query<'_, T> where
    T: ConstSystemParam + TupleSystemParam
{
}

impl<T> NotMandatoryComponentSystemParam for QueryMut<'_, T> where T: TupleSystemParam {}

impl<'a, 'b, T> NotMandatoryComponentSystemParam for Option<T> where T: SystemParam<'a, 'b> {}

pub trait ConstSystemParam {}

impl<C> ConstSystemParam for &C where C: Any {}

impl<T> ConstSystemParam for Query<'_, T> where T: ConstSystemParam + TupleSystemParam {}

impl ConstSystemParam for () {}

macro_rules! impl_const_system_param {
    ($($param:ident),+) => {
        impl<'a, 'b, $($param),+> ConstSystemParam for ($($param,)+)
        where
            $($param: ConstSystemParam + SystemParam<'a, 'b>,)+
        {
        }
    };
}

run_for_tuples!(impl_const_system_param);

pub trait ComponentSystemParam {}

impl<C> ComponentSystemParam for &C where C: Any {}

impl<C> ComponentSystemParam for &mut C where C: Any {}

impl<C> ComponentSystemParam for Option<&C> where C: Any {}

impl<C> ComponentSystemParam for Option<&mut C> where C: Any {}
