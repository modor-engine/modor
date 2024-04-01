//! Physics crate of Modor.
//!
//! # Getting started
//!
//! You need to include these dependencies in your `Cargo.toml` file:
//! ```toml
//! modor_physics = "0.1"
//! ```
//!
//! You can start to use this crate:
//!
//! ```rust
//! # use modor::*;
//! # use modor::log::*;
//! # use modor_math::*;
//! # use modor_physics::*;
//! #
//! let mut app = App::new::<Root>(Level::Info);
//! // ...
//!
//! #[derive(Node, Visit)]
//! struct Root {
//!     body: Body2D,
//! }
//!
//! impl RootNode for Root {
//!     fn on_create(ctx: &mut Context<'_>) -> Self {
//!         Self {
//!             body: Body2D::new(ctx, Vec2::ZERO, Vec2::ONE),
//!         }
//!     }
//! }
//! ```

mod body;
mod body_register;
mod collision_group;
mod delta;

pub use body::*;
pub use body_register::*;
pub use collision_group::*;
pub use delta::*;

pub use modor;
pub use modor_math;
