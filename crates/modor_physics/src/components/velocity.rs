use crate::{Acceleration, RelativeAcceleration};
use modor_math::Vec3;
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
pub struct Velocity(Vec3);

impl Velocity {
    pub(crate) fn update(&mut self, acceleration: Acceleration, delta_time: Duration) {
        **self += *acceleration * delta_time.as_secs_f32();
    }
}

impl From<Vec3> for Velocity {
    fn from(vector: Vec3) -> Self {
        Self(vector)
    }
}

impl From<Velocity> for Vec3 {
    fn from(velocity: Velocity) -> Self {
        velocity.0
    }
}

impl Deref for Velocity {
    type Target = Vec3;

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
pub struct RelativeVelocity(Vec3);

impl RelativeVelocity {
    pub(crate) fn update(&mut self, acceleration: RelativeAcceleration, delta_time: Duration) {
        **self += *acceleration * delta_time.as_secs_f32();
    }
}

impl From<Vec3> for RelativeVelocity {
    fn from(vector: Vec3) -> Self {
        Self(vector)
    }
}

impl From<RelativeVelocity> for Vec3 {
    fn from(velocity: RelativeVelocity) -> Self {
        velocity.0
    }
}

impl Deref for RelativeVelocity {
    type Target = Vec3;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for RelativeVelocity {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
