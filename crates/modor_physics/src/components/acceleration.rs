use crate::{RelativeWorldUnitPerSecondSquared, WorldUnitPerSecondSquared};
use modor_math::Vec3D;
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
pub struct Acceleration(Vec3D<WorldUnitPerSecondSquared>);

impl Acceleration {
    /// An acceleration with all components equal to `0.0`.
    pub const ZERO: Self = Self::xyz(0., 0., 0.);

    /// Creates a 3D acceleration.
    pub const fn xyz(x: f32, y: f32, z: f32) -> Self {
        Self(Vec3D::xyz(x, y, z))
    }

    /// Creates a new acceleration from 2D coordinates.
    ///
    /// Z-coordinate is initialized to `0.0`.
    pub const fn xy(x: f32, y: f32) -> Self {
        Self::xyz(x, y, 0.)
    }
}

impl Deref for Acceleration {
    type Target = Vec3D<WorldUnitPerSecondSquared>;

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
pub struct RelativeAcceleration(Vec3D<RelativeWorldUnitPerSecondSquared>);

impl RelativeAcceleration {
    /// An acceleration with all components equal to `0.0`.
    pub const ZERO: Self = Self::xyz(0., 0., 0.);

    /// Creates a 3D acceleration.
    pub const fn xyz(x: f32, y: f32, z: f32) -> Self {
        Self(Vec3D::xyz(x, y, z))
    }

    /// Creates a new acceleration from 2D coordinates.
    ///
    /// Z-coordinate is initialized to `0.0`.
    pub const fn xy(x: f32, y: f32) -> Self {
        Self::xyz(x, y, 0.)
    }
}

impl Deref for RelativeAcceleration {
    type Target = Vec3D<RelativeWorldUnitPerSecondSquared>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for RelativeAcceleration {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
