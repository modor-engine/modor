//! Modor is a modular and kind of object-oriented game engine.
//!
//! This game engine is based on the
//! [entity-component-system](https://en.wikipedia.org/wiki/Entity_component_system) pattern, but
//! provides an API that represents entities like strongly typed objects, and provides tools similar
//! to the object-oriented paradigm:
//! - data are represented by components
//! - logic is put in systems that are only run for the entity type where they are defined
//! - data and logic inheritance is possible between entity types
//!
//! # Examples
//!
//! ```rust
//! use modor::*;
//!
//! App::new()
//!     .with_entity(Character::build(Position(45., 65.), CharacterType::Main))
//!     .with_entity(Character::build(Position(98., 12.), CharacterType::Enemy))
//!     .with_entity(Character::build(Position(14., 23.), CharacterType::Enemy))
//!     .update();
//!
//! #[derive(Debug)]
//! struct Position(f32, f32);
//!
//! enum CharacterType {
//!     Main,
//!     Neutral,
//!     Enemy,
//! }
//!
//! struct Enemy;
//!
//! struct Character {
//!     ammunition: u32,
//! }
//!
//! #[entity]
//! impl Character {
//!     fn build(position: Position, type_: CharacterType) -> impl Built<Self> {
//!         EntityBuilder::new(Self { ammunition: 10 })
//!             .with_option(matches!(type_, CharacterType::Enemy).then(|| Enemy))
//!             .with(position)
//!     }
//!
//!     #[run]
//!     fn fire_when_enemy(&mut self, position: &Position, _: &Enemy) {
//!         if self.ammunition > 0 {
//!             self.ammunition -= 1;
//!             println!("Enemy at {:?} has fired", position);
//!         }
//!     }
//! }
//! ```
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
mod entities;
mod ranges;
mod storages;
mod system_params;
mod systems;

pub use actions::*;
pub use app::*;
pub use entities::building::*;
pub use entities::filters::*;
pub use entities::runner::*;
pub use entities::traits::*;
pub use ranges::*;
pub use system_params::entity::*;
pub use system_params::filters::*;
pub use system_params::queries::*;
pub use system_params::singletons::*;
pub use system_params::singletons_mut::*;
pub use system_params::world::*;
pub use system_params::*;
pub use systems::building::*;
pub use systems::checks::*;
pub use systems::traits::*;

