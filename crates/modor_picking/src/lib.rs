//! Color picking module of Modor.
//!
//! # Getting started
//!
//! You need to include these dependencies in your `Cargo.toml` file:
//! ```toml
//! modor = "0.1"
//! modor_picking = "0.1"
//! modor_graphics = "0.1"
//! modor_physics = "0.1"
//! modor_math = "0.1"
//! ```
//!
//! You can then perform color picking with the following code:
//!
//! ```rust
//! # use modor::*;
//! # use modor_graphics::*;
//! # use modor_resources::*;
//! # use modor_picking::*;
//! #
//! # fn no_run() {
//! App::new()
//!     .with_entity(modor_picking::module())
//!     .with_entity(window_target())
//!     .with_entity(EntityPicker {pixel: Pixel::new(200, 300), entity_id: None})
//!     .run(modor_graphics::runner);
//! # }
//!
//! #[derive(Component)]
//! struct EntityPicker {
//!     pixel: Pixel,
//!     entity_id: Option<usize>,
//! }
//!
//! #[systems]
//! impl EntityPicker {
//!     #[run_as(action(TextureBufferPartUpdate))]
//!     fn register_pixel(&mut self, mut picking_buffer: Custom<PickingBuffer<'_>>) {
//!         picking_buffer.register(self.pixel, WINDOW_TARGET);
//!     }
//!
//!     #[run_after(component(TextureBuffer))]
//!     fn retrieve_entity(&mut self, mut picking_buffer: Custom<PickingBuffer<'_>>) {
//!         self.entity_id = picking_buffer.entity_id(self.pixel, WINDOW_TARGET);
//!     }
//! }
//! ```

#[macro_use]
extern crate modor;

mod components;
mod data;
mod entities;
mod system_params;

pub use components::material_converter::*;
pub use components::no_picking::*;
pub use entities::module::*;
pub use system_params::buffer::*;

// TODO: add tests
