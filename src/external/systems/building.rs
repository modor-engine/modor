use crate::external::systems::building::internal::TypeAccess;
use crate::{SystemData, SystemInfo};

pub(crate) type SystemWrapper = fn(&SystemData<'_>, SystemInfo);

/// Description of a system to add.
///
/// Instances of this type are created using the [`system!`](crate::system!) macro.
pub struct SystemBuilder {
    pub(crate) wrapper: SystemWrapper,
    pub(crate) component_types: Vec<TypeAccess>,
    pub(crate) actions: bool,
}

impl SystemBuilder {
    #[doc(hidden)]
    pub fn new(wrapper: SystemWrapper, component_types: Vec<TypeAccess>, actions: bool) -> Self {
        Self {
            wrapper,
            component_types,
            actions,
        }
    }
}
/// Description of an entity system to add.
///
/// Instances of this type are created using the [`entity_system!`](crate::entity_system!) macro.
pub struct EntitySystemBuilder {
    pub(crate) wrapper: SystemWrapper,
    pub(crate) component_types: Vec<TypeAccess>,
    pub(crate) actions: bool,
}

impl EntitySystemBuilder {
    #[doc(hidden)]
    pub fn new(wrapper: SystemWrapper, component_types: Vec<TypeAccess>, actions: bool) -> Self {
        Self {
            wrapper,
            component_types,
            actions,
        }
    }
}

/// Description of a system to run once.
///
/// Instances of this type are created using the [`system_once!`](crate::system_once!) macro.
pub struct SystemOnceBuilder<S> {
    pub(crate) wrapper: S,
}

impl<S> SystemOnceBuilder<S>
where
    S: FnMut(&SystemData<'_>, SystemInfo),
{
    #[doc(hidden)]
    pub fn new(wrapper: S) -> Self {
        Self { wrapper }
    }
}

/// Create a valid instance of [`SystemBuilder`](crate::SystemBuilder).
///
/// `system` must be a function or a static closures with no captured variables that
/// implements the [`System`](crate::System) trait.
///
/// # System behaviour
///
/// There are two types of system:
/// - iterative system: at least one of its arguments is a component
/// - non-iterative system: none of its arguments is a component
///
/// The logic describes by an iterative system is run for each entity containing each type of
/// component that appears in system parameters (optional components that are not taken into
/// account).
///
/// The logic describes by a non-iterative system is only run once per application update.
///
/// # Static checks
///
/// Static checks are applied by this macro to ensure the system is valid.
///
/// The [`SystemWithParams`](crate::SystemWithParams) trait is implemented for all systems.<br>
/// In case a system is not well formed, one of these types can also be implemented:
/// - [`SystemWithMissingComponentParam`](crate::SystemWithMissingComponentParam): the system is
///     invalid because some parameters are specific to iterative systems, but a parameter of type
///     component is missing
/// - [`SystemWithQueryWithMissingComponentParam`](crate::SystemWithQueryWithMissingComponentParam):
///     the system is invalid because one of its query parameters has a missing component parameter
/// - [`SystemWithIncompatibleParams`](crate::SystemWithIncompatibleParams): the system has
///     multiple parameters that mutably access to the same resource
///
/// If at least one of these traits is implemented, the macro will return a compilation error
/// indicating a conflicting trait implementation.
///
/// # Examples
///
/// Valid systems:
/// ```rust
/// # use modor::{Application, Query, system, for_each_mut};
/// #
/// Application::new()
///     .on_update(system!(iterative_system))
///     .on_update(system!(non_iterative_system))
///     .update();
///
/// fn iterative_system(id: &u32, string: Option<&mut String>) {
///     // run for each entity with at least a component of type `u32`
///     // `String` type is used optionally, so it does not have an impact on the entity filtering
///     if let Some(string) = string {
///         *string = format!("id: {}", id);
///     }
/// }
///
/// fn non_iterative_system<'a>(mut query: Query<'a, (&'a u32, Option<&'a mut String>)>) {
///     // run only once per application update, as there is no component parameter
///     for_each_mut!(query, |id: &u32, string: Option<&mut String>| {
///         if let Some(string) = string {
///             *string = format!("id: {}", id);
///         }
///     });
/// }
/// ```
///
/// Invalid systems:
/// ```rust
/// # use modor::Group;
/// #
/// // `Group` is only valid in iterative systems
/// fn system_with_missing_component(group: Group<'_>) {}
///
/// // optional components are not enough in iterative systems
/// fn system_also_with_missing_component(optional_component: Option<&String>) {}
///
/// // there are both const and mut references to `u32` component
/// fn system_with_incompatible_params(param1: (&mut u32, &String), param2: &u32) {}
/// ```
#[macro_export]
macro_rules! system {
    ($system:expr) => {{
        let mut types = ::modor::System::component_types(&$system);
        let mut actions = ::modor::System::has_actions(&$system);
        ::modor::SystemBuilder::new(::modor::_system_wrapper!($system), types, actions)
    }};
}

/// Create a valid instance of [`EntitySystemBuilder`](crate::EntitySystemBuilder).
///
/// Expected `system` is the same as the [`system!`](crate:system!) macro, expect it is considered
/// as a valid iterative system even if it has not as parameter a reference to a component type
/// not contained in an `Option`.
///
/// # Examples
///
/// Valid systems:
/// ```rust
/// # use modor::{
/// #     Application, Query, entity_system, for_each_mut, EntityMainComponent, EntityBuilder,
/// #     Built, EntityRunner
/// # };
/// #
/// Application::new()
///     .with_group(|b| {
///         b.with_entity::<Number>(42);
///     })
///     .update();
///
/// struct Number(u32);
///
/// impl EntityMainComponent for Number {
///     type Data = u32;
///
///     fn build(builder: &mut EntityBuilder<'_, Self>, value: Self::Data) -> Built {
///         builder
///             .with(String::from(""))
///             .with_self(Self(value))
///     }
///
///     fn on_update(runner: &mut EntityRunner<'_, Self>) {
///         runner
///             .run(entity_system!(Self::iterative_system))
///             .run(entity_system!(Self::iterative_system_without_mandatory_component_param))
///             .run(entity_system!(Self::non_iterative_system));
///     }
/// }
///
/// impl Number {
///     // run for each entity with at least a component of type `Number`
///     // `String` type is used optionally, so it does not have an impact on the entity filtering
///     fn iterative_system(&self, string: Option<&mut String>) {
///         if let Some(string) = string {
///             *string = format!("number: {}", self.0);
///         }
///     }
///
///     // run for each entity with at least a component of type `Number`
///     // `Number` filtering is implicitly done as this is an iterative entity system
///     fn iterative_system_without_mandatory_component_param(string: Option<&mut String>) {
///         if let Some(string) = string {
///             print!("{}", string);
///         }
///     }
///
///     // run only once per application update, as there is no component parameter
///     fn non_iterative_system<'a>(mut query: Query<'a, (&'a Self, Option<&'a mut String>)>) {
///         for_each_mut!(query, |number: &Self, string: Option<&mut String>| {
///             if let Some(string) = string {
///                 *string = format!("number: {}", number.0);
///             }
///         });
///     }
/// }
/// ```
///
/// Invalid systems:
/// ```rust
/// # use modor::Query;
/// #
/// // there are both const and mut references to `u32` component
/// fn system_with_incompatible_params(param1: (&mut u32, &String), param2: &u32) {}
///
/// // optional components are not enough in iterative systems
/// fn system_with_query_with_missing_mandatory_component_param(
///     query: Query<'_, (Option<&String>,)>
/// ) {
/// }
/// ```
#[macro_export]
macro_rules! entity_system {
    ($system:expr) => {{
        let mut types = ::modor::System::component_types(&$system);
        let mut actions = ::modor::System::has_actions(&$system);
        ::modor::EntitySystemBuilder::new(::modor::_entity_system_wrapper!($system), types, actions)
    }};
}

/// Create a valid instance of [`SystemOnceBuilder`](crate::SystemOnceBuilder).
///
/// `system` is a function or closure that implements the [`System`](crate::System) trait.
///
/// # System behaviour
///
/// There are two types of system:
/// - iterative system: at least one of its arguments is a component
/// - non-iterative system: none of its arguments is a component
///
/// The logic describes by an iterative system is run for each entity containing each type of
/// component that appears in system parameters (optional components that are not taken into
/// account).
///
/// The logic describes by a non-iterative system is only run once.
///
/// # Static checks
///
/// Static checks are applied by this macro to ensure the system is valid.
///
/// The [`SystemWithParams`](crate::SystemWithParams) trait is implemented for all systems.<br>
/// In case a system is not well formed, one of these types can also be implemented:
/// - [`SystemWithMissingComponentParam`](crate::SystemWithMissingComponentParam): the system is
///     invalid because some parameters are specific to iterative systems, but a parameter of type
///     component is missing
/// - [`SystemWithQueryWithMissingComponentParam`](crate::SystemWithQueryWithMissingComponentParam):
///     the system is invalid because one of its query parameters has a missing component parameter
/// - [`SystemWithIncompatibleParams`](crate::SystemWithIncompatibleParams): the system has
///     multiple parameters that mutably access to the same resource
///
/// If at least one of these traits is implemented, the macro will return a compilation error
/// indicating a conflicting trait implementation.
///
/// # Examples
///
/// Valid systems:
/// ```rust
/// # use modor::{Application, Query, system_once, for_each_mut};
/// #
/// let mut application = Application::new();
/// application.run(system_once!(iterative_system));
/// application.run(system_once!(non_iterative_system));
///
/// fn iterative_system(id: &u32, string: Option<&mut String>) {
///     // run for each entity with at least a component of type `u32`
///     // `String` type is used optionally, so it does not have an impact on the entity filtering
///     if let Some(string) = string {
///         *string = format!("id: {}", id);
///     }
/// }
///
/// fn non_iterative_system<'a>(mut query: Query<'a, (&'a u32, Option<&'a mut String>)>) {
///     // run only once per application update, as there is no component parameter
///     for_each_mut!(query, |id: &u32, string: Option<&mut String>| {
///         if let Some(string) = string {
///             *string = format!("id: {}", id);
///         }
///     });
/// }
/// ```
///
/// Invalid systems:
/// ```rust
/// # use modor::Group;
/// #
/// // `Group` is only valid in iterative systems
/// fn system_with_missing_component(group: Group<'_>) {}
///
/// // Optional components are not enough in iterative systems
/// fn system_also_with_missing_component(optional_component: Option<&String>) {}
///
/// // there are both const and mut references to `u32` component
/// fn system_with_incompatible_params(param1: (&mut u32, &String), param2: &u32) {}
/// ```
#[macro_export]
macro_rules! system_once {
    ($system:expr) => {
        ::modor::SystemOnceBuilder::new(::modor::_system_wrapper!($system))
    };
}

/// Run a constant query.
///
/// `query` is an object of type [`Query`](crate::Query) with only immutable parameters.
///
/// `system` is a function or closure that implements the [`System`](crate::System) trait.
/// Its parameters must correspond to the `query` type.
///
/// # Examples
///
/// ```rust
/// # use modor::{Query, for_each};
/// #
/// fn system<'a>(mut query: Query<'a, (&'a u32, Option<&'a String>)>) {
///     for_each!(query, |id: &u32, string: Option<&String>| {
///         if let Some(string) = string {
///             println!("entity with ID {} and string '{}'", id, string);
///         } else {
///             println!("entity with ID {}", id);
///         }
///     });
/// }
/// ```
#[macro_export]
macro_rules! for_each {
    ($query:expr, $system:expr) => {{
        let query: &::modor::Query<_> = &$query;
        let mut system = $system;
        let mut query_run = query.run(system);
        let mut system = query_run.system;
        let filtered_component_types = query_run.filtered_component_types;
        let info = ::modor::SystemInfo::new(filtered_component_types, query_run.group_idx);
        ::modor::_run_system!(&query_run.data, info, system, false);
    }};
}

/// Run a mutable query.
///
/// `query` is an object of type [`Query`](crate::Query).
///
/// `system` is a function or closure that implements the [`System`](crate::System) trait.
/// Its parameters must correspond to the `query` type.
///
/// # Examples
///
/// ```rust
/// # use modor::{Query, for_each_mut};
/// #
/// fn system<'a>(mut query: Query<'a, (&'a u32, Option<&'a mut String>)>) {
///     for_each_mut!(query, |id: &u32, string: Option<&mut String>| {
///         if let Some(string) = string {
///             *string = format!("id: {}", id);
///         }
///     });
/// }
/// ```
#[macro_export]
macro_rules! for_each_mut {
    ($query:expr, $system:expr) => {{
        let query: &mut ::modor::Query<_> = &mut $query;
        let mut system = $system;
        let mut query_run = query.run_mut(system);
        let mut system = query_run.system;
        let filtered_component_types = query_run.filtered_component_types;
        let info = ::modor::SystemInfo::new(filtered_component_types, query_run.group_idx);
        ::modor::_run_system!(&query_run.data, info, system, false);
    }};
}

#[macro_export]
#[doc(hidden)]
macro_rules! _system_wrapper {
    ($system:expr) => {
        |data: &::modor::SystemData<'_>, info: ::modor::SystemInfo| {
            use ::modor::SystemWithParams as _SystemWithParams;
            use ::modor::SystemWithIncompatibleParams as _SystemWithIncompatibleParams;
            use ::modor::SystemWithMissingComponentParam as _SystemWithMissingComponentParam;
            use ::modor::SystemWithQueryWithMissingComponentParam
                as _SystemWithQueryWithMissingComponentParam;
            let mut system = $system;
            system = ::modor::SystemParamCompatibilityChecker::new(system)
                .check_param_compatibility()
                .into_inner();
            system = ::modor::SystemComponentParamChecker::new(system)
                .check_component_params()
                .into_inner();
            system = ::modor::SystemQueryComponentParamChecker::new(system)
                .check_query_component_params()
                .into_inner();
            ::modor::_run_system!(data, info, system, false);
        }
    };
}

#[macro_export]
#[doc(hidden)]
macro_rules! _entity_system_wrapper {
    ($system:expr) => {
        |data: &::modor::SystemData<'_>, info: ::modor::SystemInfo| {
            use ::modor::SystemWithParams as _SystemWithParams;
            use ::modor::SystemWithIncompatibleParams as _SystemWithIncompatibleParams;
            use ::modor::SystemWithQueryWithMissingComponentParam
                as _SystemWithQueryWithMissingComponentParam;
            let mut system = $system;
            system = ::modor::SystemParamCompatibilityChecker::new(system)
                .check_param_compatibility()
                .into_inner();
            system = ::modor::SystemQueryComponentParamChecker::new(system)
                .check_query_component_params()
                .into_inner();
            ::modor::_run_system!(data, info, system, true);
        }
    };
}

