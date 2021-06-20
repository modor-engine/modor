use crate::external::systems::building::internal::TypeAccess;
use crate::external::systems::definition::internal::{
    ArchetypeInfo, ComponentsConst, ComponentsMut, SealedSystem,
};
use crate::internal::actions::ActionFacade;
use crate::internal::components::interfaces::{ComponentInterface, Components};
use crate::internal::core::CoreFacade;
use crate::SystemParam;
use std::any::{Any, TypeId};
use std::num::NonZeroUsize;
use std::slice::{Iter, IterMut};
use std::sync::{Mutex, MutexGuard, RwLockReadGuard, RwLockWriteGuard};

// TODO: rename "lock" into "guard" when necessary

/// Characterize a system that is runnable by the application.
///
/// System can be registered and run by the application using the [`system!`](crate::system!) and
/// [`system_once!`](crate::system_once!) macros.
pub trait System<'a, 'b, T>: SealedSystem<T> {
    #[doc(hidden)]
    const HAS_MANDATORY_COMPONENT: bool;
    #[doc(hidden)]
    const HAS_ACTIONS: bool;
    #[doc(hidden)]
    type Locks: 'b;

    #[doc(hidden)]
    fn has_mandatory_component(&self) -> bool {
        Self::HAS_MANDATORY_COMPONENT
    }

    #[doc(hidden)]
    fn has_actions(&self) -> bool {
        Self::HAS_ACTIONS
    }

    #[doc(hidden)]
    fn component_types(&self) -> Vec<TypeAccess>;

    #[doc(hidden)]
    fn lock(&self, data: &'b SystemData<'_>) -> Self::Locks;

    #[doc(hidden)]
    fn archetypes(&self, data: &SystemData<'_>, info: &SystemInfo) -> Vec<ArchetypeInfo>;

    #[doc(hidden)]
    fn run_once(&mut self, info: &SystemInfo, locks: &'a mut Self::Locks);

    #[doc(hidden)]
    fn run(
        &mut self,
        data: &'b SystemData<'_>,
        info: &SystemInfo,
        locks: &'a mut Self::Locks,
        archetype: ArchetypeInfo,
    );
}

impl<S> SealedSystem<()> for S where S: FnMut() {}

// TODO: include in below macro definition
impl<'a, 'b, S> System<'a, 'b, ()> for S
where
    S: FnMut(),
{
    const HAS_MANDATORY_COMPONENT: bool = false;
    const HAS_ACTIONS: bool = false;
    type Locks = ();

    fn component_types(&self) -> Vec<TypeAccess> {
        Vec::new()
    }

    fn lock(&self, _data: &'b SystemData<'_>) -> Self::Locks {}

    fn archetypes(&self, _data: &SystemData<'_>, _info: &SystemInfo) -> Vec<ArchetypeInfo> {
        Vec::new()
    }

    fn run_once(&mut self, _info: &SystemInfo, _locks: &'a mut Self::Locks) {
        self();
    }

    fn run(
        &mut self,
        _data: &'b SystemData<'_>,
        _info: &SystemInfo,
        _locks: &'a mut Self::Locks,
        _archetype: ArchetypeInfo,
    ) {
        self();
    }
}

macro_rules! impl_fn_system {
    ($(($params:ident, $indexes:tt)),+) => {
        impl<'a, 'b: 'a, S, $($params),+> SealedSystem<($($params,)+)> for S
        where
            S: FnMut($($params),+),
            $($params: SystemParam<'a, 'b>,)+
        {
        }

        impl<'a, 'b: 'a, S, $($params),+> System<'a, 'b, ($($params,)+)> for S
        where
            S: FnMut($($params),+),
            $($params: SystemParam<'a, 'b>,)+
        {
            const HAS_MANDATORY_COMPONENT: bool = $($params::HAS_MANDATORY_COMPONENT)||+;
            const HAS_ACTIONS: bool = $($params::HAS_ACTIONS)||+;
            type Locks = ($($params::Lock,)+);

            fn component_types(&self) -> Vec<TypeAccess> {
                let mut types = Vec::new();
                $(types.extend($params::component_types().into_iter());)+
                types
            }

            fn lock(&self, data: &'b SystemData<'_>) -> Self::Locks {
                ($($params::lock(data),)+)
            }

            fn archetypes(&self, data: &SystemData<'_>, info: &SystemInfo) -> Vec<ArchetypeInfo> {
                let mut mandatory_component_types = info.filtered_component_types.to_vec();
                $(mandatory_component_types.extend(
                    $params::mandatory_component_types().into_iter()
                );)+
                data.archetypes(&mandatory_component_types, info.group_idx)
            }

            fn run_once(&mut self, info: &SystemInfo, locks: &'a mut Self::Locks) {
                self($($params::get(info, &mut locks.$indexes)),+)
            }

            #[allow(non_snake_case, unused_parens)]
            fn run(
                &mut self,
                data: &'b SystemData<'_>,
                info: &SystemInfo,
                locks: &'a mut Self::Locks,
                archetype: ArchetypeInfo,
            ) {
                itertools::izip!($($params::iter(data, info, &mut locks.$indexes, archetype)),+)
                    .for_each(|($($params),+)| self($($params),+));
            }
        }
    };
}

run_for_tuples_with_idxs!(impl_fn_system);

#[doc(hidden)]
pub struct SystemInfo {
    pub(crate) filtered_component_types: Vec<TypeId>,
    pub(crate) group_idx: Option<NonZeroUsize>,
}

impl SystemInfo {
    #[doc(hidden)]
    pub fn new(filtered_component_types: Vec<TypeId>, group_idx: Option<NonZeroUsize>) -> Self {
        Self {
            filtered_component_types,
            group_idx,
        }
    }
}

#[doc(hidden)]
#[derive(Clone)]
pub struct SystemData<'a> {
    core: &'a CoreFacade,
    components: &'a ComponentInterface<'a>,
    actions: &'a Mutex<ActionFacade>,
}

impl<'a> SystemData<'a> {
    pub(crate) fn new(
        core: &'a CoreFacade,
        components: &'a ComponentInterface<'a>,
        actions: &'a Mutex<ActionFacade>,
    ) -> Self {
        Self {
            core,
            components,
            actions,
        }
    }

