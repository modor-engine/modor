//! UI module of Modor.
//!
//! # Getting started
//!
//! You need to include these dependencies in your `Cargo.toml` file:
//! ```toml
//! modor = "0.1"
//! modor_graphics = "0.1"
//! modor_resources = "0.1"
//! modor_text = "0.1"
//! modor_ui = "0.1"
//! ```
//!
//! You can then create UI widgets with the following code:
//!
//! TODO: add example

#[macro_use]
extern crate modor;

mod components;
mod data;
mod entities;

pub use components::button::*;
pub use components::state::*;
pub use data::*;
pub use entities::button::*;
pub use entities::module::*;
