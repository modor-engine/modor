use crate::storages::archetypes::ArchetypeStorage;
use crate::storages::components::ComponentStorage;
use crate::storages::core::SystemProperties;
use crate::storages::entity_actions::EntityActionStorage;
use crate::system_params::internal::{SystemParamIterInfo, SystemParamWithLifetime};
use crate::systems::internal::{SealedSystem, SystemWrapper};
use crate::SystemParam;
use std::any::TypeId;
use std::sync::Mutex;

#[doc(hidden)]
pub struct SystemInfo {
    pub(crate) filtered_component_types: Vec<TypeId>,
}

#[doc(hidden)]
pub struct SystemData<'a> {
    pub(crate) components: &'a ComponentStorage,
    pub(crate) archetypes: &'a ArchetypeStorage,
    pub(crate) entity_actions: &'a Mutex<EntityActionStorage>,
}

/// A builder for defining a system.
///
/// The [`system!`](crate::system!) macro is used to construct a `SystemBuilder`.
pub struct SystemBuilder {
    pub(crate) properties: SystemProperties,
    pub(crate) wrapper: SystemWrapper,
}

impl SystemBuilder {
    #[doc(hidden)]
    pub fn new(properties: SystemProperties, wrapper: SystemWrapper) -> Self {
        Self {
            properties,
            wrapper,
        }
    }
}

