//! Physics module of Modor.
//!
//! # Getting started
//!
//! You need to include these dependencies in your `Cargo.toml` file:
//! ```toml
//! modor = "0.1"
//! modor_physics = "0.1"
//! modor_math = "0.1"
//! ```
//!
//! You can start to use this module the following way:
//!
//! ```rust
//! # use modor::*;
//! # use modor_math::*;
//! # use modor_physics::*;
//! #
//! App::new()
//!     .with_entity(modor_physics::module())
//!     .with_entity(object());
//!
//! fn object() -> impl BuiltEntity {
//!     EntityBuilder::new()
//!         .component(Transform2D::new())
//!         .with(|t| t.position = Vec2::new(0.25, -0.25))
//!         .with(|t| t.size = Vec2::ONE * 0.2)
//!         .component(Dynamics2D::new())
//!         .with(|d| d.velocity = Vec2::new(0.5, 0.2))
//! }
//! ```
#![cfg_attr(test, allow(clippy::unwrap_used))]

#[macro_use]
extern crate modor;
#[macro_use]
extern crate log;

mod components;
mod entities;

pub use components::collider::*;
pub use components::collision_groups::*;
pub use components::delta_time::*;
pub use components::dynamics::*;
pub use components::transform::*;
pub use entities::module::*;
