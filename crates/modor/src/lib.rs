//! Modor is a *mo*dular and *d*ata-*or*iented game engine, based on the following principles:
//!
//! - *Modularity*: the
//! [entity component system](https://en.wikipedia.org/wiki/Entity_component_system) pattern makes
//! it very easy to:
//!   - Extend functionalities of the engine in reusable modules.
//!   - Split a project into multiple independent crates.
//!   - Reduce coupling between parts of an application.
//! - *Simplicity*:
//!   - Everything is stored in an entity, even resources, settings or loaded modules.
//!   - Systems are always linked to component types to facilitate system import and limit their
//! side effects.
//!   - The ability to define a component as system dependency makes system ordering easy and
//! maintainable.
//! - *Compile-time checking*: compile-time checks are used extensively to avoid as many errors as
//! possible during runtime:
//!   - System parameters are checked to avoid mutability issues at runtime, e.g. if the same
//! component type is mutably
//! queried twice by the same system.
//!   - System execution order is checked to avoid dependency cycles.
//!   - The API is designed to avoid runtime panics as much as possible.
//!
//! # Examples
//!
//! See [`App`], [`Component`](macro@crate::Component) and [`systems`](macro@crate::systems).
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::multiple_inherent_impl))]

#[macro_use]
extern crate modor_internal;
#[macro_use]
extern crate log;

#[cfg(doctest)]
#[macro_use]
extern crate doc_comment;

#[cfg(doctest)]
doc_comment!(include_str!("../../../README.md"));

#[macro_use]
mod utils;
mod actions;
mod app;
mod components;
mod entities;
mod filters;
mod platform;
mod ranges;
mod storages;
mod system_params;
mod systems;

pub use actions::*;
pub use app::*;
pub use components::runner::*;
pub use components::traits::*;
pub use entities::*;
pub use filters::changed::*;
pub use filters::not::*;
pub use filters::or::*;
pub use filters::with::*;
pub use filters::*;
pub use ranges::*;
pub use system_params::entity::*;
pub use system_params::entity_mut::*;
pub use system_params::filters::*;
pub use system_params::queries::*;
pub use system_params::singletons::*;
pub use system_params::singletons_mut::*;
pub use system_params::world::*;
pub use system_params::*;
pub use systems::building::*;
pub use systems::checks::*;
pub use systems::traits::*;

/// Defines an action type.
///
/// This macro implements the trait [`Action`].
///
/// The type must be a unit type (if no dependency) or a type with unnamed fields, where field types
/// implement [`Action`] trait and are the dependencies of the defined action.
///
/// An action A is a dependency of an action B if all systems running as action A must be run
/// before any system running as action B.
///
/// # Static checks
///
/// The way an action type is defined ensures that cyclic dependencies are detected at compile time.
///
/// # Examples
///
/// ```rust
/// # use modor::*;
/// #
/// #[derive(Action)]
/// struct A;
///
/// #[derive(Action)]
/// pub struct B;
///
/// // systems running as C will be run only once all systems running as A and B have been run
/// #[derive(Action)]
/// pub(crate) struct C(A, B);
/// ```
pub use modor_derive::Action;

/// Defines a component type.
///
/// This macro implements the trait [`Component`].
///
/// It is also required to define systems of the component using one of these macros:
/// - [`systems`](macro@crate::systems) proc macro to define systems.
/// - [`NoSystem`](macro@crate::NoSystem) derive macro to indicate the component as no system.
///
/// # Examples
///
/// Components are generally used in 2 different ways.
///
/// ## As property/extension of an entity
///
/// Components are a good way to extend entities and bring additional features:
///
/// ```rust
/// # use modor::*;
/// #
/// App::new()
///     .with_entity(build_complex_entity().component(AutoRemoved));
///
/// fn build_complex_entity() -> impl BuiltEntity {
///     EntityBuilder::new()
///     // ...
/// }
///
/// #[derive(Component)]
/// struct AutoRemoved;
///
/// #[systems]
/// impl AutoRemoved {
///     #[run]
///     fn update(&mut self, mut entity: EntityMut<'_>) {
///         entity.delete();
///     }
/// }
/// ```
///
/// ## As self-contained entity
///
/// Some components can also be enough to create an entity, without using [`EntityBuilder`]:
///
/// ```rust
/// # use modor::*;
/// #
/// # struct Color(u8, u8, u8);
/// #
/// # #[derive(SingletonComponent, NoSystem)]
/// # struct LeftPlayerHitCount(u32);
/// #
/// App::new()
///     .with_entity(Score::default());
///
/// #[derive(SingletonComponent, Default)]
/// struct Score {
///     left_player: u32,
///     right_player: u32,
/// }
///
/// #[systems]
/// impl Score {
///     #[run]
///     fn update_left(&mut self, count: Single<'_, LeftPlayerHitCount>) {
///         self.left_player += count.0;
///     }
/// }
/// ```
pub use modor_derive::Component;

