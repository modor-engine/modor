use crate::{Acceleration, RelativeAcceleration};
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
#[derive(Clone, Copy, Debug)]
pub struct Velocity {
    /// The X-coordinate.
    pub x: f32,
    /// The Y-coordinate.
    pub y: f32,
    /// The Z-coordinate.
    pub z: f32,
}

impl Velocity {
    /// A velocity with all components equal to zero.
    pub const ZERO: Self = Self::xyz(0., 0., 0.);

    /// Creates a 3D velocity.
    pub const fn xyz(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }

    /// Creates a 2D velocity.
    ///
    /// Z-coordinate is set to zero.
    pub const fn xy(x: f32, y: f32) -> Self {
        Self::xyz(x, y, 0.)
    }

    /// Returns the magnitude.
    pub fn magnitude(&self) -> f32 {
        self.x
            .mul_add(self.x, self.y.mul_add(self.y, self.z.powi(2)))
            .sqrt()
    }

    /// Set the magnitude.
    ///
    /// If the current magnitude of the acceleration is zero, the magnitude is unchanged.
    pub fn set_magnitude(&mut self, magnitude: f32) -> &mut Self {
        let factor = magnitude / self.magnitude();
        if factor.is_finite() {
            self.x *= factor;
            self.y *= factor;
            self.z *= factor;
        }
        self
    }

    pub(crate) fn update(&mut self, acceleration: Acceleration, delta_time: Duration) {
        self.x += acceleration.x * delta_time.as_secs_f32();
        self.y += acceleration.y * delta_time.as_secs_f32();
        self.z += acceleration.z * delta_time.as_secs_f32();
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
#[derive(Clone, Copy, Debug)]
pub struct RelativeVelocity {
    /// The X-coordinate.
    pub x: f32,
    /// The Y-coordinate.
    pub y: f32,
    /// The Z-coordinate.
    pub z: f32,
}

impl RelativeVelocity {
    /// A velocity with all components equal to zero.
    pub const ZERO: Self = Self::xyz(0., 0., 0.);

    /// Creates a 3D velocity.
    pub const fn xyz(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }

    /// Creates a 2D velocity.
    ///
    /// Z-coordinate is set to zero.
    pub const fn xy(x: f32, y: f32) -> Self {
        Self::xyz(x, y, 0.)
    }

    /// Returns the magnitude.
    pub fn magnitude(&self) -> f32 {
        self.x
            .mul_add(self.x, self.y.mul_add(self.y, self.z.powi(2)))
            .sqrt()
    }

    /// Set the magnitude.
    ///
    /// If the current magnitude of the acceleration is zero, the magnitude is unchanged.
    pub fn set_magnitude(&mut self, magnitude: f32) -> &mut Self {
        let factor = magnitude / self.magnitude();
        if factor.is_finite() {
            self.x *= factor;
            self.y *= factor;
            self.z *= factor;
        }
        self
    }

    pub(crate) fn update(&mut self, acceleration: RelativeAcceleration, delta_time: Duration) {
        self.x += acceleration.x * delta_time.as_secs_f32();
        self.y += acceleration.y * delta_time.as_secs_f32();
        self.z += acceleration.z * delta_time.as_secs_f32();
    }
}