    pub(crate) fn entity_idxs(&self, archetype_idx: usize) -> &[usize] {
        self.core.archetype_entity_idxs(archetype_idx)
    }

    pub(crate) fn read_components<C>(&self) -> Option<ComponentsConst<'_>>
    where
        C: Any,
    {
        self.components.read::<C>().map(ComponentsConst)
    }

    pub(crate) fn write_components<C>(&self) -> Option<ComponentsMut<'_>>
    where
        C: Any,
    {
        self.components.write::<C>().map(ComponentsMut)
    }

    pub(crate) fn component_iter<C>(
        &self,
        guard: &'a RwLockReadGuard<'_, Components>,
        archetype_idx: usize,
    ) -> Option<Iter<'a, C>>
    where
        C: Any,
    {
        self.components.iter::<C>(guard, archetype_idx)
    }

    pub(crate) fn component_iter_mut<C>(
        &self,
        guard: &'a mut RwLockWriteGuard<'_, Components>,
        archetype_idx: usize,
    ) -> Option<IterMut<'a, C>>
    where
        C: Any,
    {
        self.components.iter_mut::<C>(guard, archetype_idx)
    }

    pub(crate) fn actions_mut(&self) -> MutexGuard<'_, ActionFacade> {
        self.actions
            .try_lock()
            .expect("internal error: lock already locked actions")
    }

    fn archetypes(
        &self,
        component_types: &[TypeId],
        group_idx: Option<NonZeroUsize>,
    ) -> Vec<ArchetypeInfo> {
        self.core.archetypes(component_types, group_idx)
    }
}

pub(crate) mod internal {
    use crate::internal::components::interfaces::Components;
    use std::num::NonZeroUsize;
    use std::sync::{RwLockReadGuard, RwLockWriteGuard};

    pub trait SealedSystem<T> {}

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub struct ArchetypeInfo {
        pub(crate) idx: usize,
        pub(crate) group_idx: NonZeroUsize,
    }

    impl ArchetypeInfo {
        pub(crate) fn new(idx: usize, group_idx: NonZeroUsize) -> Self {
            Self { idx, group_idx }
        }
    }

    pub struct ComponentsConst<'a>(pub(crate) RwLockReadGuard<'a, Components>);

    pub struct ComponentsMut<'a>(pub(crate) RwLockWriteGuard<'a, Components>);
}

#[cfg(test)]
mod system_tests {
    use super::*;
    use crate::internal::main::MainFacade;
    use crate::{Group, Query, SystemInfo, SystemOnceBuilder};

    #[test]
    fn retrieve_whether_system_with_no_param_has_mandatory_component() {
        let system = || ();

        let has_mandatory_component = System::has_mandatory_component(&system);

        assert!(!has_mandatory_component);
    }

