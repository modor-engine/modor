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
//! You can start to use this module by creating an entity of type
//! [`PhysicsModule`](crate::PhysicsModule):
//!
//! ```rust
//! // TODO: add example
//! ```
#![cfg_attr(test, allow(clippy::unwrap_used))]

#[macro_use]
extern crate modor;
#[macro_use]
extern crate log;

mod components;
mod entities;

pub use components::delta_time::*;
pub use components::dynamics::*;
pub use components::transform::*;
pub use entities::module::*;