/// A trait implemented for any system that can be passed to the [`system!`](crate::system!) macro.
pub trait System<P>: SealedSystem<P>
where
    P: SystemParam,
{
    #[doc(hidden)]
    fn properties(&self) -> SystemProperties {
        P::properties()
    }

    #[doc(hidden)]
    fn iter_info(&self, data: &SystemData<'_>, info: &SystemInfo) -> SystemParamIterInfo {
        P::iter_info(data, info)
    }

    #[doc(hidden)]
    fn lock<'a>(&self, data: &'a SystemData<'_>) -> <P as SystemParamWithLifetime<'a>>::Guard {
        P::lock(data)
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
        info: &'a SystemParamIterInfo,
    ) -> <P as SystemParamWithLifetime<'a>>::Stream
    where
        'b: 'a,
    {
        P::stream(guard, info)
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

/// Creates a valid instance of [`SystemBuilder`](crate::SystemBuilder).
///
/// The system passed as parameter must be a function or a static closure with no captured
/// variables, and must implement the [`System`](crate::System) trait.
///
/// # System behaviour
///
/// There are two types of system:
/// - iterative system: at least one of the argument types corresponds to an entity part
/// - non-iterative system: none of the argument types correspond to an entity part
///
/// The types that can represent an entity part are:
/// - `&C` where `C` is a component type
/// - `&mut C` where `C` is a component type
/// - `Option<&C>` where `C` is a component type
/// - `Option<&mut C>` where `C` is a component type
/// - [`Entity`](crate::Entity)
/// - a tuple containing at least one of the previous types
///
/// An iterative system is run for each entity containing components of type `C` when
/// `&C` or `&mut C` is the type of an argument. If there is no argument of type `&C` or `&mut C`,
/// then the system iterates on all entities.
///
/// A non-iterative system is only run once per application update.
///
/// # Static checks
///
/// Compile time checks are applied by this macro to ensure the system will not panic at runtime.
/// If the system is invalid, the macro returns a compile time error.
///
/// The [`SystemWithParams`](crate::SystemWithParams) trait is implemented for all systems.
///
/// The [`SystemWithParamMutabilityIssue`](crate::SystemWithParamMutabilityIssue) trait
/// is implemented in case the system is invalid. If this trait is implemented for the system,
/// it creates a compile time error due to a conflict with the implemented
/// [`SystemWithParams`](crate::SystemWithParams) trait.
///
/// # Limitation on the number of parameters
///
/// A system supports up to 10 parameters.<br>
/// If more parameters are needed, tuples can be used to group parameters and count as one.
///
/// # Examples
///
/// Valid systems:
/// ```rust
/// # use modor::{system, Entity, World, Query};
/// #
/// system!(iterative_system);
/// system!(other_iterative_system);
/// system!(non_iterative_system);
/// system!(iterative_system_again);
///
/// fn iterative_system(id: &u32, message: Option<&mut String>) {
///     // run for each entity with at least a component of type `u32`
///     // `String` is not used to filter entities as it is optional
///     if let Some(message) = message {
///         *message = format!("id: {}", id);
///     }
/// }
///
/// fn other_iterative_system(entity: Entity<'_>) {
///     // run for all entities
///     println!("entity detected with ID {}", entity.id());
/// }
///
/// fn non_iterative_system(mut world: World<'_>, query: Query<'_, Entity<'_>>) {
///     // run only once per application update
///     query.iter().for_each(|entity| world.delete_entity(entity.id()));
/// }
///
/// fn iterative_system_again(entity: Entity<'_>, mut world: World<'_>) {
///     // run for all entities because one of the parameters is of type `Entity`
///     // equivalent to the system `non_iterative_system`
///     world.delete_entity(entity.id());
/// }
/// ```
///
/// Invalid systems:
/// ```compile_fail
/// use modor::{system, Entity, World, Query};
///
/// system!(invalid_system);
///
/// fn invalid_system(name: &String, name_mut: &mut String) {
///     // invalid as `String` cannot be borrowed both mutably and immutably
///     *name_mut = format!("[[[ {} ]]]", name);
/// }
/// ```
#[macro_export]
macro_rules! system {
    ($system:expr) => {{
        use ::modor::{SystemWithParamMutabilityIssue, SystemWithParams};

        ::modor::SystemBuilder::new(
            ::modor::System::properties(&$system),
            |data: &::modor::SystemData<'_>, info: ::modor::SystemInfo| {
                let checker = ::modor::SystemParamMutabilityChecker::new($system);
                let mut system = checker.check_param_mutability().into_inner();
                let mut guard = ::modor::System::lock(&system, data);
                let mut guard_borrow = ::modor::System::borrow_guard(&system, &mut guard);
                let iter_info = ::modor::System::iter_info(&system, data, &info);
                let mut stream = ::modor::System::stream(&system, &mut guard_borrow, &iter_info);
                while let Some(item) = ::modor::System::stream_next(&system, &mut stream) {
                    ::modor::System::apply(&mut system, item)
                }
            },
        )
    }};
}

pub(crate) mod internal {
    use crate::{SystemData, SystemInfo};

    pub trait SealedSystem<P> {}

    pub(crate) type SystemWrapper = fn(&SystemData<'_>, SystemInfo);
}

#[cfg(test)]
mod system_info_tests {
    use super::*;
    use std::any::Any;

    impl SystemInfo {
        pub(crate) fn with_one_filtered_type<C>() -> SystemInfo
        where
            C: Any,
        {
            SystemInfo {
                filtered_component_types: vec![TypeId::of::<C>()],
            }
        }
    }
}

#[cfg(test)]
mod system_builder {
    use super::*;
    use std::panic::{RefUnwindSafe, UnwindSafe};

    assert_impl_all!(SystemBuilder: Sync, Send, UnwindSafe, RefUnwindSafe, Unpin);
}

#[cfg(test)]
mod system_tests {
    use super::*;
    use crate::storages::core::CoreStorage;
    use crate::storages::systems::Access;

    #[test]
    fn retrieve_properties() {
        let system = |_: &u32, _: &mut i64| ();

        let properties = System::properties(&system);

        assert_eq!(properties.component_types.len(), 2);
        assert_eq!(properties.component_types[0].access, Access::Read);
        assert_eq!(properties.component_types[1].access, Access::Write);
        assert!(!properties.has_entity_actions);
    }

    #[test]
    fn retrieve_iter_info() {
        let mut core = CoreStorage::default();
        let (_, archetype1_idx) = core.add_component_type::<i64>(ArchetypeStorage::DEFAULT_IDX);
        let (_, archetype2_idx) = core.add_component_type::<u32>(archetype1_idx);
        let info = SystemInfo::with_one_filtered_type::<i64>();
        let system = |_: &u32, _: &mut i64| ();

        let iter_info = System::iter_info(&system, &core.system_data(), &info);

        let expected_iter_info = SystemParamIterInfo::new_intersection(vec![(archetype2_idx, 0)]);
        assert_eq!(iter_info, expected_iter_info);
    }

    #[test]
    fn lock() {
        let mut core = CoreStorage::default();
        let archetype1_idx = ArchetypeStorage::DEFAULT_IDX;
        let (type1_idx, archetype2_idx) = core.add_component_type::<i64>(archetype1_idx);
        let (type2_idx, archetype3_idx) = core.add_component_type::<u32>(archetype2_idx);
        let location = core.create_entity(archetype3_idx);
        core.add_component(10_i64, type1_idx, location);
        core.add_component(20_u32, type2_idx, location);
        let data = core.system_data();
        let system = |_: &u32, _: &mut i64| ();

        let mut guard = System::lock(&system, &data);
        let guard_borrow = System::borrow_guard(&system, &mut guard);

        let expected_guard = ti_vec![ti_vec![], ti_vec![], ti_vec![20_u32]];
        assert_eq!(guard_borrow.0, &expected_guard);
        let expected_guard = ti_vec![ti_vec![], ti_vec![], ti_vec![10_i64]];
        assert_eq!(guard_borrow.1, &expected_guard);
    }

    #[test]
    fn retrieve_stream() {
        let guard1 = ti_vec![ti_vec![10], ti_vec![20]];
        let mut guard2 = ti_vec![ti_vec![30], ti_vec![40]];
        let mut guard_borrow = (&guard1, &mut guard2);
        let iter_info = SystemParamIterInfo::new_intersection(vec![(0.into(), 1), (1.into(), 1)]);
        let system = |_: &u32, _: &mut i64| ();

        let mut stream = System::stream(&system, &mut guard_borrow, &iter_info);

        let next = System::stream_next(&system, &mut stream);
        assert_eq!(next, Some((&10, &mut 30)));
        let next = System::stream_next(&system, &mut stream);
        assert_eq!(next, Some((&20, &mut 40)));
        assert_eq!(System::stream_next(&system, &mut stream), None);
    }

    #[test]
    fn apply_params() {
        let mut collector = vec![];
        let mut system = |a: &u32, b: &mut i64| collector.push((*a, *b));

        System::apply(&mut system, (&10, &mut 20));

        assert_eq!(collector, vec![(10, 20)]);
    }
}
