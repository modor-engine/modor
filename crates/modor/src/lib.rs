//! Modor is a modular and kind of object-oriented game engine.
//!
//! This game engine is based on the
//! [entity-component-system](https://en.wikipedia.org/wiki/Entity_component_system) pattern, but
//! provides an API that represents entities like strongly typed objects, and provides tools similar
//! to the object-oriented paradigm:
//! - data are represented by components
//! - logic is put in systems associated to a component type, and are only run for entities containing the linked component type
//! - data and logic inheritance is possible between entity types
//!
//! # Examples
//!
//! See [`App`] and [`Component`](macro@crate::Component).
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
pub use filters::or::*;
pub use filters::with::*;
pub use filters::without::*;
pub use filters::*;
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
/// Components are generally used in 3 different ways.
///
/// ## As encapsulated entity
///
/// It is common to define a component type as a type of entity, and encapsulate in this component
/// type the builder methods and systems of the entity:
///
/// ```rust
/// # use modor::*;
/// #
/// # struct Color(u8, u8, u8);
/// #
/// # #[derive(Component, NoSystem, Default)]
/// # struct Position(f32, f32);
/// #
/// # #[derive(Component, NoSystem)]
/// # struct Sprite(Color);
/// #
/// # impl Sprite {
/// #    fn new(color: Color) -> Self { Self(color) }
/// # }
/// #
/// App::new()
///     .with_entity(Ball::build(Color(255, 255, 0), 0., 0.));
///
/// #[derive(Component)]
/// struct Ball {
///     hit_count: u32,
///     direction: (f32, f32),
/// }
///
/// #[systems]
/// impl Ball {
///     const SPEED: f32 = 0.01;
///
///     fn build(color: Color, x: f32, y: f32) -> impl BuiltEntity {
///         EntityBuilder::new()
///             .with(Self { hit_count: 0, direction: (1., 0.) })
///             .with(Position(x, y))
///             .with(Sprite::new(color))
///     }
///
///     #[run]
///     fn move_(&self, position: &mut Position) {
///         position.0 += self.direction.0 * Self::SPEED;
///         position.1 += self.direction.1 * Self::SPEED;
///     }
/// }
/// ```
///
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
///
/// ## As property/extension of an entity
///
/// Components are also a good way to extend entities and bring additional features:
///
/// ```rust
/// # use modor::*;
/// #
/// App::new()
///     .with_entity(build_complex_entity().with(AutoRemoved));
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
///     fn update(&mut self, entity: Entity<'_>, mut world: World<'_>) {
///         world.delete_entity(entity.id());
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