    #[test]
    fn retrieve_whether_system_with_mandatory_component_param_has_mandatory_component() {
        let system = |_: &u32| ();

        let has_mandatory_component = System::has_mandatory_component(&system);

        assert!(has_mandatory_component);
    }

    #[test]
    fn retrieve_whether_system_with_optional_component_param_has_mandatory_component() {
        let system = |_: Option<&u32>| ();

        let has_mandatory_component = System::has_mandatory_component(&system);

        assert!(!has_mandatory_component);
    }

    #[test]
    fn retrieve_whether_system_with_mandatory_and_not_component_params_has_mandatory_component() {
        let system = |_: Option<&u32>, _: &u32| ();

        let has_mandatory_component = System::has_mandatory_component(&system);

        assert!(has_mandatory_component);
    }

    #[test]
    fn retrieve_whether_system_with_no_param_has_actions() {
        let system = || ();

        let has_actions = System::has_actions(&system);

        assert!(!has_actions);
    }

    #[test]
    fn retrieve_whether_system_with_action_param_has_actions() {
        let system = |_: Group<'_>| ();

        let has_actions = System::has_actions(&system);

        assert!(has_actions);
    }

    #[test]
    fn retrieve_whether_system_with_not_action_param_has_actions() {
        let system = |_: &u32| ();

        let has_actions = System::has_actions(&system);

        assert!(!has_actions);
    }

    #[test]
    fn retrieve_whether_system_with_action_and_not_action_params_has_actions() {
        let system = |_: &u32, _: Group<'_>| ();

        let has_actions = System::has_actions(&system);

        assert!(has_actions);
    }

    #[test]
    fn retrieve_component_types_of_system_with_no_param() {
        let system = || ();

        let component_types = System::component_types(&system);

        assert_eq!(component_types, []);
    }

    #[test]
    fn retrieve_component_types_of_system_with_component_params() {
        let system = |_: Option<&mut u32>, _: &i64| ();

        let component_types = System::component_types(&system);

        let param1_type = TypeAccess::Write(TypeId::of::<u32>());
        let param2_type = TypeAccess::Read(TypeId::of::<i64>());
        assert_eq!(component_types, [param1_type, param2_type]);
    }

    #[test]
    fn retrieve_component_types_of_system_with_component_and_not_component_params() {
        let system = |_: Group<'_>, _: &i64| ();

        let component_types = System::component_types(&system);

        assert_eq!(component_types, [TypeAccess::Read(TypeId::of::<i64>())]);
    }

    #[test]
    fn lock_resources_for_system_with_no_param() {
        let mut main = MainFacade::default();
        main.run_system_once(SystemOnceBuilder::new(|data, _| {
            let system = || ();

            assert!(matches!(System::lock(&system, data), ()));
        }));
    }

    #[test]
    fn lock_resources_for_system_with_params() {
        let mut main = MainFacade::default();
        let group_idx = main.create_group();
        let entity1_idx = main.create_entity(group_idx);
        let entity2_idx = main.create_entity(group_idx);
        main.add_component(entity1_idx, 10_u32);
        main.add_component(entity2_idx, 20_u32);
        main.run_system_once(SystemOnceBuilder::new(|data, _| {
            let system = |_: &u32, _: Group<'_>| ();

            let (lock1, lock2) = System::lock(&system, data);

            assert!(lock1.is_some());
            assert_option_iter!(lock1.unwrap().0.iter::<u32>(0), Some(vec![&10, &20]));
            assert!(std::ptr::eq(lock2, data));
        }));
    }

    #[test]
    fn retrieve_archetypes_for_system_with_no_param() {
        let mut main = MainFacade::default();
        let group_idx = main.create_group();
        let entity_idx = main.create_entity(group_idx);
        main.add_component(entity_idx, 10_u32);
        main.run_system_once(SystemOnceBuilder::new(|data, _| {
            let system = || ();
            let info = SystemInfo::new(vec![TypeId::of::<u32>()], Some(group_idx));

            let archetypes = System::archetypes(&system, data, &info);

            assert_eq!(archetypes, []);
        }));
    }

    #[test]
    fn retrieve_archetypes_for_system_with_params() {
        let mut main = MainFacade::default();
        let group_idx = main.create_group();
        let entity1_idx = main.create_entity(group_idx);
        let entity2_idx = main.create_entity(group_idx);
        main.add_component(entity1_idx, 10_u32);
        main.add_component(entity2_idx, 20_u32);
        main.add_component(entity2_idx, 30_i64);
        main.run_system_once(SystemOnceBuilder::new(|data, _| {
            let system = |_: &u32, _: Group<'_>| ();
            let info = SystemInfo::new(vec![TypeId::of::<i64>()], Some(group_idx));

            let archetypes = System::archetypes(&system, data, &info);

            assert_eq!(archetypes, [ArchetypeInfo::new(1, group_idx)]);
        }));
    }

