//! Input module of Modor.
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
//! use modor::{systems, App, EntityBuilder, SingletonComponent, SingleRef};
//! use modor_input::{InputEvent, InputEventCollector, InputModule, Mouse, MouseButton, MouseEvent};
//!
//! let mut app = App::new()
//!     .with_entity(InputModule::build())
//!     .with_entity(MouseAction);
//! loop {
//!     // By default, no input event is pushed in the module.
//!     // Crates like `modor_graphics` can automatically send events to the module.
//!     // It is also possible to create events manually, which can be convenient for testing.
//!     app.update_components(|c: &mut InputEventCollector| {
//!         c.push(InputEvent::Mouse(MouseEvent::PressedButton(MouseButton::Left)));
//!     });
//!     app.update();
//!     # break;
//! }
//!
//! #[derive(SingletonComponent)]
//! struct MouseAction;
//!
//! #[systems]
//! impl MouseAction {
//!     #[run]
//!     fn run(mouse: SingleRef<'_, '_, Mouse>) {
//!         assert!(mouse.get().button(MouseButton::Left).is_pressed);
//!     }
//! }
//! ```

#[macro_use]
extern crate modor;
#[macro_use]
extern crate log;

mod data;
mod entities;
mod utils;

pub use data::*;
pub use entities::events::*;
pub use entities::gamepads::*;
pub use entities::keyboard::*;
pub use entities::module::*;
pub use entities::mouse::*;
pub use entities::touch::*;
