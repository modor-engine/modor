use crate::{Acceleration, RelativeAcceleration, RelativeWorldUnitPerSecond, WorldUnitPerSecond};
use modor_math::Vec3D;
use std::ops::{Deref, DerefMut};
use std::time::Duration;

/// The absolute velocity of an entity.
///
/// The velocity is measured in distance units per second.
///
/// # Modor
///
/// - **Type**: component
/// - **Default if missing**: `Velocity::xyz(0., 0., 0.)`
/// - **Required components**: [`Position`](crate::Position)
/// - **Updated by**: [`PhysicsModule`](crate::PhysicsModule)
/// - **Updated during**: [`UpdatePhysicsAction`](crate::UpdatePhysicsAction)
/// - **Updated using**: [`Acceleration`](crate::Acceleration), [`DeltaTime`](crate::DeltaTime)
///
/// # Examples
///
/// See [`Position`](crate::Position).
#[derive(Default, Clone, Copy, Debug)]
pub struct Velocity(Vec3D<WorldUnitPerSecond>);

impl Velocity {
    /// A velocity with all components equal to `0.0`.
    pub const ZERO: Self = Self::xyz(0., 0., 0.);

    /// Creates a 3D velocity.
    #[inline]
    pub const fn xyz(x: f32, y: f32, z: f32) -> Self {
        Self(Vec3D::xyz(x, y, z))
    }

    /// Creates a new velocity from 2D coordinates.
    ///
    /// Z-coordinate is initialized to `0.0`.
    #[inline]
    pub const fn xy(x: f32, y: f32) -> Self {
        Self::xyz(x, y, 0.)
    }

    pub(crate) fn update(&mut self, acceleration: Acceleration, delta_time: Duration) {
        **self += *acceleration * crate::Duration::from(delta_time);
    }
}

impl Deref for Velocity {
    type Target = Vec3D<WorldUnitPerSecond>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Velocity {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

/// The relative velocity of an entity.
///
/// The velocity is measured in distance units per second.<br>
/// A distance unit of 1 along an axis corresponds to the size along this axis of the first
/// parent having a position and a size.
///
/// # Modor
///
/// - **Type**: component
/// - **Default if missing**: `RelativeVelocity::xyz(0., 0., 0.)`
/// - **Required components**: [`RelativePosition`](crate::RelativePosition)
/// - **Updated by**: [`PhysicsModule`](crate::PhysicsModule)
/// - **Updated during**: [`UpdatePhysicsAction`](crate::UpdatePhysicsAction)
/// - **Updated using**: [`RelativeAcceleration`](crate::RelativeAcceleration),
///     [`DeltaTime`](crate::DeltaTime)
///
/// # Examples
///
/// See [`RelativePosition`](crate::RelativePosition).
#[derive(Default, Clone, Copy, Debug)]
pub struct RelativeVelocity(Vec3D<RelativeWorldUnitPerSecond>);

impl RelativeVelocity {
    /// A velocity with all components equal to `0.0`.
    pub const ZERO: Self = Self::xyz(0., 0., 0.);

    // TODO: add missing constants

    /// Creates a new velocity.
    #[inline]
    pub const fn xyz(x: f32, y: f32, z: f32) -> Self {
        Self(Vec3D::xyz(x, y, z))
    }

    /// Creates a new velocity from 2D coordinates.
    ///
    /// Z-coordinate is initialized to `0.0`.
    #[inline]
    pub const fn xy(x: f32, y: f32) -> Self {
        Self::xyz(x, y, 0.)
    }

    pub(crate) fn update(&mut self, acceleration: RelativeAcceleration, delta_time: Duration) {
        **self += *acceleration * crate::Duration::from(delta_time);
    }
}

impl Deref for RelativeVelocity {
    type Target = Vec3D<RelativeWorldUnitPerSecond>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for RelativeVelocity {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
