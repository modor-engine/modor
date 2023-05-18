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

// TODO: add doc
// TODO: add tests
// TODO: remove old graphics crate
// TODO: refactor other modules the same way (module structure, separation of component/entities, ...)