    #[test]
    fn run_once_system_with_no_param() {
        let mut main = MainFacade::default();
        main.run_system_once(SystemOnceBuilder::new(|_data, _| {
            let mut count = 0;
            let mut system = || count += 1;
            let info = SystemInfo::new(vec![TypeId::of::<i64>()], None);

            System::run_once(&mut system, &info, &mut ());

            assert_eq!(count, 1);
        }));
    }

    #[test]
    fn run_once_system_with_params() {
        let mut main = MainFacade::default();
        main.run_system_once(SystemOnceBuilder::new(|data, _| {
            let mut count = 0;
            let mut system = |_: Query<'_, (&u32,)>| count += 1;
            let info = SystemInfo::new(vec![TypeId::of::<i64>()], None);
            let mut locks = System::lock(&system, data);

            System::run_once(&mut system, &info, &mut locks);

            assert_eq!(count, 1);
        }));
    }

    #[test]
    fn run_system_with_no_param() {
        let mut main = MainFacade::default();
        let group_idx = main.create_group();
        main.run_system_once(SystemOnceBuilder::new(|data, _| {
            let mut count = 0;
            let mut system = || count += 1;
            let info = SystemInfo::new(vec![TypeId::of::<i64>()], None);
            let archetype = ArchetypeInfo::new(0, group_idx);

            System::run(&mut system, data, &info, &mut (), archetype);

            assert_eq!(count, 1);
        }));
    }

    #[test]
    fn run_system_with_params() {
        let mut main = MainFacade::default();
        let group_idx = main.create_group();
        let entity1_idx = main.create_entity(group_idx);
        let entity2_idx = main.create_entity(group_idx);
        main.add_component(entity1_idx, 10_u32);
        main.add_component(entity2_idx, 20_u32);
        main.add_component(entity2_idx, 30_i64);
        main.run_system_once(SystemOnceBuilder::new(|data, _| {
            let mut components = Vec::new();
            let mut system = |c: &u32| components.push(*c);
            let info = SystemInfo::new(vec![TypeId::of::<i64>()], None);
            let mut locks = System::lock(&system, data);
            let archetype = ArchetypeInfo::new(1, group_idx);

            System::run(&mut system, data, &info, &mut locks, archetype);

            assert_eq!(components, [20]);
        }));
    }
}

#[cfg(test)]
mod system_info_tests {
    use super::*;

    assert_impl_all!(SystemInfo: Sync, Send);
    assert_not_impl_any!(SystemInfo: Clone);
}

#[cfg(test)]
mod system_data_tests {
    use super::*;
    use crate::internal::main::MainFacade;
    use crate::SystemOnceBuilder;

