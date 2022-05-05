//! Graphics module of modor.
//!
//! # Getting started
//!
//! You need to include these dependencies in your `Cargo.toml` file:
//! ```toml
//! modor = "0.1"
//! modor_graphics = "0.1"
//! ```
//!
//! You can then create a window with the following code:
//!
//! ```rust
//! use modor::App;
//! use modor_graphics::{GraphicsModule, SurfaceSize};
//!
//! # fn no_run() {
//! let mut app = App::new()
//!     .with_entity(GraphicsModule::build(SurfaceSize::new(640, 480), "Title"))
//!     .run(modor_graphics::runner);
//! # }
//! ```
//!
//! Examples of [`ShapeColor`](crate::ShapeColor) show how to create renderable shapes.

#[macro_use]
extern crate modor;
#[macro_use]
extern crate modor_internal;

mod backend;
mod components;
mod data;
mod entities;
mod runner;
mod storages;
mod utils;

pub mod testing;

pub use components::shape_color::*;
pub use data::*;
pub use entities::background_color::*;
pub use entities::frame_rate::*;
pub use entities::module::*;
pub use entities::render_target::*;
pub use runner::*;
