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
//! # use modor::*;
//! # use modor_input::*;
//! #
//! let mut app = App::new()
//!     .with_entity(modor_input::module())
//!     .with_entity(MouseUpdate { is_left_button_pressed: true })
//!     .with_entity(MouseStateDisplay);
//!
//! #[derive(SingletonComponent)]
//! struct MouseUpdate {
//!     is_left_button_pressed: bool
//! }
//!
//! #[systems]
//! impl MouseUpdate {
//!     #[run_as(component(Mouse))]
//!     fn run(&self, mut mouse: SingleMut<'_, '_, Mouse>) {
//!         if self.is_left_button_pressed {
//!             mouse.get_mut()[MouseButton::Left].press();
//!         }
//!     }
//! }
//!
//! #[derive(SingletonComponent)]
//! struct MouseStateDisplay;
//!
//! #[systems]
//! impl MouseStateDisplay {
//!     #[run_after(component(Mouse))]
//!     fn run(&mut self, mouse: SingleRef<'_, '_, Mouse>) {
//!         if mouse.get()[MouseButton::Left].is_pressed() {
//!             println!("Mouse left button is pressed");
//!         } else {
//!             println!("Mouse left button is not pressed");
//!         }
//!     }
//! }
//! ```
//!
//! Note that some crates like `modor_graphics` automatically update the input state.

#[macro_use]
extern crate modor;

mod components;
mod data;
mod entities;
mod platform;
mod utils;

pub use components::fingers::*;
pub use components::gamepads::*;
pub use components::keyboard::*;
pub use components::mouse::*;
pub use components::virtual_keyboard::*;
pub use data::*;
pub use entities::module::*;
