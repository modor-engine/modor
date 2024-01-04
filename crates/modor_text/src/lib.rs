//! Text module of Modor.
//!
//! # Getting started
//!
//! You need to include these dependencies in your `Cargo.toml` file:
//! ```toml
//! modor = "0.1"
//! modor_text = "0.1"
//! modor_graphics = "0.1"
//! modor_physics = "0.1"
//! modor_math = "0.1"
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
//!     .with_entity(window_target())
//!     .with_entity(text_2d(WINDOW_CAMERA_2D, "my text", 100.))
//!     .run(modor_graphics::runner);
//! # }
//! ```

#[macro_use]
extern crate modor;

mod components;
mod entities;

pub use components::font::*;
pub use components::material::*;
pub use components::text::*;
pub use entities::module::*;
pub use entities::text::*;
