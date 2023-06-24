//! Graphics module of Modor.
//!
//! # Getting started
//!
//! You need to include these dependencies in your `Cargo.toml` file:
//! ```toml
//! modor = "0.1"
//! modor_graphics = "0.1"
//! modor_physics = "0.1"
//! modor_math = "0.1"
//! ```
//!
//! You can then create a window rendering a rectangle with the following code:
//!
//! ```rust
//! # use modor::*;
//! # use modor_physics::*;
//! # use modor_math::*;
//! # use modor_graphics::*;
//! #
//! # fn no_run() {
//! App::new()
//!     .with_entity(modor_graphics::module())
//!     .with_entity(window())
//!     .with_entity(Camera2D::new(CameraKey, TargetKey))
//!     .with_entity(Material::new(MaterialKey).with_color(Color::RED))
//!     .with_entity(rectangle(Vec2::ZERO, Vec2::new(0.5, 0.2)))
//!     .run(modor_graphics::runner);
//! # }
//!
//! fn window() -> impl BuiltEntity {
//!     EntityBuilder::new()
//!         .with(Window::default())
//!         .with(RenderTarget::new(TargetKey))
//! }
//!
//! fn rectangle(position: Vec2, size: Vec2) -> impl BuiltEntity {
//!     EntityBuilder::new()
//!         .with(Transform2D::new().with_position(position).with_size(size))
//!         .with(Model::rectangle(MaterialKey, CameraKey))
//! }
//!
//! #[derive(Debug, Clone, PartialEq, Eq, Hash)]
//! struct TargetKey;
//!
//! #[derive(Debug, Clone, PartialEq, Eq, Hash)]
//! struct CameraKey;
//!
//! #[derive(Debug, Clone, PartialEq, Eq, Hash)]
//! struct MaterialKey;
//! ```

#[macro_use]
extern crate modor;
#[macro_use]
extern crate log;

mod components;
mod data;
mod entities;
mod gpu_data;
mod input;
mod platform;
mod runner;

pub mod testing;

pub use components::camera::*;
pub use components::frame_rate::*;
pub use components::material::*;
pub use components::model::*;
pub use components::render_target::*;
pub use components::renderer::*;
pub use components::texture::*;
pub use components::texture_buffer::*;
pub use components::window::*;
pub use components::z_index::*;
pub use data::color::*;
pub use data::size::*;
pub use entities::module::*;
pub use runner::*;