    assert_impl_all!(SystemData<'_>: Sync, Send, Clone);

    #[test]
    fn retrieve_entity_idxs() {
        let mut main = MainFacade::default();
        let group_idx = main.create_group();
        let entity1_idx = main.create_entity(group_idx);
        let entity2_idx = main.create_entity(group_idx);
        main.add_component(entity1_idx, 10_u32);
        main.add_component(entity2_idx, 20_u32);
        main.run_system_once(SystemOnceBuilder::new(|data, _| {
            let entity_idxs = data.entity_idxs(0);

            assert_eq!(entity_idxs, [0, 1]);
        }));
    }

    #[test]
    fn read_components() {
        let mut main = MainFacade::default();
        let group_idx = main.create_group();
        let entity1_idx = main.create_entity(group_idx);
        let entity2_idx = main.create_entity(group_idx);
        main.add_component(entity1_idx, 10_u32);
        main.add_component(entity2_idx, 20_u32);
        main.run_system_once(SystemOnceBuilder::new(|data, _| {
            let components = data.read_components::<u32>();

            assert!(components.is_some());
            assert_option_iter!(components.unwrap().0.iter::<u32>(0), Some(vec![&10, &20]));
        }));
    }

    #[test]
    fn write_components() {
        let mut main = MainFacade::default();
        let group_idx = main.create_group();
        let entity1_idx = main.create_entity(group_idx);
        let entity2_idx = main.create_entity(group_idx);
        main.add_component(entity1_idx, 10_u32);
        main.add_component(entity2_idx, 20_u32);
        main.run_system_once(SystemOnceBuilder::new(|data, _| {
            let components = data.write_components::<u32>();

            assert!(components.is_some());
            assert_option_iter!(components.unwrap().0.iter::<u32>(0), Some(vec![&10, &20]));
        }));
    }

    #[test]
    fn retrieve_component_iter() {
        let mut main = MainFacade::default();
        let group_idx = main.create_group();
        let entity1_idx = main.create_entity(group_idx);
        let entity2_idx = main.create_entity(group_idx);
        main.add_component(entity1_idx, 10_u32);
        main.add_component(entity2_idx, 20_u32);
        main.run_system_once(SystemOnceBuilder::new(|data, _| {
            let components = data.read_components::<u32>().unwrap();

            let component_iter = data.component_iter::<u32>(&components.0, 0);

            assert_option_iter!(component_iter, Some(vec![&10, &20]));
        }));
    }

    #[test]
    fn retrieve_component_iter_mut() {
        let mut main = MainFacade::default();
        let group_idx = main.create_group();
        let entity1_idx = main.create_entity(group_idx);
        let entity2_idx = main.create_entity(group_idx);
        main.add_component(entity1_idx, 10_u32);
        main.add_component(entity2_idx, 20_u32);
        main.run_system_once(SystemOnceBuilder::new(|data, _| {
            let mut components = data.write_components::<u32>().unwrap();

            let component_iter = data.component_iter_mut::<u32>(&mut components.0, 0);

            assert_option_iter!(component_iter, Some(vec![&mut 10, &mut 20]));
        }));
    }

    #[test]
    fn retrieve_actions_mut() {
        let mut main = MainFacade::default();
        let group_idx = main.create_group();
        let entity1_idx = main.create_entity(group_idx);
        let entity2_idx = main.create_entity(group_idx);
        main.add_component(entity1_idx, 10_u32);
        main.add_component(entity2_idx, 20_u32);

        main.run_system_once(SystemOnceBuilder::new(|data, _| {
            data.actions_mut().delete_entity(0);
        }));

        main.apply_system_actions();
        main.run_system_once(SystemOnceBuilder::new(|data, _| {
            let components = data.read_components::<u32>().unwrap();
            let component_iter = data.component_iter::<u32>(&components.0, 0);
            assert_option_iter!(component_iter, Some(vec![&20]));
        }));
    }

    #[test]
    #[should_panic(expected = "internal error: lock already locked actions")]
    fn retrieve_already_locked_actions_mut() {
        let mut main = MainFacade::default();
        let group_idx = main.create_group();
        let entity1_idx = main.create_entity(group_idx);
        let entity2_idx = main.create_entity(group_idx);
        main.add_component(entity1_idx, 10_u32);
        main.add_component(entity2_idx, 20_u32);

        main.run_system_once(SystemOnceBuilder::new(|data, _| {
            let actions = data.actions_mut();
            data.actions_mut().delete_entity(0);
            drop(actions);
        }));
    }

    #[test]
    fn retrieve_archetypes() {
        let mut main = MainFacade::default();
        let group1_idx = main.create_group();
        let group2_idx = main.create_group();
        let entity1_idx = main.create_entity(group1_idx);
        let entity2_idx = main.create_entity(group1_idx);
        let entity3_idx = main.create_entity(group1_idx);
        let entity4_idx = main.create_entity(group2_idx);
        main.add_component(entity1_idx, 10_u32);
        main.add_component(entity2_idx, 20_u32);
        main.add_component(entity2_idx, 20_i64);
        main.add_component(entity3_idx, 30_i64);
        main.add_component(entity4_idx, 40_u32);
        main.run_system_once(SystemOnceBuilder::new(|data, _| {
            let component_types = &[TypeId::of::<u32>()];
            let group_idx_filter = Some(group1_idx);

            let archetypes = data.archetypes(component_types, group_idx_filter);

            let expected_archetypes = [
                ArchetypeInfo::new(0, group1_idx),
                ArchetypeInfo::new(1, group1_idx),
            ];
            assert_eq!(archetypes, expected_archetypes);
        }));
    }
}

#[cfg(test)]
mod archetype_info_tests {
    use super::internal::*;
    use std::fmt::Debug;

    assert_impl_all!(ArchetypeInfo: Sync, Send, Copy, Eq, Debug);
}

#[cfg(test)]
mod components_const_tests {
    use super::internal::*;

    assert_impl_all!(ComponentsConst<'_>: Sync);
    assert_not_impl_any!(ComponentsConst<'_>: Clone);
}

#[cfg(test)]
mod components_mut_tests {
    use super::internal::*;

    assert_impl_all!(ComponentsMut<'_>: Sync);
    assert_not_impl_any!(ComponentsMut<'_>: Clone);
}
