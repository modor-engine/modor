//! Input module of modor.
//!
//! # Getting started
//!
//! You need to include these dependencies in your `Cargo.toml` file:
//! ```toml
//! modor = "0.1"
//! modor_input = "0.1"
//! modor_math = "0.1"
//! ```
//!
//! You can then start to use this module:
//!
//! ```rust
//! use modor::{singleton, App, Single, Built, EntityBuilder};
//! use modor_input::{InputEvent, InputEventCollector, InputModule, Mouse, MouseButton, MouseEvent};
//!
//! let mut app = App::new()
//!     .with_entity(InputModule::build())
//!     .with_entity(MouseAction::build());
//! // By default, no input event is pushed in the module.
//! // Crates like `modor_graphics` can automatically send events to the module.
//! // It is also possible to create events manually, which can be convenient for testing.
//! let event = InputEvent::Mouse(MouseEvent::PressedButton(MouseButton::Left));
//! app.run_for_singleton(|c: &mut InputEventCollector| c.push(event));
//! loop {
//!     app.update();
//!     # break;
//! }
//!
//! struct MouseAction;
//!
//! #[singleton]
//! impl MouseAction {
//!     fn build() -> impl Built<Self> {
//!         EntityBuilder::new(Self)
//!     }
//!
//!     #[run]
//!     fn run(mouse: Single<'_, Mouse>) {
//!         assert!(mouse.button(MouseButton::Left).is_pressed());
//!     }
//! }
//! ```

#[macro_use]
extern crate modor;
#[macro_use]
extern crate derive_more;

mod data;
mod entities;

pub use data::*;
pub use entities::events::*;
pub use entities::keyboard::*;
pub use entities::module::*;
pub use entities::mouse::*;