#[macro_export]
#[doc(hidden)]
macro_rules! _run_system {
    ($data:expr, $info:expr, $system:expr, $is_entity_system:tt) => {
        let mut data = $data;
        let mut info = $info;
        let mut system = $system;
        let mut guards = ::modor::System::lock(&system, data);
        let has_mandatory_component = ::modor::System::has_mandatory_component(&system);
        let has_entity_part = ::modor::System::has_entity_part(&system);
        if has_entity_part && (has_mandatory_component || $is_entity_system) {
            for archetype in ::modor::System::archetypes(&system, data, &info) {
                ::modor::System::run(&mut system, data, &info, &mut guards, archetype);
            }
        } else {
            ::modor::System::run_once(&mut system, &info, &mut guards);
        }
    };
}

pub(crate) mod internal {
    use std::any::TypeId;

    #[derive(Clone, Copy, PartialEq, Eq, Debug)]
    pub enum TypeAccess {
        Read(TypeId),
        Write(TypeId),
    }
}

#[cfg(test)]
mod system_wrapper_tests {
    use super::*;

    assert_impl_all!(SystemWrapper: Sync, Send, Clone);
}

#[cfg(test)]
mod system_builder_tests {
    use super::*;

    assert_impl_all!(SystemBuilder: Sync, Send);
    assert_not_impl_any!(SystemBuilder: Clone);
}

#[cfg(test)]
mod system_once_builder_tests {
    use super::*;

    assert_impl_all!(SystemOnceBuilder<fn(&u32)>: Sync, Send);
    assert_not_impl_any!(SystemOnceBuilder<fn(&u32)>: Clone);
    assert_not_impl_any!(SystemOnceBuilder<Box<dyn FnMut(&u32)>>: Sync, Send);
}

#[cfg(test)]
mod type_access_tests {
    use super::internal::*;
    use std::fmt::Debug;

    assert_impl_all!(TypeAccess: Sync, Send, Copy, Eq, Debug);
}
