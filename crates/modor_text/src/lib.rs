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
//! # use modor_text::*;
//! #
//! # fn no_run() {
//! App::new()
//!     .with_entity(modor_text::module())
//!     .with_entity(window())
//!     .with_entity(Camera2D::new(CameraKey, TargetKey))
//!     .with_entity(text())
//!     .run(modor_graphics::runner);
//! # }
//!
//! fn window() -> impl BuiltEntity {
//!     EntityBuilder::new()
//!         .with(Window::default())
//!         .with(RenderTarget::new(TargetKey))
//! }
//!
//! fn text() -> impl BuiltEntity {
//!     TextMaterialBuilder::new(MaterialKey, "my text", 100.)
//!         .build()
//!         .with(Transform2D::new())
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

mod builders;
mod components;
mod entities;

pub use builders::material::*;
pub use components::font::*;
pub use components::text::*;
pub use entities::module::*;
