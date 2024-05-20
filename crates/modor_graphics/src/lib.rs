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

#![allow(clippy::non_canonical_partial_ord_impl)] // warnings caused by Derivative
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
pub use material::default_2d::*;
pub use material::*;
pub use model::*;
pub use runner::*;
pub use shader::glob::*;
pub use shader::*;
pub use size::*;
pub use target::*;
pub use texture::glob::*;
pub use texture::*;
pub use window::*;

pub use bytemuck;
pub use modor;
pub use modor_input;
pub use modor_physics;
pub use modor_resources;
