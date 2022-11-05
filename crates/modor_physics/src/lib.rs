//! Physics module of modor.
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
//! You can start to use this module by creating an entity of type
//! [`PhysicsModule`](crate::PhysicsModule):
//!
//! ```rust
//! use modor::App;
//! use modor_physics::PhysicsModule;
//!
//! let mut app = App::new().with_entity(PhysicsModule::build());
//! loop {
//!     app.update();
//!     # break;
//! }
//! ```
#![cfg_attr(test, allow(clippy::unwrap_used))]

#[macro_use]
extern crate modor;
#[cfg(test)]
#[macro_use]
extern crate modor_internal;

mod components;
mod data;
mod entities;
mod storages_2d;
mod utils;

pub use components::collider_2d::*;
pub use components::dynamics_2d::*;
pub use components::relative_transform_2d::*;
pub use components::transform_2d::*;
pub use data::*;
pub use entities::delta_time::*;
pub use entities::module::*;
