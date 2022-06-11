use crate::storages::actions::ActionStorage;
use crate::storages::archetypes::{ArchetypeStorage, FilteredArchetypeIdxIter};
use crate::storages::components::{ComponentStorage, ComponentTypeIdx};
use crate::storages::core::CoreStorage;
use crate::storages::entities::EntityStorage;
use crate::storages::systems::SystemProperties;
use crate::storages::updates::UpdateStorage;
use crate::system_params::internal::SystemParamWithLifetime;
use crate::systems::internal::{SealedSystem, SystemWrapper};
use crate::SystemParam;
use std::sync::Mutex;

#[doc(hidden)]
#[macro_export]
macro_rules! system {
    ($system:expr) => {{
        use $crate::{SystemWithParamMutabilityIssue, SystemWithParams};

        #[allow(clippy::semicolon_if_nothing_returned)]
        $crate::SystemBuilder {
            properties_fn: $crate::System::properties_fn(&$system),
            wrapper: |data: $crate::SystemData<'_>, info: $crate::SystemInfo<'_>| {
                let checker = $crate::SystemParamMutabilityChecker::new($system);
                let mut system = checker.check_param_mutability().into_inner();
                let mut guard = $crate::System::lock(&system, data, info);
                let mut guard_borrow = $crate::System::borrow_guard(&system, &mut guard);
                let mut stream = $crate::System::stream(&system, &mut guard_borrow);
                while let Some(item) = $crate::System::stream_next(&system, &mut stream) {
                    $crate::System::apply(&mut system, item);
                }
            },
        }
    }};
}

#[doc(hidden)]
#[derive(Clone, Copy)]
pub struct SystemInfo<'a> {
    pub(crate) filtered_component_type_idxs: &'a [ComponentTypeIdx],
    pub(crate) item_count: usize,
}

#[doc(hidden)]
#[derive(Clone, Copy)]
pub struct SystemData<'a> {
    pub(crate) entities: &'a EntityStorage,
    pub(crate) components: &'a ComponentStorage,
    pub(crate) archetypes: &'a ArchetypeStorage,
    pub(crate) actions: &'a ActionStorage,
    pub(crate) updates: &'a Mutex<UpdateStorage>,
}

impl SystemData<'_> {
    pub(crate) fn filter_archetype_idx_iter<'a>(
        &'a self,
        filtered_component_type_idxs: &'a [ComponentTypeIdx],
    ) -> FilteredArchetypeIdxIter<'a> {
        if let Some((&first_idx, other_idxs)) = filtered_component_type_idxs.split_first() {
            self.archetypes.filter_idxs(
                self.components.sorted_archetype_idxs(first_idx).iter(),
                other_idxs,
            )
        } else {
            self.archetypes
                .filter_idxs(self.archetypes.all_sorted_idxs().iter(), &[])
        }
    }

    pub(crate) fn item_count(&self, filtered_component_type_idxs: &[ComponentTypeIdx]) -> usize {
        self.filter_archetype_idx_iter(filtered_component_type_idxs)
            .map(|a| self.archetypes.entity_idxs(a).len())
            .sum()
    }
}

#[doc(hidden)]
pub struct SystemBuilder {
    #[doc(hidden)]
    pub properties_fn: fn(&mut CoreStorage) -> SystemProperties,
    #[doc(hidden)]
    pub wrapper: SystemWrapper,
}

/// A trait implemented for any system.
pub trait System<P>: SealedSystem<P>
where
    P: SystemParam,
{
    #[doc(hidden)]
    fn properties_fn(&self) -> fn(&mut CoreStorage) -> SystemProperties {
        P::properties
    }

    #[doc(hidden)]
    fn lock<'a>(
        &self,
        data: SystemData<'a>,
        info: SystemInfo<'a>,
    ) -> <P as SystemParamWithLifetime<'a>>::Guard {
        P::lock(data, info)
    }

    #[doc(hidden)]
    fn borrow_guard<'a, 'b>(
        &self,
        guard: &'a mut <P as SystemParamWithLifetime<'b>>::Guard,
    ) -> <P as SystemParamWithLifetime<'a>>::GuardBorrow
    where
        'b: 'a,
    {
        P::borrow_guard(guard)
    }

    #[doc(hidden)]
    fn stream<'a, 'b>(
        &self,
        guard: &'a mut <P as SystemParamWithLifetime<'b>>::GuardBorrow,
    ) -> <P as SystemParamWithLifetime<'a>>::Stream
    where
        'b: 'a,
    {
        P::stream(guard)
    }

    #[doc(hidden)]
    #[inline]
    fn stream_next<'a, 'b>(
        &self,
        stream: &'a mut <P as SystemParamWithLifetime<'b>>::Stream,
    ) -> Option<<P as SystemParamWithLifetime<'a>>::Param>
    where
        'b: 'a,
    {
        P::stream_next(stream)
    }

    #[doc(hidden)]
    fn apply(&mut self, item: P);
}

macro_rules! impl_system {
    ($(($params:ident, $indexes:tt)),*) => {
        impl<$($params,)* S> SealedSystem<($($params,)*)> for S
        where
            S: FnMut($($params),*),
            $($params: SystemParam,)*
        {
        }

        impl<$($params,)* S> System<($($params,)*)> for S
        where
            S: FnMut($($params),*),
            $($params: SystemParam,)*
        {
            #[allow(unused_variables)]
            #[inline]
            fn apply(&mut self, item: ($($params,)*)) {
                self($(item.$indexes),*);
            }
        }
    };
}

impl_system!();
run_for_tuples_with_idxs!(impl_system);

pub(crate) mod internal {
    use crate::{SystemData, SystemInfo};

    pub trait SealedSystem<P> {}

    pub(crate) type SystemWrapper = fn(SystemData<'_>, SystemInfo<'_>);
}
