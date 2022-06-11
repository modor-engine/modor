use crate::{Acceleration, RelativeAcceleration};
use modor_math::Vector3D;
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
#[derive(Clone, Copy, Debug, Add, Sub, AddAssign, SubAssign)]
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

    pub(crate) fn update(&mut self, acceleration: Acceleration, delta_time: Duration) {
        self.x += acceleration.x * delta_time.as_secs_f32();
        self.y += acceleration.y * delta_time.as_secs_f32();
        self.z += acceleration.z * delta_time.as_secs_f32();
    }
}

impl Vector3D for Velocity {
    fn create(x: f32, y: f32, z: f32) -> Self {
        Self::xyz(x, y, z)
    }

    fn components(self) -> (f32, f32, f32) {
        (self.x, self.y, self.z)
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
#[derive(Clone, Copy, Debug, Add, Sub, AddAssign, SubAssign)]
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

    pub(crate) fn update(&mut self, acceleration: RelativeAcceleration, delta_time: Duration) {
        self.x += acceleration.x * delta_time.as_secs_f32();
        self.y += acceleration.y * delta_time.as_secs_f32();
        self.z += acceleration.z * delta_time.as_secs_f32();
    }
}

impl Vector3D for RelativeVelocity {
    fn create(x: f32, y: f32, z: f32) -> Self {
        Self::xyz(x, y, z)
    }

    fn components(self) -> (f32, f32, f32) {
        (self.x, self.y, self.z)
    }
}
