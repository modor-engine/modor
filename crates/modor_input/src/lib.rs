//! Input crate of Modor.
//!
//! # Getting started
//!
//! You need to include these dependencies in your `Cargo.toml` file:
//! ```toml
//! modor_input = "0.1"
//! ```
//!
//! Now you can start using this crate, for example by accessing [`Inputs`] state.

mod fingers;
mod gamepads;
mod inputs;
mod keyboard;
mod mouse;
mod normalization;
mod state;

pub use fingers::*;
pub use gamepads::*;
pub use inputs::*;
pub use keyboard::*;
pub use mouse::*;
pub use state::*;

pub use modor;
pub use modor_math;
