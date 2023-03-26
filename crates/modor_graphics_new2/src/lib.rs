#[macro_use]
extern crate modor;

mod components;
mod data;
mod entities;
mod gpu_data;
mod input;
mod resource;
mod runner;

pub use components::camera::*;
pub use components::frame_rate::*;
pub use components::material::*;
pub use components::model::*;
pub use components::render_target::*;
pub use components::renderer::*;
pub use components::texture::*;
pub use components::texture_target_buffer::*;
pub use components::window::*;
pub use components::z_index::*;
pub use data::color::*;
pub use data::size::*;
pub use entities::*;
pub use resource::*;
pub use runner::*;

// TODO: add logs
// TODO: add text rendering
// TODO: add multi-window support
