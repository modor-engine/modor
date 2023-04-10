//! Text module of Modor.
//!
//! # Getting started
//!
//! You need to include these dependencies in your `Cargo.toml` file:
//! ```toml
//! modor = "0.1"
//! modor_graphics = "0.1"
//! modor_text = "0.1"
//! ```
//!
//! You can then use the components provided by this crate to render texts.

// TODO: add example in doc

#[macro_use]
extern crate modor;

mod builders;
mod components;
mod entities;

pub use builders::material::*;
pub use components::font::*;
pub use components::text::*;
pub use entities::module::*;

// TODO: add doc
// TODO: add tests
