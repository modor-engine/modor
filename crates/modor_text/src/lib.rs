//! Text crate of Modor.
//!
//! # Getting started
//!
//! You need to include these dependencies in your `Cargo.toml` file:
//! ```toml
//! modor_text = "0.1"
//! ```
//!
//! Now you can start using this crate, for example by creating a [`Text2D`] to render.

mod font;
mod material;
mod resources;
mod text;

pub use font::*;
pub use material::*;
pub use text::*;

pub use modor_graphics;
