#![allow(
    clippy::non_canonical_clone_impl,
    clippy::non_canonical_partial_ord_impl
)] // warnings caused by Derivative

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
//! #[derive(FromApp)]
//! struct Root {
//!     counter: Counter,
//! }
//!
//! impl State for Root {
//!     fn update(&mut self, app: &mut App) {
//!         println!("Update counter...");
//!         self.counter.value += 1;
//!         println!("Counter updated, new value is {}", self.counter.value);
//!     }
//! }
//!
//! #[derive(Default)]
//! struct Counter {
//!     value: u32,
//! }
//! ```

#[cfg(target_os = "android")]
pub use android_activity;
pub use log;
#[cfg(target_arch = "wasm32")]
pub use wasm_bindgen_test;

mod app;
mod from_app;
mod globals;
mod platform;
mod state;
mod update;

pub use app::*;
pub use from_app::*;
pub use globals::*;
#[allow(unused_imports, unreachable_pub)]
pub use platform::*;
pub use state::*;
pub use update::*;

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
///     let mut app = App::new::<Root>(Level::Info);
///     app.update();
/// }
///
/// #[derive(Default, State)]
/// struct Root;
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
///   `#[test]`.
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

/// Implements [`State`].
///
/// Both structs and enums are supported.
///
/// # Examples
///
/// ```rust
/// # use modor::*;
/// #
/// #[derive(Default, State)]
/// struct Root {
///     value: u32,
/// }
/// ```
pub use modor_derive::State;

/// Implements [`Global`].
///
/// Both structs and enums are supported.
///
/// # Examples
///
/// ```rust
/// # use modor::*;
/// #
/// #[derive(FromApp, Global)]
/// struct Value {
///     integer: u32,
/// }
/// ```
pub use modor_derive::Global;

/// Generates builder methods for a `struct` with named fields.
///
/// The following attributes can be applied on the `struct` fields:
/// - `#[builder(form(value))]`: generates a builder method that replaces the value.
/// - `#[builder(form(closure))]`: generates a builder method that modifies the value.
///
/// # Examples
///
/// ```rust
/// # use modor::*;
/// #
/// #[derive(Default, Builder)]
/// pub struct BuiltStruct {
///     #[builder(form(value))]
///     pub value1: u32,
///     #[builder(form(closure))]
///     value2: Vec<i64>,
///     value3: u8,
/// }
///
/// let value = BuiltStruct::default()
///     .with_value1(10)
///     .with_value2(|v| v.push(20));
/// assert_eq!(value.value1, 10);
/// assert_eq!(value.value2, [20]);
/// ```
///
/// The above `struct` is expended to:
///
/// ```rust
/// # use modor::*;
/// #
/// #[derive(Default)]
/// pub struct BuiltStruct {
///     value1: u32,
///     value2: Vec<i64>,
///     value3: u8,
/// }
///
/// impl BuiltStruct {
///     /// Returns `self` with a different [`value1`](#structfield.value1).
///     pub fn with_value1(mut self, value1: u32) -> Self {
///         self.value1 = value1;
///         self
///     }
///     
///     /// Returns `self` with a different [`value2`](#structfield.value2).
///     fn with_value2(mut self, f: impl FnOnce(&mut Vec<i64>)) -> Self {
///         f(&mut self.value2);
///         self
///     }
/// }
/// ```
pub use modor_derive::Builder;

/// Implements [`FromApp`].
///
/// Only structs are supported. All fields must implement [`FromApp`] (or [`Default`], as
/// [`FromApp`] is automatically implemented for all types implementing [`Default`]).
///
/// # Special cases
///
/// - [`Instant`](std::time::Instant) type is initialized with
///   [`Instant::now`](std::time::Instant::now) method.
pub use modor_derive::FromApp;

/// Generates an updater struct.
///
/// This macro creates a struct `<type name>Updater` that contain fields to updated, wrapped in
/// [`Update`].
///
/// Generally, an `apply` method should be manually added for the type `<type name>Updater`
/// to apply the updates, as it is not generated by this macro.
///
/// # Attributes
///
/// The following attributes can be used on the source fields to control the generation
/// of the associated methods/fields of the `<type name>Updater` struct.
///
/// ## `#[updater(field)]`
///
/// On a field `x: T`, generates a method `fn x(self, x: T) -> Self` to replace the value of the
/// field.
///
/// ## `#[updater(for_field)]`
///
/// On a field `x: T`, generates a method
/// `fn for_x<O>(self, f: impl FnOnce(&mut T) -> O) -> Self`
/// to update the value of the field.
///
/// ## `#[updater(inner_type)]`
///
/// The inner type of the field will be taken (i.e. the first generic type).
///
/// # Examples
///
/// ```rust
/// # use modor::*;
/// # use std::marker::PhantomData;
/// #
/// #[derive(Updater)]
/// struct Value {
///     #[updater(field, for_field)]
///     integer: u8,
///     #[updater(for_field)]
///     string: String,
///     #[updater(inner_type, field)]
///     additional_integer: PhantomData<u16>,
/// }
///
/// impl Default for Value {
///     fn default() -> Self {
///         Self {
///             integer: 5,
///             string: "abcd".into(),
///             additional_integer: PhantomData,
///         }
///     }
/// }
///
/// impl ValueUpdater<'_> {
///     fn apply(mut self, value: &mut Value) {
///         // Bitwise OR operator is used to make sure all `apply_checked` are executed
///         let mut is_updated = Update::apply_checked(&mut self.integer, &mut value.integer)
///             | Update::apply_checked(&mut self.string, &mut value.string);
///         if let Some(additional_integer) = self.additional_integer.take_value_checked(|| 0) {
///             value.integer += additional_integer as u8;
///             is_updated = true;
///         }
///         if is_updated {
///             // perform action
///         }
///     }
/// }
///
/// fn update_value(app: &mut App, value: &mut Value) {
///     ValueUpdater::default()
///         .integer(10)
///         .for_integer(|i| *i += 1)
///         .for_string(String::pop)
///         .additional_integer(20_u16)
///         .apply(value);
///     assert_eq!(value.integer, 31);
///     assert_eq!(value.string, "bbc");
/// }
/// ```
pub use modor_derive::Updater;
