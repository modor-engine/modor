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
//! let mut app = App::new().with_entity(PhysicsModule::build::<()>());
//! loop {
//!     app.update();
//!     # break;
//! }
//! ```

#[macro_use]
extern crate modor;
#[macro_use]
extern crate modor_internal;
extern crate core;

mod colliders;
mod collisions;
mod components;
mod data;
mod entities;

pub use components::collider::*;
pub use components::dynamic_body::*;
pub use components::relative_transform::*;
pub use components::transform::*;
pub use data::*;
pub use entities::delta_time::*;
pub use entities::module::*;
