use self::internal::SealedSystemParam;
use crate::external::systems::building::internal::TypeAccess;
use crate::external::systems::definition::internal::{
    ArchetypeInfo, Components, ComponentsMut,
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
    const HAS_ENTITY_PART: bool;
    #[doc(hidden)]
    const HAS_ACTIONS: bool;
    #[doc(hidden)]
    type Guard: 'b;
    #[doc(hidden)]
    type Iter: Iterator<Item = Self>;

    #[doc(hidden)]
    fn component_types() -> Vec<TypeAccess>;

    #[doc(hidden)]
    fn mandatory_component_types() -> Vec<TypeId>;

    #[doc(hidden)]
    fn lock(data: &'b SystemData<'_>) -> Self::Guard;

    #[doc(hidden)]
    fn iter(
        data: &'b SystemData<'_>,
        info: &SystemInfo,
        guard: &'a mut Self::Guard,
        archetype: ArchetypeInfo,
    ) -> Self::Iter;

    #[doc(hidden)]
    fn get(info: &SystemInfo, guard: &'a mut Self::Guard) -> Self;
}

impl<'a, 'b: 'a, C> SealedSystemParam for &'a C {}

impl<'a, 'b: 'a, C> SystemParam<'a, 'b> for &'a C
where
    C: Any,
{
    const HAS_MANDATORY_COMPONENT: bool = true;
    const HAS_ENTITY_PART: bool = true;
    const HAS_ACTIONS: bool = false;
    type Guard = Option<Components<'b, C>>;
    type Iter = Iter<'a, C>;

    fn component_types() -> Vec<TypeAccess> {
        vec![TypeAccess::Read(TypeId::of::<C>())]
    }

    fn mandatory_component_types() -> Vec<TypeId> {
        vec![TypeId::of::<C>()]
    }

    fn lock(data: &'b SystemData<'_>) -> Self::Guard {
        data.read_components::<C>()
    }

    fn iter(
        _data: &'b SystemData<'_>,
        _info: &SystemInfo,
        guard: &'a mut Self::Guard,
        archetype: ArchetypeInfo,
    ) -> Self::Iter {
        let components_guard = &guard
            .as_ref()
            .expect("internal error: access to not existing components")
            .0;
        components_guard
            .archetype_iter(archetype.idx)
            .expect("internal error: iterate on missing archetype")
    }

    fn get(_info: &SystemInfo, _guard: &'a mut Self::Guard) -> Self {
        panic!("single component retrieved")
    }
}

impl<'a, 'b: 'a, C> SealedSystemParam for &'a mut C {}

impl<'a, 'b: 'a, C> SystemParam<'a, 'b> for &'a mut C
where
    C: Any,
{
    const HAS_MANDATORY_COMPONENT: bool = true;
    const HAS_ENTITY_PART: bool = true;
    const HAS_ACTIONS: bool = false;
    type Guard = Option<ComponentsMut<'b, C>>;
    type Iter = IterMut<'a, C>;

    fn component_types() -> Vec<TypeAccess> {
        vec![TypeAccess::Write(TypeId::of::<C>())]
    }

    fn mandatory_component_types() -> Vec<TypeId> {
        vec![TypeId::of::<C>()]
    }

    fn lock(data: &'b SystemData<'_>) -> Self::Guard {
        data.write_components::<C>()
    }

    fn iter(
        _data: &'b SystemData<'_>,
        _info: &SystemInfo,
        guard: &'a mut Self::Guard,
        archetype: ArchetypeInfo,
    ) -> Self::Iter {
        let components_guard = &mut guard
            .as_mut()
            .expect("internal error: mutably access to not existing components")
            .0;
        components_guard
            .archetype_iter_mut(archetype.idx)
            .expect("internal error: mutably iterate on mandatory components that does not exist")
    }

    fn get(_info: &SystemInfo, _guard: &'a mut Self::Guard) -> Self {
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
    const HAS_ENTITY_PART: bool = true;
    const HAS_ACTIONS: bool = false;
    type Guard = Option<Components<'b, C>>;
    type Iter = OptionComponentIter<'a, C>;

    fn component_types() -> Vec<TypeAccess> {
        vec![TypeAccess::Read(TypeId::of::<C>())]
    }

    fn mandatory_component_types() -> Vec<TypeId> {
        Vec::new()
    }

    fn lock(data: &'b SystemData<'_>) -> Self::Guard {
        data.read_components::<C>()
    }

    fn iter(
        data: &'b SystemData<'_>,
        _info: &SystemInfo,
        guard: &'a mut Self::Guard,
        archetype: ArchetypeInfo,
    ) -> Self::Iter {
        OptionComponentIter::new(
            guard
                .as_ref()
                .and_then(|l| l.0.archetype_iter(archetype.idx)),
            data.entity_idxs(archetype.idx).len(),
        )
    }

    fn get(_info: &SystemInfo, _guard: &'a mut Self::Guard) -> Self {
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
    const HAS_ENTITY_PART: bool = true;
    const HAS_ACTIONS: bool = false;
    type Guard = Option<ComponentsMut<'b, C>>;
    type Iter = OptionComponentMutIter<'a, C>;

    fn component_types() -> Vec<TypeAccess> {
        vec![TypeAccess::Write(TypeId::of::<C>())]
    }

    fn mandatory_component_types() -> Vec<TypeId> {
        Vec::new()
    }

    fn lock(data: &'b SystemData<'_>) -> Self::Guard {
        data.write_components::<C>()
    }

    fn iter(
        data: &'b SystemData<'_>,
        _info: &SystemInfo,
        guard: &'a mut Self::Guard,
        archetype: ArchetypeInfo,
    ) -> Self::Iter {
        OptionComponentMutIter::new(
            guard
                .as_mut()
                .and_then(|l| l.0.archetype_iter_mut(archetype.idx)),
            data.entity_idxs(archetype.idx).len(),
        )
    }

    fn get(_info: &SystemInfo, _guard: &'a mut Self::Guard) -> Self {
        panic!("single component retrieved")
    }
}

impl<'a, 'b: 'a> SealedSystemParam for Group<'a> {}

impl<'a, 'b: 'a> SystemParam<'a, 'b> for Group<'a> {
    const HAS_MANDATORY_COMPONENT: bool = false;
    const HAS_ENTITY_PART: bool = true;
    const HAS_ACTIONS: bool = true;
    type Guard = &'b SystemData<'b>;
    type Iter = GroupIter<'a>;

    fn component_types() -> Vec<TypeAccess> {
        Vec::new()
    }

    fn mandatory_component_types() -> Vec<TypeId> {
        Vec::new()
    }

    fn lock(data: &'b SystemData<'_>) -> Self::Guard {
        data
    }

    fn iter(
        _data: &SystemData<'_>,
        _info: &SystemInfo,
        guard: &'a mut Self::Guard,
        archetype: ArchetypeInfo,
    ) -> Self::Iter {
        GroupIter::new(
            archetype.group_idx,
            guard.clone(),
            guard.entity_idxs(archetype.idx).len(),
        )
    }

    fn get(_info: &SystemInfo, _guard: &'a mut Self::Guard) -> Self {
        panic!("group retrieved with no entity component")
    }
}

impl<'a, 'b: 'a> SealedSystemParam for Entity<'a> {}

impl<'a, 'b: 'a> SystemParam<'a, 'b> for Entity<'a> {
    const HAS_MANDATORY_COMPONENT: bool = false;
    const HAS_ENTITY_PART: bool = true;
    const HAS_ACTIONS: bool = true;
    type Guard = &'b SystemData<'b>;
    type Iter = EntityIter<'a>;

    fn component_types() -> Vec<TypeAccess> {
        Vec::new()
    }

    fn mandatory_component_types() -> Vec<TypeId> {
        Vec::new()
    }

    fn lock(data: &'b SystemData<'_>) -> Self::Guard {
        data
    }

    fn iter(
        _data: &SystemData<'_>,
        _info: &SystemInfo,
        guard: &'a mut Self::Guard,
        archetype: ArchetypeInfo,
    ) -> Self::Iter {
        EntityIter::new(guard.entity_idxs(archetype.idx).iter(), guard.clone())
    }

    fn get(_info: &SystemInfo, _guard: &'a mut Self::Guard) -> Self {
        panic!("entity retrieved with no entity component")
    }
}

impl<'a, 'b: 'a, T> SealedSystemParam for Query<'a, T> where T: TupleSystemParam {}

impl<'a, 'b: 'a, T> SystemParam<'a, 'b> for Query<'a, T>
where
    T: TupleSystemParam + SystemParam<'a, 'b>,
{
    const HAS_MANDATORY_COMPONENT: bool = false;
    const HAS_ENTITY_PART: bool = false;
    const HAS_ACTIONS: bool = T::HAS_ACTIONS;
    type Guard = &'b SystemData<'b>;
    type Iter = QueryIter<'a, T>;

    fn component_types() -> Vec<TypeAccess> {
        T::component_types()
    }

    fn mandatory_component_types() -> Vec<TypeId> {
        Vec::new()
    }

    fn lock(data: &'b SystemData<'_>) -> Self::Guard {
        data
    }

    fn iter(
        _data: &SystemData<'_>,
        info: &SystemInfo,
        guard: &'a mut Self::Guard,
        archetype: ArchetypeInfo,
    ) -> Self::Iter {
        QueryIter::new(
            Self::new(info.group_idx, guard.clone()),
            guard.entity_idxs(archetype.idx).len(),
        )
    }

    fn get(info: &SystemInfo, guard: &'a mut Self::Guard) -> Self {
        Self::new(info.group_idx, guard.clone())
    }
}

macro_rules! impl_system_param_for_tuple {
    ($(($params:ident, $indexes:tt)),*) => {
        impl<'a, 'b: 'a, $($params),*> SealedSystemParam for ($($params,)*) {}

        impl<'a, 'b: 'a, $($params),*> SystemParam<'a, 'b> for ($($params,)*)
        where
            $($params: SystemParam<'a, 'b>,)*
        {
            const HAS_MANDATORY_COMPONENT: bool =
                impl_system_param_for_tuple!(@condition $($params::HAS_MANDATORY_COMPONENT),*);
            const HAS_ENTITY_PART: bool =
                impl_system_param_for_tuple!(@condition $($params::HAS_ENTITY_PART),*);
            const HAS_ACTIONS: bool =
                impl_system_param_for_tuple!(@condition $($params::HAS_ACTIONS),*);
            type Guard = ($($params::Guard,)*);
            #[allow(clippy::type_complexity)]
            type Iter = impl_system_param_for_tuple!(@iterator_type $($params),*);

            #[allow(unused_mut)]
            fn component_types() -> Vec<TypeAccess> {
                let mut types = Vec::new();
                $(types.extend($params::component_types().into_iter());)*
                types
            }

            #[allow(unused_mut)]
            fn mandatory_component_types() -> Vec<TypeId> {
                let mut types = Vec::new();
                $(types.extend($params::mandatory_component_types().into_iter());)*
                types
            }

            #[allow(unused_variables)]
            fn lock(data: &'b SystemData<'_>) -> Self::Guard {
                ($($params::lock(data),)*)
            }

            #[allow(unused_variables)]
            fn iter(
                data: &'b SystemData<'_>,
                info: &SystemInfo,
                guard: &'a mut Self::Guard,
                archetype: ArchetypeInfo,
            ) -> Self::Iter {
                impl_system_param_for_tuple!(
                    @iteration data, guard, archetype, info $(,$params, $indexes)*
                )
            }

            #[allow(unused_variables)]
            fn get(info: &SystemInfo, guard: &'a mut Self::Guard) -> Self {
                (
                    $($params::get(info, &mut guard.$indexes),)*
                )
            }
        }
    };
    (@condition $($term:expr),+) => { $($term)||+ };
    (@condition) => { false };
    (
        @iteration
        $data:ident,
        $guard:ident,
        $archetype:ident,
        $info:ident
    ) => {
        iter::repeat(())
    };
    (
        @iteration
        $data:ident,
        $guard:ident,
        $archetype:ident,
        $info:ident,
        $param:ident,
        $index:tt
    ) => {
        A::iter($data, $info, &mut $guard.$index, $archetype).map(|item| (item,))
    };
    (
        @iteration
        $data:ident,
        $guard:ident,
        $archetype:ident,
        $info:ident,
        $($params:ident, $index:tt),+
    ) => {
        itertools::izip!($($params::iter($data, $info, &mut $guard.$index, $archetype),)+)
    };
    (@iterator_type) => {
        Repeat<()>
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

impl_system_param_for_tuple!();
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

#[cfg(test)]
mod component_system_param_tests {
    use super::*;
    use crate::internal::main::MainFacade;
    use crate::SystemInfo;

    type Param<'a> = &'a u32;

    #[test]
    fn retrieve_component_types() {
        let component_types = Param::component_types();

        assert_eq!(component_types, [TypeAccess::Read(TypeId::of::<u32>())])
    }

    #[test]
    fn retrieve_mandatory_component_types() {
        let component_types = Param::mandatory_component_types();

        assert_eq!(component_types, [TypeId::of::<u32>()])
    }

    #[test]
    fn lock() {
        let mut main = MainFacade::default();
        let group_idx = main.create_group();
        let entity1_idx = main.create_entity(group_idx);
        let entity2_idx = main.create_entity(group_idx);
        main.add_component(entity1_idx, 10_u32);
        main.add_component(entity2_idx, 20_u32);
        let data = main.system_data();

        let guard = Param::lock(&data);

        let components = guard.unwrap();
        assert_option_iter!(components.0.archetype_iter(0), Some(vec![&10, &20]));
    }

    #[test]
    fn retrieve_iter_using_valid_guard() {
        let mut main = MainFacade::default();
        let group_idx = main.create_group();
        let entity1_idx = main.create_entity(group_idx);
        let entity2_idx = main.create_entity(group_idx);
        main.add_component(entity1_idx, 10_u32);
        main.add_component(entity2_idx, 20_u32);
        let data = main.system_data();
        let info = SystemInfo::new(Vec::new(), None);
        let mut guard = Param::lock(&data);
        let archetype = ArchetypeInfo::new(0, group_idx);

        let iter = Param::iter(&data, &info, &mut guard, archetype);

        assert_iter!(iter, [&10, &20]);
    }

    #[test]
    fn retrieve_iter_using_invalid_guard() {
        let mut main = MainFacade::default();
        let group_idx = main.create_group();
        let entity1_idx = main.create_entity(group_idx);
        let entity2_idx = main.create_entity(group_idx);
        main.add_component(entity1_idx, 10_u32);
        main.add_component(entity2_idx, 20_u32);
        let data = main.system_data();
        let info = SystemInfo::new(Vec::new(), None);
        let mut guard = None;
        let archetype = ArchetypeInfo::new(0, group_idx);

        assert_panics!(Param::iter(&data, &info, &mut guard, archetype));
    }

    #[test]
    fn retrieve_iter_using_wrong_archetype_idx() {
        let mut main = MainFacade::default();
        let group_idx = main.create_group();
        let entity1_idx = main.create_entity(group_idx);
        let entity2_idx = main.create_entity(group_idx);
        main.add_component(entity1_idx, 10_u32);
        main.add_component(entity2_idx, 20_u32);
        let data = main.system_data();
        let info = SystemInfo::new(Vec::new(), None);
        let mut guard = Param::lock(&data);
        let archetype = ArchetypeInfo::new(1, group_idx);

        assert_panics!(Param::iter(&data, &info, &mut guard, archetype));
    }

    #[test]
    fn get_single_value() {
        let mut main = MainFacade::default();
        let group_idx = main.create_group();
        let entity1_idx = main.create_entity(group_idx);
        let entity2_idx = main.create_entity(group_idx);
        main.add_component(entity1_idx, 10_u32);
        main.add_component(entity2_idx, 20_u32);
        let data = main.system_data();
        let info = SystemInfo::new(Vec::new(), None);
        let mut guard = Param::lock(&data);

        assert_panics!(Param::get(&info, &mut guard));
    }
}

#[cfg(test)]
mod component_mut_system_param_tests {
    use super::*;
    use crate::internal::main::MainFacade;
    use crate::SystemInfo;

    type Param<'a> = &'a mut u32;

    #[test]
    fn retrieve_component_types() {
        let component_types = Param::component_types();

        assert_eq!(component_types, [TypeAccess::Write(TypeId::of::<u32>())])
    }

    #[test]
    fn retrieve_mandatory_component_types() {
        let component_types = Param::mandatory_component_types();

        assert_eq!(component_types, [TypeId::of::<u32>()])
    }

    #[test]
    fn lock() {
        let mut main = MainFacade::default();
        let group_idx = main.create_group();
        let entity1_idx = main.create_entity(group_idx);
        let entity2_idx = main.create_entity(group_idx);
        main.add_component(entity1_idx, 10_u32);
        main.add_component(entity2_idx, 20_u32);
        let data = main.system_data();

        let guard = Param::lock(&data);

        let mut components = guard.unwrap();
        let component_iter = components.0.archetype_iter_mut(0);
        assert_option_iter!(component_iter, Some(vec![&mut 10, &mut 20]));
    }

    #[test]
    fn retrieve_iter_using_valid_guard() {
        let mut main = MainFacade::default();
        let group_idx = main.create_group();
        let entity1_idx = main.create_entity(group_idx);
        let entity2_idx = main.create_entity(group_idx);
        main.add_component(entity1_idx, 10_u32);
        main.add_component(entity2_idx, 20_u32);
        let data = main.system_data();
        let info = SystemInfo::new(Vec::new(), None);
        let mut guard = Param::lock(&data);
        let archetype = ArchetypeInfo::new(0, group_idx);

        let iter = Param::iter(&data, &info, &mut guard, archetype);

        assert_iter!(iter, [&10, &20]);
    }

    #[test]
    fn retrieve_iter_using_invalid_guard() {
        let mut main = MainFacade::default();
        let group_idx = main.create_group();
        let entity1_idx = main.create_entity(group_idx);
        let entity2_idx = main.create_entity(group_idx);
        main.add_component(entity1_idx, 10_u32);
        main.add_component(entity2_idx, 20_u32);
        let data = main.system_data();
        let info = SystemInfo::new(Vec::new(), None);
        let mut guard = None;
        let archetype = ArchetypeInfo::new(0, group_idx);

        assert_panics!(Param::iter(&data, &info, &mut guard, archetype));
    }

    #[test]
    fn retrieve_iter_using_wrong_archetype_idx() {
        let mut main = MainFacade::default();
        let group_idx = main.create_group();
        let entity1_idx = main.create_entity(group_idx);
        let entity2_idx = main.create_entity(group_idx);
        main.add_component(entity1_idx, 10_u32);
        main.add_component(entity2_idx, 20_u32);
        let data = main.system_data();
        let info = SystemInfo::new(Vec::new(), None);
        let mut guard = Param::lock(&data);
        let archetype = ArchetypeInfo::new(1, group_idx);

        assert_panics!(Param::iter(&data, &info, &mut guard, archetype));
    }

    #[test]
    fn get_single_value() {
        let mut main = MainFacade::default();
        let group_idx = main.create_group();
        let entity1_idx = main.create_entity(group_idx);
        let entity2_idx = main.create_entity(group_idx);
        main.add_component(entity1_idx, 10_u32);
        main.add_component(entity2_idx, 20_u32);
        let data = main.system_data();
        let info = SystemInfo::new(Vec::new(), None);
        let mut guard = Param::lock(&data);

        assert_panics!(Param::get(&info, &mut guard));
    }
}

#[cfg(test)]
mod component_option_system_param_tests {
    use super::*;
    use crate::internal::main::MainFacade;
    use crate::SystemInfo;

    type Param<'a> = Option<&'a u32>;

    #[test]
    fn retrieve_component_types() {
        let component_types = Param::component_types();

        assert_eq!(component_types, [TypeAccess::Read(TypeId::of::<u32>())])
    }

    #[test]
    fn retrieve_mandatory_component_types() {
        let component_types = Param::mandatory_component_types();

        assert_eq!(component_types, [])
    }

    #[test]
    fn lock() {
        let mut main = MainFacade::default();
        let group_idx = main.create_group();
        let entity1_idx = main.create_entity(group_idx);
        let entity2_idx = main.create_entity(group_idx);
        main.add_component(entity1_idx, 10_u32);
        main.add_component(entity2_idx, 20_u32);
        let data = main.system_data();

        let guard = Param::lock(&data);

        let components = guard.unwrap();
        assert_option_iter!(components.0.archetype_iter(0), Some(vec![&10, &20]));
    }

    #[test]
    fn retrieve_iter_using_valid_guard() {
        let mut main = MainFacade::default();
        let group_idx = main.create_group();
        let entity1_idx = main.create_entity(group_idx);
        let entity2_idx = main.create_entity(group_idx);
        main.add_component(entity1_idx, 10_u32);
        main.add_component(entity2_idx, 20_u32);
        let data = main.system_data();
        let info = SystemInfo::new(Vec::new(), None);
        let mut guard = Param::lock(&data);
        let archetype = ArchetypeInfo::new(0, group_idx);

        let iter = <Param<'_> as SystemParam>::iter(&data, &info, &mut guard, archetype);

        assert_iter!(iter, [Some(&10), Some(&20)]);
    }

    #[test]
    fn retrieve_iter_using_invalid_guard() {
        let mut main = MainFacade::default();
        let group_idx = main.create_group();
        let entity1_idx = main.create_entity(group_idx);
        let entity2_idx = main.create_entity(group_idx);
        main.add_component(entity1_idx, 10_u32);
        main.add_component(entity2_idx, 20_u32);
        let data = main.system_data();
        let info = SystemInfo::new(Vec::new(), None);
        let mut guard = None;
        let archetype = ArchetypeInfo::new(0, group_idx);

        let mut iter = <Param<'_> as SystemParam>::iter(&data, &info, &mut guard, archetype);

        assert_eq!(iter.next(), Some(None));
        assert_eq!(iter.next(), Some(None));
    }

    #[test]
    fn retrieve_iter_using_wrong_archetype_idx() {
        let mut main = MainFacade::default();
        let group_idx = main.create_group();
        let entity1_idx = main.create_entity(group_idx);
        let entity2_idx = main.create_entity(group_idx);
        main.add_component(entity1_idx, 10_u32);
        main.add_component(entity2_idx, 20_u32);
        let data = main.system_data();
        let info = SystemInfo::new(Vec::new(), None);
        let mut guard = Param::lock(&data);
        let archetype = ArchetypeInfo::new(1, group_idx);

        let mut iter = <Param<'_> as SystemParam>::iter(&data, &info, &mut guard, archetype);

        assert_eq!(iter.next(), None);
    }

    #[test]
    fn get_single_value() {
        let mut main = MainFacade::default();
        let group_idx = main.create_group();
        let entity1_idx = main.create_entity(group_idx);
        let entity2_idx = main.create_entity(group_idx);
        main.add_component(entity1_idx, 10_u32);
        main.add_component(entity2_idx, 20_u32);
        let data = main.system_data();
        let info = SystemInfo::new(Vec::new(), None);
        let mut guard = Param::lock(&data);

        assert_panics!(Param::get(&info, &mut guard));
    }
}

#[cfg(test)]
mod component_mut_option_system_param_tests {
    use super::*;
    use crate::internal::main::MainFacade;
    use crate::SystemInfo;

    type Param<'a> = Option<&'a mut u32>;

    #[test]
    fn retrieve_component_types() {
        let component_types = Param::component_types();

        assert_eq!(component_types, [TypeAccess::Write(TypeId::of::<u32>())])
    }

    #[test]
    fn retrieve_mandatory_component_types() {
        let component_types = Param::mandatory_component_types();

        assert_eq!(component_types, [])
    }

    #[test]
    fn lock() {
        let mut main = MainFacade::default();
        let group_idx = main.create_group();
        let entity1_idx = main.create_entity(group_idx);
        let entity2_idx = main.create_entity(group_idx);
        main.add_component(entity1_idx, 10_u32);
        main.add_component(entity2_idx, 20_u32);
        let data = main.system_data();

        let guard = Param::lock(&data);

        let mut components = guard.unwrap();
        let component_iter = components.0.archetype_iter_mut(0);
        assert_option_iter!(component_iter, Some(vec![&mut 10, &mut 20]));
    }

    #[test]
    fn retrieve_iter_using_valid_guard() {
        let mut main = MainFacade::default();
        let group_idx = main.create_group();
        let entity1_idx = main.create_entity(group_idx);
        let entity2_idx = main.create_entity(group_idx);
        main.add_component(entity1_idx, 10_u32);
        main.add_component(entity2_idx, 20_u32);
        let data = main.system_data();
        let info = SystemInfo::new(Vec::new(), None);
        let mut guard = Param::lock(&data);
        let archetype = ArchetypeInfo::new(0, group_idx);

        let iter = <Param<'_> as SystemParam>::iter(&data, &info, &mut guard, archetype);

        assert_iter!(iter, [Some(&mut 10), Some(&mut 20)]);
    }

    #[test]
    fn retrieve_iter_using_invalid_guard() {
        let mut main = MainFacade::default();
        let group_idx = main.create_group();
        let entity1_idx = main.create_entity(group_idx);
        let entity2_idx = main.create_entity(group_idx);
        main.add_component(entity1_idx, 10_u32);
        main.add_component(entity2_idx, 20_u32);
        let data = main.system_data();
        let info = SystemInfo::new(Vec::new(), None);
        let mut guard = None;
        let archetype = ArchetypeInfo::new(0, group_idx);

        let mut iter = <Param<'_> as SystemParam>::iter(&data, &info, &mut guard, archetype);

        assert_eq!(iter.next(), Some(None));
        assert_eq!(iter.next(), Some(None));
    }

    #[test]
    fn retrieve_iter_using_wrong_archetype_idx() {
        let mut main = MainFacade::default();
        let group_idx = main.create_group();
        let entity1_idx = main.create_entity(group_idx);
        let entity2_idx = main.create_entity(group_idx);
        main.add_component(entity1_idx, 10_u32);
        main.add_component(entity2_idx, 20_u32);
        let data = main.system_data();
        let info = SystemInfo::new(Vec::new(), None);
        let mut guard = Param::lock(&data);
        let archetype = ArchetypeInfo::new(1, group_idx);

        let mut iter = <Param<'_> as SystemParam>::iter(&data, &info, &mut guard, archetype);

        assert_eq!(iter.next(), None);
    }

    #[test]
    fn get_single_value() {
        let mut main = MainFacade::default();
        let group_idx = main.create_group();
        let entity1_idx = main.create_entity(group_idx);
        let entity2_idx = main.create_entity(group_idx);
        main.add_component(entity1_idx, 10_u32);
        main.add_component(entity2_idx, 20_u32);
        let data = main.system_data();
        let info = SystemInfo::new(Vec::new(), None);
        let mut guard = Param::lock(&data);

        assert_panics!(Param::get(&info, &mut guard));
    }
}

#[cfg(test)]
mod group_system_param_tests {
    use super::*;
    use crate::internal::main::MainFacade;
    use crate::SystemInfo;
    use std::ptr;

    type Param<'a> = Group<'a>;

    #[test]
    fn retrieve_component_types() {
        let component_types = Param::component_types();

        assert_eq!(component_types, [])
    }

    #[test]
    fn retrieve_mandatory_component_types() {
        let component_types = Param::mandatory_component_types();

        assert_eq!(component_types, [])
    }

    #[test]
    fn lock() {
        let mut main = MainFacade::default();
        let group_idx = main.create_group();
        let entity1_idx = main.create_entity(group_idx);
        let entity2_idx = main.create_entity(group_idx);
        main.add_component(entity1_idx, 10_u32);
        main.add_component(entity2_idx, 20_u32);
        let data = main.system_data();

        let guard = Param::lock(&data);

        assert!(ptr::eq(guard, &data));
    }

    #[test]
    fn retrieve_iter() {
        let mut main = MainFacade::default();
        let group_idx = main.create_group();
        let entity1_idx = main.create_entity(group_idx);
        let entity2_idx = main.create_entity(group_idx);
        main.add_component(entity1_idx, 10_u32);
        main.add_component(entity2_idx, 20_u32);
        let data = main.system_data();
        let info = SystemInfo::new(Vec::new(), None);
        let mut guard = Param::lock(&data);
        let archetype = ArchetypeInfo::new(0, group_idx);

        let mut iter = <Param<'_> as SystemParam>::iter(&data, &info, &mut guard, archetype);

        iter.next().unwrap().delete();
        main.apply_system_actions();
        let data = main.system_data();
        assert_eq!(data.entity_idxs(0), []);
    }

    #[test]
    fn get_single_value() {
        let mut main = MainFacade::default();
        let group_idx = main.create_group();
        let entity1_idx = main.create_entity(group_idx);
        let entity2_idx = main.create_entity(group_idx);
        main.add_component(entity1_idx, 10_u32);
        main.add_component(entity2_idx, 20_u32);
        let data = main.system_data();
        let info = SystemInfo::new(Vec::new(), None);
        let mut guard = Param::lock(&data);

        assert_panics!(Param::get(&info, &mut guard));
    }
}

#[cfg(test)]
mod entity_system_param_tests {
    use super::*;
    use crate::internal::main::MainFacade;
    use crate::SystemInfo;
    use std::ptr;

    type Param<'a> = Entity<'a>;

    #[test]
    fn retrieve_component_types() {
        let component_types = Param::component_types();

        assert_eq!(component_types, [])
    }

    #[test]
    fn retrieve_mandatory_component_types() {
        let component_types = Param::mandatory_component_types();

        assert_eq!(component_types, [])
    }

    #[test]
    fn lock() {
        let mut main = MainFacade::default();
        let group_idx = main.create_group();
        let entity1_idx = main.create_entity(group_idx);
        let entity2_idx = main.create_entity(group_idx);
        main.add_component(entity1_idx, 10_u32);
        main.add_component(entity2_idx, 20_u32);
        let data = main.system_data();

        let guard = Param::lock(&data);

        assert!(ptr::eq(guard, &data));
    }

    #[test]
    fn retrieve_iter() {
        let mut main = MainFacade::default();
        let group_idx = main.create_group();
        let entity1_idx = main.create_entity(group_idx);
        let entity2_idx = main.create_entity(group_idx);
        let entity3_idx = main.create_entity(group_idx);
        main.add_component(entity1_idx, 10_u32);
        main.add_component(entity2_idx, 20_u32);
        main.add_component(entity3_idx, 30_u32);
        let data = main.system_data();
        let info = SystemInfo::new(Vec::new(), None);
        let mut guard = Param::lock(&data);
        let archetype = ArchetypeInfo::new(0, group_idx);

        let mut iter = <Param<'_> as SystemParam>::iter(&data, &info, &mut guard, archetype);

        iter.next().unwrap().delete();
        assert!(iter.next().is_some());
        iter.next().unwrap().delete();
        main.apply_system_actions();
        let data = main.system_data();
        assert_eq!(data.entity_idxs(0), [1]);
    }

    #[test]
    fn get_single_value() {
        let mut main = MainFacade::default();
        let group_idx = main.create_group();
        let entity1_idx = main.create_entity(group_idx);
        let entity2_idx = main.create_entity(group_idx);
        main.add_component(entity1_idx, 10_u32);
        main.add_component(entity2_idx, 20_u32);
        let data = main.system_data();
        let info = SystemInfo::new(Vec::new(), None);
        let mut guard = Param::lock(&data);

        assert_panics!(Param::get(&info, &mut guard));
    }
}

#[cfg(test)]
mod query_system_param_tests {
    use super::*;
    use crate::internal::main::MainFacade;
    use crate::SystemInfo;
    use std::ptr;

    type Param<'a> = Query<'a, (&'a u32, Option<&'a mut i64>)>;

    #[test]
    fn retrieve_component_types() {
        let component_types = Param::component_types();

        let expected_types = [
            TypeAccess::Read(TypeId::of::<u32>()),
            TypeAccess::Write(TypeId::of::<i64>()),
        ];
        assert_eq!(component_types, expected_types)
    }

    #[test]
    fn retrieve_mandatory_component_types() {
        let component_types = Param::mandatory_component_types();

        assert_eq!(component_types, [])
    }

    #[test]
    fn lock() {
        let mut main = MainFacade::default();
        let group_idx = main.create_group();
        let entity1_idx = main.create_entity(group_idx);
        let entity2_idx = main.create_entity(group_idx);
        main.add_component(entity1_idx, 10_u32);
        main.add_component(entity2_idx, 20_u32);
        let data = main.system_data();

        let guard = Param::lock(&data);

        assert!(ptr::eq(guard, &data));
    }

    #[test]
    fn retrieve_iter() {
        let mut main = MainFacade::default();
        let group_idx = main.create_group();
        let entity1_idx = main.create_entity(group_idx);
        let entity2_idx = main.create_entity(group_idx);
        main.add_component(entity1_idx, 10_u32);
        main.add_component(entity2_idx, 20_u32);
        let data = main.system_data();
        let info = SystemInfo::new(vec![TypeId::of::<i64>()], Some(group_idx));
        let mut guard = Param::lock(&data);
        let archetype = ArchetypeInfo::new(0, group_idx);

        let mut iter = <Param<'_> as SystemParam>::iter(&data, &info, &mut guard, archetype);

        let mut query = iter.next().unwrap();
        let query_run = query.run_mut(|_: &u32, _: Option<&mut i64>| ());
        assert_eq!(query_run.group_idx, Some(group_idx));
        assert_eq!(query_run.filtered_component_types, []);
    }

    #[test]
    fn get_single_value() {
        let mut main = MainFacade::default();
        let group_idx = main.create_group();
        let entity1_idx = main.create_entity(group_idx);
        let entity2_idx = main.create_entity(group_idx);
        main.add_component(entity1_idx, 10_u32);
        main.add_component(entity2_idx, 20_u32);
        let data = main.system_data();
        let info = SystemInfo::new(vec![TypeId::of::<i64>()], Some(group_idx));
        let mut guard = Param::lock(&data);

        let mut query = Param::get(&info, &mut guard);

        let query_run = query.run_mut(|_: &u32, _: Option<&mut i64>| ());
        assert_eq!(query_run.group_idx, Some(group_idx));
        assert_eq!(query_run.filtered_component_types, []);
    }
}

#[cfg(test)]
mod tuple_with_many_items_system_param_tests {
    use super::*;
    use crate::internal::main::MainFacade;
    use crate::SystemInfo;

    type Param<'a> = (&'a u32, Option<&'a mut i64>);

    #[test]
    fn retrieve_component_types() {
        let component_types = Param::component_types();

        let expected_types = [
            TypeAccess::Read(TypeId::of::<u32>()),
            TypeAccess::Write(TypeId::of::<i64>()),
        ];
        assert_eq!(component_types, expected_types)
    }

    #[test]
    fn retrieve_mandatory_component_types() {
        let component_types = Param::mandatory_component_types();

        assert_eq!(component_types, [TypeId::of::<u32>()])
    }

    #[test]
    fn lock() {
        let mut main = MainFacade::default();
        let group_idx = main.create_group();
        let entity1_idx = main.create_entity(group_idx);
        let entity2_idx = main.create_entity(group_idx);
        main.add_component(entity1_idx, 10_u32);
        main.add_component(entity2_idx, 20_u32);
        main.add_component(entity2_idx, 30_i64);
        let data = main.system_data();

        let (guard1, guard2) = Param::lock(&data);

        let components1 = guard1.unwrap();
        let mut components2 = guard2.unwrap();
        assert_option_iter!(components1.0.archetype_iter(0), Some(vec![&10]));
        assert_option_iter!(components1.0.archetype_iter(1), Some(vec![&20]));
        assert_option_iter!(components2.0.archetype_iter_mut(1), Some(vec![&mut 30]));
    }

    #[test]
    fn retrieve_iter() {
        let mut main = MainFacade::default();
        let group_idx = main.create_group();
        let entity1_idx = main.create_entity(group_idx);
        let entity2_idx = main.create_entity(group_idx);
        main.add_component(entity1_idx, 10_u32);
        main.add_component(entity2_idx, 20_u32);
        main.add_component(entity2_idx, 30_i64);
        let data = main.system_data();
        let info = SystemInfo::new(Vec::new(), None);
        let mut guard = Param::lock(&data);
        let archetype = ArchetypeInfo::new(1, group_idx);

        let iter = <Param<'_> as SystemParam>::iter(&data, &info, &mut guard, archetype);

        assert_iter!(iter, [(&20, Some(&mut 30))]);
    }

    #[test]
    fn get_single_value() {
        let mut main = MainFacade::default();
        let group_idx = main.create_group();
        let entity1_idx = main.create_entity(group_idx);
        let entity2_idx = main.create_entity(group_idx);
        main.add_component(entity1_idx, 10_u32);
        main.add_component(entity2_idx, 20_u32);
        main.add_component(entity2_idx, 30_i64);
        let data = main.system_data();
        let info = SystemInfo::new(Vec::new(), None);
        let mut guard = Param::lock(&data);

        assert_panics!(Param::get(&info, &mut guard));
    }
}

#[cfg(test)]
mod tuple_system_param_tests {
    use super::*;

    assert_impl_all!((): TupleSystemParam);
    assert_impl_all!((&u32, Option<&mut i64>): TupleSystemParam);
    assert_impl_all!((&u32, (&String,), Option<&mut i64>): TupleSystemParam);

    assert_not_impl_any!(&u32: TupleSystemParam);
    assert_not_impl_any!(&mut u32: TupleSystemParam);
    assert_not_impl_any!(Option<& u32>: TupleSystemParam);
    assert_not_impl_any!(Option<&mut u32>: TupleSystemParam);
    assert_not_impl_any!(Group<'_>: TupleSystemParam);
    assert_not_impl_any!(Entity<'_>: TupleSystemParam);
    assert_not_impl_any!(Query<'_, (&u32, Option<&i64>)>: TupleSystemParam);
}

#[cfg(test)]
mod const_system_param_tests {
    use super::*;

    assert_impl_all!((): ConstSystemParam);
    assert_impl_all!(&u32: ConstSystemParam);
    assert_impl_all!(Option<&u32>: ConstSystemParam);
    assert_impl_all!((&u32, Option<&i64>): ConstSystemParam);
    assert_impl_all!(Query<'_, (&u32, Option<&i64>)>: ConstSystemParam);

    assert_not_impl_any!(&mut u32: ConstSystemParam);
    assert_not_impl_any!(Option<&mut u32>: ConstSystemParam);
    assert_not_impl_any!(Group<'_>: ConstSystemParam);
    assert_not_impl_any!(Entity<'_>: ConstSystemParam);
    assert_not_impl_any!((&u32, Option<&mut i64>): ConstSystemParam);
    assert_not_impl_any!(Query<'_, (&u32, Option<&mut i64>)>: ConstSystemParam);
}

#[cfg(test)]
mod multiple_system_param_tests {
    use super::internal::*;
    use super::*;

    assert_impl_all!((): MultipleSystemParams<TupleSystemParams = ()>);
    assert_impl_all!((&u32, Option<&i64>): MultipleSystemParams);
    assert_impl_all!(Query<'_, (&u32, Option<&i64>)>: MultipleSystemParams);

    assert_not_impl_any!(&u32: MultipleSystemParams);
    assert_not_impl_any!(&mut u32: MultipleSystemParams);
    assert_not_impl_any!(Option<& u32>: MultipleSystemParams);
    assert_not_impl_any!(Option<&mut u32>: MultipleSystemParams);
    assert_not_impl_any!(Group<'_>: MultipleSystemParams);
    assert_not_impl_any!(Entity<'_>: MultipleSystemParams);
}

#[cfg(test)]
mod entity_part_system_param_tests {
    use super::internal::*;
    use super::*;

    assert_impl_all!(&u32: EntityPartSystemParam<Resource = u32, Mutability = Const>);
    assert_impl_all!(&mut u32: EntityPartSystemParam<Resource = u32, Mutability = Mut>);
    assert_impl_all!(Option<& u32>: EntityPartSystemParam<Resource = u32, Mutability = Const>);
    assert_impl_all!(Option<&mut u32>: EntityPartSystemParam<Resource = u32, Mutability = Mut>);
    assert_impl_all!(Group<'_>: EntityPartSystemParam<Resource = Group<'static>, Mutability = Mut>);
    assert_impl_all!(Entity<'_>: EntityPartSystemParam<Resource = Entity<'static>, Mutability = Mut>);

    assert_not_impl_any!((): EntityPartSystemParam);
    assert_not_impl_any!((&u32, Option<&i64>): EntityPartSystemParam);
    assert_not_impl_any!(Query<'_, (&u32, Option<&i64>)>: EntityPartSystemParam);
}

#[cfg(test)]
mod not_enough_entity_part_system_param_tests {
    use super::internal::*;
    use super::*;

    assert_impl_all!(Option<& u32>: NotEnoughEntityPartSystemParam);
    assert_impl_all!(Option<&mut u32>: NotEnoughEntityPartSystemParam);
    assert_impl_all!(Group<'_>:NotEnoughEntityPartSystemParam);
    assert_impl_all!(Entity<'_>: NotEnoughEntityPartSystemParam);

    assert_not_impl_any!(&u32: NotEnoughEntityPartSystemParam);
    assert_not_impl_any!(&mut u32: NotEnoughEntityPartSystemParam);
    assert_not_impl_any!((): NotEnoughEntityPartSystemParam);
    assert_not_impl_any!((&u32, Option<&i64>): NotEnoughEntityPartSystemParam);
    assert_not_impl_any!(Query<'_, (&u32, Option<&i64>)>: NotEnoughEntityPartSystemParam);
}
