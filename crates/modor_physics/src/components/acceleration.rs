use modor_math::Vec3;
use std::ops::{Deref, DerefMut};

/// The absolute acceleration of an entity.
///
/// The acceleration is measured in distance units per second squared.
///
/// # Modor
///
/// - **Type**: component
/// - **Default if missing**: `Acceleration::xyz(0., 0., 0.)`
/// - **Required components**: [`Velocity`](crate::Velocity)
///
/// # Examples
///
/// See [`Position`](crate::Position).
#[derive(Default, Clone, Copy, Debug)]
pub struct Acceleration(Vec3);

impl From<Vec3> for Acceleration {
    fn from(vector: Vec3) -> Self {
        Self(vector)
    }
}

impl From<Acceleration> for Vec3 {
    fn from(acceleration: Acceleration) -> Self {
        acceleration.0
    }
}

impl Deref for Acceleration {
    type Target = Vec3;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Acceleration {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

/// The relative acceleration of an entity.
///
/// The acceleration is measured in distance units per second squared.<br>
/// A distance unit along 1 on an axis corresponds to the size along this axis of the first
/// parent having a position and a size.
///
/// # Modor
///
/// - **Type**: component
/// - **Default if missing**: `RelativeAcceleration::xyz(0., 0., 0.)`
/// - **Required components**: [`RelativeVelocity`](crate::RelativeVelocity)
///
/// # Examples
///
/// See [`RelativePosition`](crate::RelativePosition).
#[derive(Default, Clone, Copy, Debug)]
pub struct RelativeAcceleration(Vec3);

impl From<Vec3> for RelativeAcceleration {
    fn from(vector: Vec3) -> Self {
        Self(vector)
    }
}

impl From<RelativeAcceleration> for Vec3 {
    fn from(acceleration: RelativeAcceleration) -> Self {
        acceleration.0
    }
}

impl Deref for RelativeAcceleration {
    type Target = Vec3;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for RelativeAcceleration {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
