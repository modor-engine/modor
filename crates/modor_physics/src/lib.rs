//! Physics module of modor.
//!
//! # Getting started
//!
//! You need to include these dependencies in your `Cargo.toml` file:
//! ```toml
//! modor = "0.1"
//! modor_physics = "0.1"
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

#[macro_use]
extern crate modor;

mod components;
mod entities;

pub use components::acceleration::*;
pub use components::position::*;
pub use components::scale::*;
pub use components::shape::*;
pub use components::velocity::*;
pub use entities::delta_time::*;
pub use entities::module::*;
pub use entities::updates_per_second::*;
