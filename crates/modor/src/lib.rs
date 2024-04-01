//! Modor is a *mod*ular and *o*bject-o*r*iented game engine.
//!
//! It has been designed with the following principles in mind:
//!
//! - *Modularity*: the engine makes it easy to extend functionalities in an integrated way and to
//!     limit coupling between the different parts of an application.
//! - *Compile-time checking*: the API is designed to avoid as many errors as possible during
//!     runtime.
//! - *Simplicity*: the emphasis is on simplifying the API while guaranteeing good performance for
//!   real-life use cases.
//!
//! # Examples
//!
//! ```rust
//! # use modor::*;
//! # use log::*;
//! #
//! fn main() {
//!     let mut app = App::new::<Root>(Level::Info);
//!     app.update();
//!     app.update();
//!     app.update();
//! }
//!
//! #[derive(Visit)]
//! struct Root {
//!     counter: Counter,
//! }
//!
//! impl RootNode for Root {
//!     fn on_create(ctx: &mut Context<'_>) -> Self {
//!         Self {
//!             counter: Counter::default()
//!         }
//!     }
//! }
//!
//! impl Node for Root {
//!     fn on_enter(&mut self, ctx: &mut Context<'_>) {
//!         println!("Update counter...");
//!     }
//!
//!     fn on_exit(&mut self, ctx: &mut Context<'_>) {
//!         println!("Counter updated, new value is {}", self.counter.value);
//!     }
//! }
//!
//! #[derive(Default, Visit)]
//! struct Counter {
//!     #[modor(skip)] // u32 does not implement `Node` trait, so it cannot be visited
//!     value: u32,
//! }
//!
//! impl Node for Counter {
//!     fn on_enter(&mut self, ctx: &mut Context<'_>) {
//!         self.value += 1;
//!     }
//! }
//! ```

#[cfg(target_os = "android")]
pub use android_activity;
pub use log;
#[cfg(target_arch = "wasm32")]
pub use wasm_bindgen_test;

mod app;
mod node;
mod platform;

pub use app::*;
pub use node::*;
#[allow(unused_imports, unreachable_pub)]
pub use platform::*;

/// Defines the main function of a Modor application.
///
/// This macro ensures to have the same code for all platforms supported by Modor.
///
/// # Examples
///
/// ```rust
/// # use modor::*;
/// # use log::*;
/// #
/// #[modor::main]
/// fn my_main() {
///     let mut app = App::new::<RootNode>(Level::Info);
///     app.update();
/// }
///
/// #[derive(Default, RootNode, Node, Visit)]
/// struct RootNode;
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

/// Implements [`RootNode`].
///
/// The type must implement [`Default`] trait.
///
/// # Examples
///
/// ```rust
/// # use modor::*;
/// #
/// #[derive(Default, RootNode, Node, Visit)]
/// struct Root {
///     #[modor(skip)]
///     value: u32,
/// }
/// ```
pub use modor_derive::RootNode;

/// Implements [`Node`].
///
/// # Examples
///
/// See [`RootNode`](macro@crate::RootNode).
pub use modor_derive::Node;

/// Implements [`Visit`] in case there is no inner node.
///
/// This macro can be used instead of [`Visit`](macro@crate::Visit) when there is no inner node.
/// This avoids unnecessary usage of `#[modor(skip)]` in case all fields should be skipped.
///
/// # Examples
///
/// ```rust
/// # use modor::*;
/// #
/// #[derive(Default, RootNode, Node, NoVisit)]
/// struct Root {
///     value: u32,
/// }
/// ```
pub use modor_derive::NoVisit;

/// Implements [`Visit`] so that inner node are visited.
///
/// `#[modor(skip)]` can be added on a field to skip its automatic update, for example because:
/// - the field type doesn't implement [`Node`].
/// - the field is a node but shouldn't be updated for performance reasons.
///
/// # Examples
///
/// See [`modor`](crate).
pub use modor_derive::Visit;
