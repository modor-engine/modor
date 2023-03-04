#[macro_use]
extern crate modor;

mod colors;
mod instances;
mod module;
mod render;
mod rendering;
mod resources;
mod runner;
mod settings;
mod targets;

pub use colors::*;
pub use module::*;
pub use render::captures::*;
pub use render::rendering::*;
pub use resources::cameras::*;
pub use resources::meshes::*;
pub use resources::registries::*;
pub use runner::*;
pub use settings::frame_rate::*;
pub use settings::rendering::*;
pub use settings::window::*;

// TODO: apply everywhere (maybe to put in a wiki):
//  - make sure components can be switched without side effects
//  - avoid creating manually actions (instead, use entity actions as much as possible)
//  - reorder methods in entities (public methods after systems, but system after builders)
//  - take into account buildable entities on user side does not exist
//  - add logging
