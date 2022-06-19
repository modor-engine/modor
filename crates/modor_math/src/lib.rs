//! Math module of modor.

mod matrices_4d;
mod quaternion;
mod vectors_2d;
mod vectors_3d;

pub use matrices_4d::*;
pub use quaternion::*;
pub use vectors_2d::*;
pub use vectors_3d::*;

// TODO: define must_use policy more precisely
