use self::internal::SealedSystemParam;
use crate::external::systems::building::internal::TypeAccess;
use crate::external::systems::definition::internal::{
    ArchetypeInfo, ComponentsConst, ComponentsMut,
};
use crate::{
    Entity, EntityIter, Group, GroupIter, OptionComponentIter, OptionComponentMutIter, Query,
    QueryIter, SystemData, SystemInfo,
};
use std::any::{Any, TypeId};
use std::iter::{self, Map, Repeat, Zip};
use std::slice::{Iter, IterMut};

/// Characterise a type that can be a parameter of a [`System`](crate::System).
pub trait SystemParam<'a, 'b>: SealedSystemParam {
    #[doc(hidden)]
    const HAS_MANDATORY_COMPONENT: bool;
    #[doc(hidden)]
    const HAS_ACTIONS: bool;
    #[doc(hidden)]
    type Lock: 'b;
    #[doc(hidden)]
    type Iter: Iterator<Item = Self>;

    #[doc(hidden)]
    fn component_types() -> Vec<TypeAccess>;

    #[doc(hidden)]
    fn mandatory_component_types() -> Vec<TypeId>;

    #[doc(hidden)]
    fn lock(data: &'b SystemData<'_>) -> Self::Lock;

    #[doc(hidden)]
    fn iter(
        data: &'b SystemData<'_>,
        info: &SystemInfo,
        lock: &'a mut Self::Lock,
        archetype: ArchetypeInfo,
    ) -> Self::Iter;

    #[doc(hidden)]
    fn get(info: &SystemInfo, lock: &'a mut Self::Lock) -> Self;
}

impl<'a, 'b: 'a, C> SealedSystemParam for &'a C {}

impl<'a, 'b: 'a, C> SystemParam<'a, 'b> for &'a C
where
    C: Any,
{
    const HAS_MANDATORY_COMPONENT: bool = true;
    const HAS_ACTIONS: bool = false;
    type Lock = Option<ComponentsConst<'b>>;
    type Iter = Iter<'a, C>;

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
    ) -> Self::Iter {
        let components_guard = &lock
            .as_ref()
            .expect("internal error: access to not existing components")
            .0;
        data.component_iter(components_guard, archetype.idx)
            .expect("internal error: iter on mandatory components that does not exist")
    }

    fn get(_info: &SystemInfo, _lock: &'a mut Self::Lock) -> Self {
        panic!("single component retrieved")
    }
}

impl<'a, 'b: 'a, C> SealedSystemParam for &'a mut C {}

impl<'a, 'b: 'a, C> SystemParam<'a, 'b> for &'a mut C
where
    C: Any,
{
    const HAS_MANDATORY_COMPONENT: bool = true;
    const HAS_ACTIONS: bool = false;
    type Lock = Option<ComponentsMut<'b>>;
    type Iter = IterMut<'a, C>;

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
    ) -> Self::Iter {
        let components_guard = &mut lock
            .as_mut()
            .expect("internal error: mutably access to not existing components")
            .0;
        data.component_iter_mut(components_guard, archetype.idx)
            .expect("internal error: mutably iter on mandatory components that does not exist")
    }

    fn get(_info: &SystemInfo, _lock: &'a mut Self::Lock) -> Self {
        panic!("single component retrieved")
    }
}

impl<'a, 'b: 'a, C> SealedSystemParam for Option<&'a C> {}

#[allow(clippy::use_self)]
impl<'a, 'b: 'a, C> SystemParam<'a, 'b> for Option<&'a C>
where
    C: Any,
{
    const HAS_MANDATORY_COMPONENT: bool = false;
    const HAS_ACTIONS: bool = false;
    type Lock = Option<ComponentsConst<'b>>;
    type Iter = OptionComponentIter<'a, C>;

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
    ) -> Self::Iter {
        OptionComponentIter::new(
            lock.as_ref()
                .and_then(|l| data.component_iter(&l.0, archetype.idx)),
        )
    }

    fn get(_info: &SystemInfo, _lock: &'a mut Self::Lock) -> Self {
        panic!("single component retrieved")
    }
}

impl<'a, 'b: 'a, C> SealedSystemParam for Option<&'a mut C> {}

#[allow(clippy::use_self)]
impl<'a, 'b: 'a, C> SystemParam<'a, 'b> for Option<&'a mut C>
where
    C: Any,
{
    const HAS_MANDATORY_COMPONENT: bool = false;
    const HAS_ACTIONS: bool = false;
    type Lock = Option<ComponentsMut<'b>>;
    type Iter = OptionComponentMutIter<'a, C>;

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
    ) -> Self::Iter {
        OptionComponentMutIter::new(
            lock.as_mut()
                .and_then(|l| data.component_iter_mut(&mut l.0, archetype.idx)),
        )
    }

    fn get(_info: &SystemInfo, _lock: &'a mut Self::Lock) -> Self {
        panic!("single component retrieved")
    }
}

