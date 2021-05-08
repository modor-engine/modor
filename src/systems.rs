use crate::internal::components::interfaces::{ComponentInterface, Components};
use crate::internal::core::CoreFacade;
use crate::internal::group_actions::GroupActionFacade;
use crate::{SystemParam, TypeAccess};
use std::any::{Any, TypeId};
use std::num::NonZeroUsize;
use std::slice::{Iter, IterMut};
use std::sync::{Mutex, MutexGuard, RwLockReadGuard, RwLockWriteGuard};

pub trait System<'a, 'b, T> {
    const HAS_MANDATORY_COMPONENT: bool;
    const HAS_GROUP_ACTIONS: bool;
    type Locks: 'b;

    fn has_mandatory_component(&self) -> bool {
        Self::HAS_MANDATORY_COMPONENT
    }

    fn has_group_actions(&self) -> bool {
        Self::HAS_GROUP_ACTIONS
    }

    fn component_types(&self) -> Vec<TypeAccess>;

    fn lock(&self, data: &'b SystemData<'_>) -> Self::Locks;

    fn archetypes(&self, data: &SystemData<'_>, info: &SystemInfo) -> Vec<ArchetypeInfo>;

    fn run_once(&mut self, info: &SystemInfo, locks: &'a mut Self::Locks);

    fn run(
        &mut self,
        data: &'b SystemData<'_>,
        info: &SystemInfo,
        locks: &'a mut Self::Locks,
        archetype: ArchetypeInfo,
    );
}

impl<'a, 'b, SYS> System<'a, 'b, ()> for SYS
where
    SYS: FnMut(),
{
    const HAS_MANDATORY_COMPONENT: bool = false;
    const HAS_GROUP_ACTIONS: bool = false;
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
    ($(($param:ident, $index:tt)),+) => {
        impl<'a, 'b: 'a, SYS, $($param),+> System<'a, 'b, ($($param,)+)> for SYS
        where
            SYS: FnMut($($param),+),
            $($param: SystemParam<'a, 'b>,)+
        {
            const HAS_MANDATORY_COMPONENT: bool = $($param::HAS_MANDATORY_COMPONENT)||+;
            const HAS_GROUP_ACTIONS: bool = $($param::HAS_GROUP_ACTIONS)||+;
            type Locks = ($($param::Lock,)+);

            fn component_types(&self) -> Vec<TypeAccess> {
                let mut types = Vec::new();
                $(types.extend($param::component_types().into_iter());)+
                types
            }

            fn lock(&self, data: &'b SystemData<'_>) -> Self::Locks {
                ($($param::lock(data),)+)
            }

            fn archetypes(&self, data: &SystemData<'_>, info: &SystemInfo) -> Vec<ArchetypeInfo> {
                let mut mandatory_component_types = info.filtered_component_types.to_vec();
                $(mandatory_component_types.extend($param::mandatory_component_types().into_iter());)+
                data.archetypes(&mandatory_component_types, info.group_idx)
            }

            fn run_once(&mut self, info: &SystemInfo, locks: &'a mut Self::Locks) {
                self($($param::get(info, &mut locks.$index)),+)
            }

            #[allow(non_snake_case, unused_parens)]
            fn run(
                &mut self,
                data: &'b SystemData<'_>,
                info: &SystemInfo,
                locks: &'a mut Self::Locks,
                archetype: ArchetypeInfo,
            ) {
                itertools::izip!($($param::iter(data, info, &mut locks.$index, archetype)),+)
                    .for_each(|($($param),+)| self($($param),+));
            }
        }
    };
}

run_for_tuples_with_idxs!(impl_fn_system);

#[derive(Clone)]
pub struct SystemData<'a> {
    core: &'a CoreFacade,
    components: &'a ComponentInterface<'a>,
    group_actions: &'a Mutex<GroupActionFacade>,
}

impl<'a> SystemData<'a> {
    pub(crate) fn new(
        core: &'a CoreFacade,
        components: &'a ComponentInterface<'a>,
        group_actions: &'a Mutex<GroupActionFacade>,
    ) -> Self {
        Self {
            core,
            components,
            group_actions,
        }
    }

    pub(crate) fn archetypes(
        &self,
        component_types: &[TypeId],
        group_idx: Option<NonZeroUsize>,
    ) -> Vec<ArchetypeInfo> {
        self.core.archetypes(component_types, group_idx)
    }

    pub(crate) fn read_components<C>(&self) -> Option<RwLockReadGuard<'_, Components>>
    where
        C: Any,
    {
        self.components.read::<C>()
    }

    pub(crate) fn write_components<C>(&self) -> Option<RwLockWriteGuard<'_, Components>>
    where
        C: Any,
    {
        self.components.write::<C>()
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

    pub(crate) fn group_actions_mut(&self) -> MutexGuard<'_, GroupActionFacade> {
        self.group_actions.try_lock().unwrap()
    }
}

pub struct SystemInfo {
    pub(crate) filtered_component_types: Vec<TypeId>,
    pub(crate) group_idx: Option<NonZeroUsize>,
}

impl SystemInfo {
    pub fn new(filtered_component_types: Vec<TypeId>, group_idx: Option<NonZeroUsize>) -> Self {
        Self {
            filtered_component_types,
            group_idx,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ArchetypeInfo {
    pub(crate) idx: usize,
    pub(crate) group_idx: NonZeroUsize,
}

impl ArchetypeInfo {
    pub fn new(idx: usize, group_idx: NonZeroUsize) -> Self {
        Self { idx, group_idx }
    }
}
