use crate::{AngularAcceleration, RelativeAngularAcceleration};
use modor_math::{Quat, Vec3};
use std::ops::{Deref, DerefMut};
use std::time::Duration;

/// The absolute angular velocity of an entity.
///
/// The angular velocity is measured in radians per second.
///
/// # Modor
///
/// - **Type**: component
/// - **Default if missing**: `AngularVelocity::from(Quat::ZERO)`
/// - **Required components**: [`Rotation`](crate::Rotation)
/// - **Updated by**: [`PhysicsModule`](crate::PhysicsModule)
/// - **Updated during**: [`UpdatePhysicsAction`](crate::UpdatePhysicsAction)
/// - **Updated using**: [`AngularAcceleration`](crate::AngularAcceleration),
///     [`DeltaTime`](crate::DeltaTime)
///
/// # Examples
///
/// See [`Position`](crate::Position).
#[derive(Default, Clone, Copy, Debug)]
pub struct AngularVelocity(Quat);

impl AngularVelocity {
    pub(crate) fn update(&mut self, acceleration: AngularAcceleration, delta_time: Duration) {
        let axis = acceleration.axis().unwrap_or(Vec3::ZERO);
        let angle = acceleration.angle();
        let rotation = Quat::from_axis_angle(axis, angle * delta_time.as_secs_f32());
        **self = self.with_rotation(rotation);
    }
}

impl From<Quat> for AngularVelocity {
    fn from(vector: Quat) -> Self {
        Self(vector)
    }
}

impl From<AngularVelocity> for Quat {
    fn from(velocity: AngularVelocity) -> Self {
        velocity.0
    }
}

impl Deref for AngularVelocity {
    type Target = Quat;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for AngularVelocity {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

/// The relative angular velocity of an entity.
///
/// The angular velocity is measured in radians per second.
///
/// # Modor
///
/// - **Type**: component
/// - **Default if missing**: `RelativeAngularVelocity::xyz(0., 0., 0.)`
/// - **Required components**: [`RelativeRotation`](crate::RelativeRotation)
/// - **Updated by**: [`PhysicsModule`](crate::PhysicsModule)
/// - **Updated during**: [`UpdatePhysicsAction`](crate::UpdatePhysicsAction)
/// - **Updated using**: [`RelativeAngularAcceleration`](crate::RelativeAngularAcceleration),
///     [`DeltaTime`](crate::DeltaTime)
///
/// # Examples
///
/// See [`RelativePosition`](crate::RelativePosition).
#[derive(Default, Clone, Copy, Debug)]
pub struct RelativeAngularVelocity(Quat);

impl RelativeAngularVelocity {
    pub(crate) fn update(
        &mut self,
        acceleration: RelativeAngularAcceleration,
        delta_time: Duration,
    ) {
        let axis = acceleration.axis().unwrap_or(Vec3::ZERO);
        let angle = acceleration.angle();
        let rotation = Quat::from_axis_angle(axis, angle * delta_time.as_secs_f32());
        **self = self.with_rotation(rotation);
    }
}

impl From<Quat> for RelativeAngularVelocity {
    fn from(quat: Quat) -> Self {
        Self(quat)
    }
}

impl From<RelativeAngularVelocity> for Quat {
    fn from(velocity: RelativeAngularVelocity) -> Self {
        velocity.0
    }
}

impl Deref for RelativeAngularVelocity {
    type Target = Quat;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for RelativeAngularVelocity {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