impl<'a, 'b: 'a> SealedSystemParam for Group<'a> {}

impl<'a, 'b: 'a> SystemParam<'a, 'b> for Group<'a> {
    const HAS_MANDATORY_COMPONENT: bool = false;
    const HAS_ACTIONS: bool = true;
    type Lock = &'b SystemData<'b>;
    type Iter = GroupIter<'a>;

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
    ) -> Self::Iter {
        GroupIter::new(archetype.group_idx, lock.clone())
    }

    fn get(_info: &SystemInfo, _lock: &'a mut Self::Lock) -> Self {
        panic!("group retrieved with no entity component")
    }
}

impl<'a, 'b: 'a> SealedSystemParam for Entity<'a> {}

impl<'a, 'b: 'a> SystemParam<'a, 'b> for Entity<'a> {
    const HAS_MANDATORY_COMPONENT: bool = false;
    const HAS_ACTIONS: bool = true;
    type Lock = &'b SystemData<'b>;
    type Iter = EntityIter<'a>;

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
    ) -> Self::Iter {
        EntityIter::new(lock.entity_idxs(archetype.idx).iter(), lock.clone())
    }

    fn get(_info: &SystemInfo, _lock: &'a mut Self::Lock) -> Self {
        panic!("entity retrieved with no entity component")
    }
}

impl<'a, 'b: 'a, T> SealedSystemParam for Query<'a, T> where T: TupleSystemParam {}

impl<'a, 'b: 'a, T> SystemParam<'a, 'b> for Query<'a, T>
where
    T: TupleSystemParam + SystemParam<'a, 'b>,
{
    const HAS_MANDATORY_COMPONENT: bool = false;
    const HAS_ACTIONS: bool = T::HAS_ACTIONS;
    type Lock = &'b SystemData<'b>;
    type Iter = QueryIter<'a, T>;

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
    ) -> Self::Iter {
        QueryIter::new(Self::new(lock.clone(), info.group_idx))
    }

    fn get(info: &SystemInfo, lock: &'a mut Self::Lock) -> Self {
        Self::new(lock.clone(), info.group_idx)
    }
}

impl<'a, 'b: 'a> SealedSystemParam for () {}

impl<'a, 'b: 'a> SystemParam<'a, 'b> for () {
    const HAS_MANDATORY_COMPONENT: bool = false;
    const HAS_ACTIONS: bool = false;
    type Lock = ();
    type Iter = Repeat<()>;

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
    ) -> Self::Iter {
        iter::repeat(())
    }

    fn get(_info: &SystemInfo, _lock: &'a mut Self::Lock) -> Self {}
}