/// Defines an entity.
///
/// This macro should be applied on the `impl` block of the main component of the entity to define.
///
/// # System definition
///
/// Systems are methods run at each [`App`] update, and can access to other objects stored
/// by the engine. A system must implement the [`System`](crate::System) trait.
///
/// You can apply a `run*` attribute on a method present in the `impl` block to run it at each
/// update.
/// Several attributes are available to configure when the system will be run:
/// - `#[run]` to run the system without constraint
/// - `#[run_as(MyAction)]` to run the system labeled with the action `MyAction` implementing
/// the [`Action`](crate::Action) trait.
/// - `#[run_after(Action1, Action2, ...)` to run the system once systems labeled with
/// `Action1`, `Action2`, ... have been executed
/// - `#[run_after_previous]` to run the system after the previous one defined in the `impl` block
/// (has no effect if there is no previous system)
///
/// Note than the entity also implements [`Action`](crate::Action).<br>
/// If the entity type is put as dependency of a system, then the system will be run once all
/// systems of the entity type have been run.
///
/// Cyclic dependencies between systems are detected at compile time.
///
/// # System behaviour
///
/// If the system is defined for an entity main component of type `E`, the system is run for each
/// entity containing a component of type `E`.
///
/// Some system parameter types help to access information about the current entity:
/// - `&C` where `C` is a component type (the system is not executed for the entity
/// if it does not have a component of type `C`)
/// - `&mut C` where `C` is a component type (the system is not executed for the entity
/// if it does not have a component of type `C`)
/// - `Option<&C>` where `C` is a component type
/// - `Option<&mut C>` where `C` is a component type
/// - [`Entity`](crate::Entity)
///
/// Other system parameter types are more global.
///
/// See [`SystemParam`](crate::SystemParam) to see the full list of system parameter types.
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
/// # use modor::{entity, action, EntityBuilder, Built};
/// #
/// struct MyEntity;
///
/// #[entity]
/// impl MyEntity {
///     fn build() -> impl Built<Self> {
///         EntityBuilder::new(Self)
///     }
///
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
/// }
///
/// #[action]
/// struct Action1;
///
/// #[action(Action1)]
/// struct Action2;
/// ```
///
/// Here are some valid systems:
///
/// ```rust
/// # use modor::{entity, World, Query, Entity};
/// #
/// struct MyEntity;
///
/// #[entity]
/// impl MyEntity {
///     #[run]
///     fn access_entity_info(id: &u32, message: Option<&mut String>) {
///         // Run for each entity with at least a component of type `u32`
///         // and `MyEntity` (the main component is always used to filter).
///         // `String` is not used to filter entities as it is optional.
///         if let Some(message) = message {
///             *message = format!("id: {}", id);
///         }
///     }
///
///     #[run]
///     fn access_global_info(mut world: World<'_>, query: Query<'_, Entity<'_>>) {
///         // Even if there is no entity-specific parameter, this will be executed for each entity
///         // with the main component `MyEntity`.
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
/// # use modor::{entity};
/// #
/// struct MyEntity;
///
/// #[entity]
/// impl MyEntity {
///     #[run]
///     fn invalid_system(name: &String, name_mut: &mut String) {
///         // invalid as `String` cannot be borrowed both mutably and immutably
///         *name_mut = format!("[[[ {} ]]]", name);
///     }
/// }
/// ```
pub use modor_derive::entity;

/// Defines a singleton entity.
///
/// This macro works in the same way as the [`entity`](macro@crate::entity) proc macro.
///
/// When you create a singleton entity, any existing instance is deleted first, except with the
/// [`EntityBuilder::with_dependency`](crate::EntityBuilder::with_dependency) method.<br>
/// The instance can be directly accessed in systems using [`Single`](crate::Single) and
/// [`SingleMut`](crate::SingleMut) parameter types.
///
/// It has to be noted that an entity main component defined as singleton can be added to entities
/// as a simple component (e.g. using [`EntityBuilder::with`](crate::EntityBuilder::with)).<br>
/// In this case, the entity will not be tracked as a singleton by the engine, and so the component
/// will not be accessible in systems using [`Single`](crate::Single) and
/// [`SingleMut`](crate::SingleMut).
///
/// # Examples
///
/// ```rust
/// # use modor::{singleton, EntityBuilder, Built};
/// #
/// struct UpdateCounter(u32);
///
/// #[singleton]
/// impl UpdateCounter {
///     fn build() -> impl Built<Self> {
///         EntityBuilder::new(Self(0))
///     }
///
///     #[run]
///     fn increment_score(&mut self) {
///         self.0 += 1;
///         println!("Number of updates: {}", self.0);
///     }
/// }
/// ```
pub use modor_derive::singleton;

/// Defines a type implementing [`Action`](crate::Action).
///
/// Dependent actions can be passed as argument of this macro.
///
/// # Examples
///
/// ```rust
/// # use modor::action;
/// #
/// #[action]
/// struct A;
///
/// #[action]
/// pub struct B;
///
/// #[action(A, B)]
/// pub(crate) struct C;
/// ```
///
/// This is equivalent to:
/// ```rust
/// # use modor::{Action, DependsOn};
/// #
/// struct A;
///
/// impl Action for A {
///     type Constraint = ();
/// }
///
/// pub struct B;
///
/// impl Action for B {
///     type Constraint = ();
/// }
///
/// pub(crate) struct C;
///
/// impl Action for C {
///     type Constraint = (DependsOn<A>, DependsOn<B>);
/// }
/// ```
pub use modor_derive::action;
