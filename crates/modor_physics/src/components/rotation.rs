use crate::{AngularVelocity, RelativeAngularVelocity};
use modor_math::{Quat, Vec3};
use std::ops::{Deref, DerefMut};
use std::time::Duration;

/// The absolute rotation of an entity.
///
/// # Modor
///
/// - **Type**: component
/// - **Default if missing**: `Rotation::from(Quat::ZERO)`
/// - **Updated by**: [`PhysicsModule`](crate::PhysicsModule)
/// - **Updated during**: [`UpdatePhysicsAction`](crate::UpdatePhysicsAction)
/// - **Updated using**:
///     - [`AngularVelocity`](crate::AngularVelocity), [`DeltaTime`](crate::DeltaTime)
///     - [`RelativeRotation`](crate::RelativeRotation),
///         [`Rotation`](crate::Rotation) of parent entity
///
/// # Examples
///
/// See [`Position`](crate::Position).
#[derive(Default, Clone, Copy, Debug)]
pub struct Rotation(Quat);

impl Rotation {
    pub(crate) fn update_with_velocity(&mut self, velocity: AngularVelocity, delta_time: Duration) {
        let axis = velocity.axis().unwrap_or(Vec3::ZERO);
        let angle = velocity.angle();
        let rotation = Quat::from_axis_angle(axis, angle * delta_time.as_secs_f32());
        **self = self.with_rotation(rotation);
    }

    pub(crate) fn update_with_relative(
        &mut self,
        relative_rotation: RelativeRotation,
        parent_rotation: Self,
    ) {
        **self = parent_rotation.with_rotation(*relative_rotation);
    }
}

impl From<Quat> for Rotation {
    fn from(quaternion: Quat) -> Self {
        Self(quaternion)
    }
}

impl From<Rotation> for Quat {
    fn from(rotation: Rotation) -> Self {
        rotation.0
    }
}

impl Deref for Rotation {
    type Target = Quat;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Rotation {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

/// The relative rotation of an entity.
///
/// The rotation is relative to the first parent entity with a position, a size and a rotation.<br>
/// In case the entity does not have any parent with a position, a size and a rotation,
/// the relative rotation is equal to the absolute rotation.
///
/// # Modor
///
/// - **Type**: component
/// - **Required components**: [`Position`](crate::Position), [`Size`](crate::Size)
/// - **Updated by**: [`PhysicsModule`](crate::PhysicsModule)
/// - **Updated during**: [`UpdatePhysicsAction`](crate::UpdatePhysicsAction)
/// - **Updated using**: [`RelativeAngularVelocity`](crate::RelativeAngularVelocity),
///     [`DeltaTime`](crate::DeltaTime)
///
/// # Examples
///
/// See [`RelativePosition`](crate::RelativePosition).
#[derive(Default, Clone, Copy, Debug)]
pub struct RelativeRotation(Quat);

impl RelativeRotation {
    pub(crate) fn update(&mut self, velocity: RelativeAngularVelocity, delta_time: Duration) {
        let axis = velocity.axis().unwrap_or(Vec3::ZERO);
        let angle = velocity.angle();
        let rotation = Quat::from_axis_angle(axis, angle * delta_time.as_secs_f32());
        **self = self.with_rotation(rotation);
    }
}

impl From<Quat> for RelativeRotation {
    fn from(quaternion: Quat) -> Self {
        Self(quaternion)
    }
}

impl From<RelativeRotation> for Quat {
    fn from(rotation: RelativeRotation) -> Self {
        rotation.0
    }
}

impl Deref for RelativeRotation {
    type Target = Quat;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for RelativeRotation {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
