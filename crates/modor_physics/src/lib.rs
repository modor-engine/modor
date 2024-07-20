//! Physics crate of Modor.
//!
//! # Getting started
//!
//! You need to include these dependencies in your `Cargo.toml` file:
//! ```toml
//! modor_physics = "0.1"
//! ```
//!
//! Now you can start using this crate, for example by creating a [`Body2D`] node.

mod body;
mod collision_group;
mod collisions;
mod delta;
mod physics_hooks;
mod pipeline;
mod user_data;

pub use body::*;
pub use collision_group::*;
pub use collisions::*;
pub use delta::*;

pub use modor;
pub use modor_math;
