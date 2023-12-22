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
//! # use modor_resources::*;
//! #
//! # fn no_run() {
//! App::new()
//!     .with_entity(modor_graphics::module())
//!     .with_entity(window_target())
//!     .with_entity(rectangle(Vec2::ZERO, Vec2::new(0.5, 0.2)))
//!     .run(modor_graphics::runner);
//! # }
//!
//! fn rectangle(position: Vec2, size: Vec2) -> impl BuiltEntity {
//!     instance_2d(WINDOW_CAMERA_2D, Default2DMaterial::new())
//!         .updated(|t: &mut Transform2D| t.position = position)
//!         .updated(|t: &mut Transform2D| t.size = size)
//!         .updated(|m: &mut Default2DMaterial| m.color = Color::RED)
//! }
//! ```

#[macro_use]
extern crate modor;
#[macro_use]
extern crate log;

mod components;
mod data;
mod entities;
mod errors;
mod gpu_data;
mod input;
mod platform;
mod runner;

pub mod testing;

pub use components::animation::*;
pub use components::anti_aliasing::*;
pub use components::camera::*;
pub use components::frame_rate::*;
pub use components::instance_group::*;
pub use components::instance_rendering::*;
pub use components::material::*;
pub use components::material_source::*;
pub use components::render_target::*;
pub use components::renderer::*;
pub use components::shader::*;
pub use components::texture::*;
pub use components::texture_buffer::*;
pub use components::window::*;
pub use components::z_index::*;
pub use data::color::*;
pub use data::size::*;
pub use entities::instance::*;
pub use entities::material::*;
pub use entities::module::*;
pub use entities::targets::*;
pub use runner::*;
