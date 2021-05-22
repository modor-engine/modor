use crate::internal::actions::ActionFacade;
use crate::internal::components::interfaces::{ComponentInterface, Components};
use crate::internal::core::CoreFacade;
use crate::systems::internal::Sealed;
use crate::{SystemParam, TypeAccess};
use std::any::{Any, TypeId};
use std::num::NonZeroUsize;
use std::slice::{Iter, IterMut};
use std::sync::{Mutex, MutexGuard, RwLockReadGuard, RwLockWriteGuard};

pub trait System<'a, 'b, T>: Sealed<T> {
    const HAS_MANDATORY_COMPONENT: bool;
    const HAS_ACTIONS: bool;
    type Locks: 'b;

    fn has_mandatory_component(&self) -> bool {
        Self::HAS_MANDATORY_COMPONENT
    }

    fn has_actions(&self) -> bool {
        Self::HAS_ACTIONS
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

impl<S> Sealed<()> for S where S: FnMut() {}

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
        impl<'a, 'b: 'a, S, $($params),+> Sealed<($($params,)+)> for S
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
    pub(crate) fn new(idx: usize, group_idx: NonZeroUsize) -> Self {
        Self { idx, group_idx }
    }
}

pub struct ComponentsConst<'a>(pub(crate) RwLockReadGuard<'a, Components>);

pub struct ComponentsMut<'a>(pub(crate) RwLockWriteGuard<'a, Components>);

mod internal {
    pub trait Sealed<T> {}
}
