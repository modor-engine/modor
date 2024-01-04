//! Color picking module of Modor.
//!
//! # Getting started
//!
//! You need to include these dependencies in your `Cargo.toml` file:
//! ```toml
//! modor = "0.1"
//! modor_picking = "0.1"
//! modor_graphics = "0.1"
//! modor_physics = "0.1"
//! modor_math = "0.1"
//! ```
//!
//! You can then perform color picking with the following code:
//!
//! TODO: add example
//! ```

#[macro_use]
extern crate modor;

mod components;
mod entities;
mod new;
mod system_params;

pub use components::material_converter::*;
pub use components::no_picking::*;
pub use components::picking::*;
pub use entities::module::*;
pub use system_params::buffer::*;
