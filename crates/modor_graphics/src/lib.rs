//! Graphics crate of Modor.
//!
//! # Getting started
//!
//! You need to include these dependencies in your `Cargo.toml` file:
//! ```toml
//! modor_graphics = "0.1"
//! ```
//!
//! TODO: finish

#![allow(missing_docs, clippy::missing_errors_doc)] // TODO: remove

mod buffer;
mod camera;
mod color;
mod gpu;
mod material;
mod mesh;
mod model;
mod platform;
mod resources;
mod runner;
mod shader;
mod size;
mod target;
mod texture;
mod validation;
mod vertex_buffer;
mod window;

pub use camera::*;
pub use color::*;
pub use material::*;
pub use model::*;
pub use resources::*;
pub use runner::*;
pub use shader::*;
pub use size::*;
pub use target::*;
pub use texture::*;
pub use window::*;

pub use bytemuck;
pub use modor;
pub use modor_input;
pub use modor_physics;
pub use modor_resources;
