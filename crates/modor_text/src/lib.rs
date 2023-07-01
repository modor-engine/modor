//! Text module of Modor.
//!
//! # Getting started
//!
//! You need to include these dependencies in your `Cargo.toml` file:
//! ```toml
//! modor = "0.1"
//! modor_graphics = "0.1"
//! modor_text = "0.1"
//! ```
//!
//! You can then create a window rendering a text with the following code:
//!
//! ```rust
//! # use modor::*;
//! # use modor_physics::*;
//! # use modor_graphics::*;
//! # use modor_resources::*;
//! # use modor_text::*;
//! #
//! # fn no_run() {
//! App::new()
//!     .with_entity(modor_text::module())
//!     .with_entity(window())
//!     .with_entity(text())
//!     .run(modor_graphics::runner);
//! # }
//!
//! fn window() -> impl BuiltEntity {
//!     let target_key = ResKey::unique("window");
//!     EntityBuilder::new()
//!         .component(Window::default())
//!         .component(RenderTarget::new(target_key))
//!         .component(Camera2D::new(CAMERA, target_key))
//! }
//!
//! fn text() -> impl BuiltEntity {
//!     let material_key = ResKey::unique("text");
//!     TextMaterialBuilder::new(material_key, "my text", 100.)
//!         .build()
//!         .component(Transform2D::new())
//!         .component(Model::rectangle(material_key, CAMERA))
//! }
//!
//! const CAMERA: ResKey<Camera2D> = ResKey::new("main");
//! ```

#[macro_use]
extern crate modor;

mod builders;
mod components;
mod entities;

pub use builders::material::*;
pub use components::font::*;
pub use components::text::*;
pub use entities::module::*;
