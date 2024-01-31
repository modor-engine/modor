//! Modor is a *mod*ular and *o*bject-o*r*iented game engine.
//!
//! It has been designed with the following principles in mind:
//!
//! - *Simplicity*: everything in the engine is an "object" accessible by any other object.
//! - *Modularity*: the engine makes it easy to:
//!     - Extend functionalities of the engine with reusable modules.
//!     - Split big projects into multiple independent crates.
//!     - Reduce coupling between parts of a crate.
//! - *Performance*: the engine can take advantage of CPU caching and parallelization for the most
//!   demanding operations.
//!
//! # Examples
//!
//! ```rust
//! # use modor::*;
//! #
//! fn main() {
//!     App::new()
//!         .create(|_| CounterDisplay)
//!         .create(|ctx| Counter::new(ctx, 1))
//!         .create(|ctx| Counter::new(ctx, 2))
//!         .create(|ctx| Counter::new(ctx, 5))
//!         .update();
//! }
//!
//! // Roles are used to order object updates.
//! // Note that modules like physics and graphics modules already provide commonly used roles.
//! struct Resource;
//!
//! impl Role for Resource {
//!     fn constraints() -> Vec<RoleConstraint> {
//!         vec![]
//!     }
//! }
//!
//! struct ResourceReader;
//!
//! impl Role for ResourceReader {
//!     fn constraints() -> Vec<RoleConstraint> {
//!         vec![RoleConstraint::after::<Resource>()]
//!     }
//! }
//!
//! struct CounterDisplay;
//!
//! // Singleton objects have a maximum of one instance that can be easily accessed.
//! impl SingletonObject for CounterDisplay {
//!     type Role = ResourceReader;
//!
//!     // Run at each execution of `App::update()`
//!     fn update(&mut self, ctx: &mut Context<'_, Self>) -> modor::Result<()> {
//!         for counter in ctx.objects::<Counter>()? {
//!             let step = counter.config.get(ctx)?.step;
//!             println!("Value of counter with step {}: {}", step, counter.value);
//!         }
//!         Ok(())
//!     }
//! }
//!
//! struct Counter {
//!     value: u32,
//!     config: Id<CounterConfig>, // Enables easy access to the associated CounterConfig instance
//! }
//!
//! // Regular objects can have any number of instances on which it is easy to iterate, and can be
//! // individually accessed by ID.
//! impl Object for Counter {
//!     type Role = Resource;
//!
//!     // Run at each execution of `App::update()`
//!     fn update(&mut self, ctx: &mut Context<'_, Self>) -> modor::Result<()> {
//!         self.value += self.config.get(ctx)?.step;
//!         Ok(())
//!     }
//! }
//!
//! impl Counter {
//!     fn new(ctx: &mut Context<'_, Self>, step: u32) -> Self {
//!         Self {
//!             value: 0,
//!             config: ctx.create(move |_| CounterConfig::new(step)),
//!         }
//!     }
//! }
//!
//! // Objects without an `update()` method can be defined with less verbosity.
//! #[derive(Object)] // SingletonObject can also be derived
//! struct CounterConfig {
//!     step: u32
//! }
//!
//! impl CounterConfig {
//!     fn new(step: u32) -> Self {
//!         Self { step }
//!     }
//! }
//! ```

#![allow(clippy::non_canonical_clone_impl)] // Warning coming from Derivative macro

mod app;
mod context;
mod id;
mod logging;
mod object;
mod objects;
mod platform;
mod ranges;
mod result;
mod role;
mod storages;

pub use app::*;
pub use context::*;
pub use id::*;
pub use object::*;
pub use objects::*;
#[allow(unused_imports, unreachable_pub)]
pub use platform::*;
pub use ranges::*;
pub use result::*;
pub use role::*;

#[cfg(target_os = "android")]
pub use android_activity;
pub use log;
pub use rayon;
#[cfg(target_arch = "wasm32")]
pub use wasm_bindgen_test;

/// Defines the main function of a Modor application.
///
/// This macro ensures to have the same code for all platforms supported by Modor.
///
/// # Examples
///
/// ```rust
/// # use modor::App;
/// #
/// #[modor::main]
/// fn my_main() {
///     App::new().update();
/// }
/// ```
pub use modor_derive::main;

/// Defines a test function that is conditionally run depending on the platform.
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
/// It is also possible to parametrize the test using `cases(...)` argument, that accepts key-value
/// pairs, where key is a test suffix and value is a string containing the arguments to pass to
/// the test method.
///
/// # Platform-specific
///
/// - Web: function is annotated with `#[wasm_bindgen_test::wasm_bindgen_test]` instead of
/// `#[test]`.
///
/// # Examples
///
/// ```rust
/// #[modor::test]
/// fn run_on_all_platforms() { }
///
/// #[modor::test(disabled(linux, wasm))]
/// fn run_on_all_platforms_except_linux_and_wasm() { }
///
/// #[modor::test(cases(zero = "0, false", one = "1, false", failure = "100, true"))]
/// fn run_parametrized(number: u32, failure: bool) { }
/// ```
pub use modor_derive::test;

/// Implements [`Object`] with a disabled [`Object::update`] method.
///
/// # Examples
///
/// See [`modor`](crate).
pub use modor_derive::Object;

/// Implements [`SingletonObject`] with a disabled [`SingletonObject::update`] method.
///
/// # Examples
///
/// See [`modor`](crate).
pub use modor_derive::SingletonObject;
