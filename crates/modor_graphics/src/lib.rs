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

// TODO: is it necessary to recreate all resources when surface is recreated on Android ?
//   - If yes: put all WGPU resources at one place to handle cases where Instance is destroyed
//   - If no: drop the gpu_version checks

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