macro_rules! impl_system_param_for_tuple {
    ($(($params:ident, $indexes:tt)),+) => {
        impl<'a, 'b: 'a, $($params),+> SealedSystemParam for ($($params,)+) {}

        impl<'a, 'b: 'a, $($params),+> SystemParam<'a, 'b> for ($($params,)+)
        where
            $($params: SystemParam<'a, 'b>,)+
        {
            const HAS_MANDATORY_COMPONENT: bool = $($params::HAS_MANDATORY_COMPONENT)||+;
            const HAS_ACTIONS: bool = $($params::HAS_ACTIONS)||+;
            type Lock = ($($params::Lock,)+);
            #[allow(clippy::type_complexity)]
            type Iter = impl_system_param_for_tuple!(@iterator_type $($params),+);

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
            ) -> Self::Iter {
                impl_system_param_for_tuple!(
                    @iteration data, lock, archetype, info, $($params, $indexes),+
                )
            }

            fn get(info: &SystemInfo, lock: &'a mut Self::Lock) -> Self {
                (
                    $($params::get(info, &mut lock.$indexes),)+
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
        Zip<$param1::Iter, $param2::Iter>
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
        impl_system_param_for_tuple!(
            @iterator_zip @reverse
            ($($params),*),
            ($param $(,$reversed_params)*)
        )
    };
    (@iterator_zip @reverse (), ($($reversed_params:ident),+)) => {
        impl_system_param_for_tuple!(@iterator_zip @generate $($reversed_params),+)
    };
    (@iterator_zip @generate $param:ident $(, $params:ident)+) => {
        Zip<impl_system_param_for_tuple!(@iterator_zip @generate $($params),+), $param::Iter>
    };
    (@iterator_zip @generate $param:ident) => {
        $param::Iter
    };
    (@iterator_tuple $($params:ident),+) => {
        impl_system_param_for_tuple!(@iterator_tuple @reverse ($($params),+), ())
    };
    (@iterator_tuple @reverse ($param:ident $(, $params:ident)*), ($($reversed_params:ident),*)) => {
        impl_system_param_for_tuple!(
            @iterator_tuple @reverse
            ($($params),*),
            ($param $(,$reversed_params)*)
        )
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

/// Characterise a tuple of [`SystemParam`](crate::SystemParam) items.
pub trait TupleSystemParam: SealedSystemParam {}

macro_rules! impl_tuple_system_param {
    ($($params:ident),*) => {
        impl<'a, 'b $(,$params)*> TupleSystemParam for ($($params,)*)
        where
            $($params: SystemParam<'a, 'b>),*
        {
        }
    };
}

impl_tuple_system_param!();
run_for_tuples!(impl_tuple_system_param);

/// Characterise a [`SystemParam`](crate::SystemParam) that accesses to resources in an immutable
/// way.
pub trait ConstSystemParam: SealedSystemParam {}

impl<C> ConstSystemParam for &C where C: Any {}

impl<C> ConstSystemParam for Option<&C> where C: Any {}

impl<T> ConstSystemParam for Query<'_, T> where T: TupleSystemParam + ConstSystemParam {}

macro_rules! impl_const_system_param {
    ($($params:ident),*) => {
        impl<'a, 'b, $($params),*> ConstSystemParam for ($($params,)*)
        where
            $($params: ConstSystemParam + SystemParam<'a, 'b>,)*
        {
        }
    };
}

impl_const_system_param!();
run_for_tuples!(impl_const_system_param);

pub(crate) mod internal {
    use crate::{Entity, Group, Query, TupleSystemParam};
    use std::any::Any;

    pub trait SealedSystemParam {}

    pub trait MultipleSystemParams: SealedSystemParam {
        type TupleSystemParams: TupleSystemParam;
    }

    impl<T> MultipleSystemParams for T
    where
        T: TupleSystemParam,
    {
        type TupleSystemParams = Self;
    }

    impl<T> MultipleSystemParams for Query<'_, T>
    where
        T: TupleSystemParam,
    {
        type TupleSystemParams = T;
    }

    pub struct Const;

    pub struct Mut;

    pub trait EntityPartSystemParam: SealedSystemParam {
        type Resource;
        type Mutability;
    }

    impl<C> EntityPartSystemParam for &C
    where
        C: Any,
    {
        type Resource = C;
        type Mutability = Const;
    }

    impl<C> EntityPartSystemParam for Option<&C>
    where
        C: Any,
    {
        type Resource = C;
        type Mutability = Const;
    }

    impl<C> EntityPartSystemParam for &mut C
    where
        C: Any,
    {
        type Resource = C;
        type Mutability = Mut;
    }

    impl<C> EntityPartSystemParam for Option<&mut C>
    where
        C: Any,
    {
        type Resource = C;
        type Mutability = Mut;
    }

    impl EntityPartSystemParam for Group<'_> {
        type Resource = Group<'static>;
        type Mutability = Mut;
    }

    impl EntityPartSystemParam for Entity<'_> {
        type Resource = Entity<'static>;
        type Mutability = Mut;
    }

    pub trait NotEnoughEntityPartSystemParam: SealedSystemParam {}

    impl<C> NotEnoughEntityPartSystemParam for Option<&C> where C: Any {}

    impl<C> NotEnoughEntityPartSystemParam for Option<&mut C> where C: Any {}

    impl NotEnoughEntityPartSystemParam for Group<'_> {}

    impl NotEnoughEntityPartSystemParam for Entity<'_> {}

    macro_rules! impl_not_enough_entity_part_system_param {
    ($($params:ident),*) => {
        impl<$($params),*> NotEnoughEntityPartSystemParam for ($($params,)*)
        where
            $($params: NotEnoughEntityPartSystemParam,)*
        {
        }
    };
}

    run_for_tuples!(impl_not_enough_entity_part_system_param);

    pub trait NotMandatoryComponentSystemParam: SealedSystemParam {}

    impl<C> NotMandatoryComponentSystemParam for Option<&C> where C: Any {}

    impl<C> NotMandatoryComponentSystemParam for Option<&mut C> where C: Any {}

    impl NotMandatoryComponentSystemParam for Group<'_> {}

    impl NotMandatoryComponentSystemParam for Entity<'_> {}

    impl<T> NotMandatoryComponentSystemParam for Query<'_, T> where T: TupleSystemParam {}

    pub trait QuerySystemParam: SealedSystemParam {}

    impl<T> QuerySystemParam for Query<'_, T> where T: TupleSystemParam {}
}
