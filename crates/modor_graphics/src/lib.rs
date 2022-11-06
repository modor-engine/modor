//! Graphics module of modor.
//!
//! # Getting started
//!
//! You need to include these dependencies in your `Cargo.toml` file:
//! ```toml
//! modor = "0.1"
//! modor_graphics = "0.1"
//! modor_math = "0.1"
//! ```
//!
//! You can then create a window with the following code:
//!
//! ```rust
//! use modor::App;
//! use modor_graphics::{GraphicsModule, SurfaceSize, WindowSettings};
//!
//! # fn no_run() {
//! let mut app = App::new()
//!      .with_entity(GraphicsModule::build(
//!          WindowSettings::default()
//!              .size(SurfaceSize::new(640, 480))
//!              .title("Title"),
//!      ))
//!     .run(modor_graphics::runner);
//! # }
//! ```
//!
//! Examples of [`Mesh2D`](crate::Mesh2D) show how to create renderable shapes.

#[macro_use]
extern crate modor;
#[macro_use]
extern crate modor_internal;
#[macro_use]
extern crate log;

mod backend;
mod components;
mod data;
mod entities;
mod runner;
mod storages;
mod utils;

pub mod testing;

pub use components::mesh_2d::*;
pub use data::*;
pub use entities::background::*;
pub use entities::camera_2d::*;
pub use entities::frame_rate::*;
pub use entities::module::*;
pub use entities::render_target::*;
pub use entities::textures::*;
pub use runner::*;