/// Defines a singleton component type.
///
/// This macro works the same way as [`Component`](macro@crate::Component), except that when a
/// singleton component is created, an entity with an existing instance of this component
/// type is deleted first, so that there is always a maximum of one instance of the component.
///
/// The instance can be directly accessed in systems using [`Single`] and [`SingleMut`] parameter
/// types.
///
/// # Examples
///
/// ```rust
/// # use modor::*;
/// #
/// #[derive(SingletonComponent, NoSystem)]
/// struct Score(u64);
/// ```
pub use modor_derive::SingletonComponent;

/// Indicates a component type has no system.
///
/// It automatically implements the traits [`ComponentSystems`].
///
/// To define systems for a component type, replace by macro by the
/// [`systems`](macro@crate::systems) proc macro.
///
/// # Examples
///
/// ```rust
/// # use modor::*;
/// #
/// #[derive(Component, NoSystem)]
/// struct Id(u32);
/// ```
pub use modor_derive::NoSystem;

/// Defines the systems of a component.
///
/// This macro should be applied on the component type `impl` block.
///
/// It automatically implements the traits [`ComponentSystems`].
///
/// # System definition
///
/// Systems are methods run at each [`App`] update, and can access to other objects stored
/// by the engine. A system must implement the [`System`] trait.
///
/// You can apply a `run*` attribute on a method present in the `impl` block to run it at each
/// update.
/// Several attributes are available to configure when the system will be run:
/// - `#[run]` to run the system without constraint
/// - `#[run_as(MyAction)]` to run the system labeled with the action `MyAction` implementing
/// the [`Action`] trait.
/// - `#[run_after(Action1, Action2, ...)` to run the system once systems labeled with
/// `Action1`, `Action2`, ... have been executed
/// - `#[run_after_previous]` to run the system after the previous one defined in the `impl` block
/// (has no effect if there is no previous system)
///
/// Note that an action type is created for each component type:
/// - In previously defined attributes, it is possible to refer this action using the `component`
/// attribute: `#[run_as(component(MyComponent))]`
/// - It is also possible to add this action as a dependency of another action defined using the
/// [`Action`](macro@crate::Action) derive macro:
/// ```rust
/// # use modor::*;
/// #
/// # #[derive(Component, NoSystem)]
/// # struct MyComponent;
/// #
/// #[derive(Action)]
/// struct MyAction(<MyComponent as ComponentSystems>::Action);
/// ```
///
/// The action associated to a component type is considered as finished once all systems of the
/// component type have been run.
///
/// The way actions are defined makes sure cyclic dependencies between systems are detected at
/// compile time.
///
/// # System behaviour
///
/// If the system is defined for a component of type `T`, the system is run for each
/// entity containing a component of type `T`.
///
/// Some system parameter types help to access information about the current entity:
/// - `&C` where `C` is a component type (the system is not executed for the entity
/// if it does not have a component of type `C`)
/// - `&mut C` where `C` is a component type (the system is not executed for the entity
/// if it does not have a component of type `C`)
/// - `Option<&C>` where `C` is a component type
/// - `Option<&mut C>` where `C` is a component type
/// - [`Entity`]
///
/// Other system parameter types are more global.
///
/// See implementations of [`SystemParam`] to see the full list of
/// system parameter types.
///
/// # Static checks
///
/// Compile time checks are applied by this macro to ensure the system will not panic at runtime.
/// If the system is invalid, the macro returns a compile time error.
///
/// The [`SystemWithParams`] trait is implemented for all systems.
///
/// The [`SystemWithParamMutabilityIssue`] trait is implemented in case the system is invalid.
/// If this trait is implemented for the system, it creates a compile time error due to a conflict
/// with the implemented [`SystemWithParams`] trait.
///
/// # Limitations
///
/// A system supports up to 10 parameters.<br>
/// If more parameters are needed, tuples can be used to group parameters and count them as one.
///
/// # Examples
///
/// You can use `run*` attributes this way:
///
/// ```rust
/// # use modor::*;
/// #
/// #[derive(Component)]
/// struct MyComponent;
///
/// #[systems]
/// impl MyComponent {
///     #[run]
///     fn system1() {
///         // has no constraint
///     }
///
///     #[run_as(Action2)]
///     fn system2() {
///         // will be run after `system3` because of `Action2` constraints
///     }
///
///     #[run_as(Action1)]
///     fn system3() {
///         // has no constraint because of `Action1` constraints
///     }
///
///     #[run_after(Action1, Action2)]
///     fn system4() {
///         //  will be run after `system2` and `system3`
///     }
///
///     #[run_after_previous]
///     fn system5() {
///         //  will be run after `system4`
///     }
///
///     #[run_after_previous_and(Action2)]
///     fn system6() {
///         //  will be run after `system5` and `system2`
///     }
/// }
///
/// #[derive(Action)]
/// struct Action1;
///
/// #[derive(Action)]
/// struct Action2(Action1);
/// ```
///
/// Here are some valid systems:
///
/// ```rust
/// # use modor::*;
/// #
/// #[derive(Component)]
/// struct Id(u32);
///
/// #[derive(Component)]
/// struct Text(String);
///
/// #[derive(Component)]
/// struct Label;
///
/// #[systems]
/// impl Label {
///     #[run]
///     fn access_entity_info(id: &Id, message: Option<&mut Text>) {
///         // Run for each entity with at least a component of type `Id`
///         // and `Label` (the component type of the `impl` block is always used to filter).
///         // `Text` is not used to filter entities as it is optional.
///         if let Some(message) = message {
///             message.0 = format!("id: {}", id.0);
///         }
///     }
///
///     #[run]
///     fn access_global_info(mut world: World<'_>, query: Query<'_, Entity<'_>>) {
///         // Even if there is no entity-specific parameter, this will be executed for each entity
///         // with the component `Label`.
///         // You generally want to define this type of system for a singleton entity, as it will
///         // be executed at most once.
///         query.iter().for_each(|entity| world.delete_entity(entity.id()));
///     }
///
///     #[run]
///     fn mixed_system(entity: Entity<'_>, mut world: World<'_>) {
///         // You can also mix entity and global parameters.
///         world.delete_entity(entity.id());
///     }
/// }
/// ```
///
/// And here is an invalid system detected at compile time:
///
/// ```compile_fail
/// # use modor::*;
/// #
/// #[derive(Component)]
/// struct MyComponent;
///
/// #[systems]
/// impl MyComponent {
///     #[run]
///     fn invalid_system(&self, self_mut: &mut Self) {
///         // invalid as `MyComponent` cannot be borrowed both mutably and immutably
///     }
/// }
/// ```
pub use modor_derive::systems;

/// Defines a test method that is conditionally run depending on the platform.
///
/// This method adds the `#[test]` attribute to the method if the current platform is not in the
/// list of disabled platforms.
/// The list of disabled platforms must be specified in a `disabled(...)` argument.
///
/// The allowed platforms are:
/// - `android`
/// - `linux`
/// - `macos`
/// - `wasm`
/// - `windows`
///
/// # Platform-specific
///
/// - Web: function is annotated with `#[::wasm_bindgen_test::wasm_bindgen_test]` instead of
/// `#[test]`.
///
/// # Examples
///
/// ```rust
/// # use modor::*;
/// #
/// #[modor_test]
/// fn run_on_all_platforms() { }
///
/// #[modor_test(disabled(linux, wasm))]
/// fn run_on_all_platforms_expect_linux_and_wasm() { }
/// ```
pub use modor_derive::modor_test;
