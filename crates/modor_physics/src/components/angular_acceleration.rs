use modor_math::Quat;
use std::ops::{Deref, DerefMut};

/// The absolute angular acceleration of an entity.
///
/// The angular acceleration is measured in radians per second squared.
///
/// # Modor
///
/// - **Type**: component
/// - **Default if missing**: `AngularAcceleration::from(Quat::ZERO)`
/// - **Required components**: [`AngularVelocity`](crate::AngularVelocity)
///
/// # Examples
///
/// See [`Position`](crate::Position).
#[derive(Default, Clone, Copy, Debug)]
pub struct AngularAcceleration(Quat);

impl From<Quat> for AngularAcceleration {
    fn from(vector: Quat) -> Self {
        Self(vector)
    }
}

impl From<AngularAcceleration> for Quat {
    fn from(acceleration: AngularAcceleration) -> Self {
        acceleration.0
    }
}

impl Deref for AngularAcceleration {
    type Target = Quat;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for AngularAcceleration {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

/// The relative angular acceleration of an entity.
///
/// The angular acceleration is measured in radians per second squared.
///
/// # Modor
///
/// - **Type**: component
/// - **Default if missing**: `RelativeAngularAcceleration::xyz(0., 0., 0.)`
/// - **Required components**: [`RelativeAngularVelocity`](crate::RelativeAngularVelocity)
///
/// # Examples
///
/// See [`RelativePosition`](crate::RelativePosition).
#[derive(Default, Clone, Copy, Debug)]
pub struct RelativeAngularAcceleration(Quat);

impl From<Quat> for RelativeAngularAcceleration {
    fn from(quat: Quat) -> Self {
        Self(quat)
    }
}

impl From<RelativeAngularAcceleration> for Quat {
    fn from(acceleration: RelativeAngularAcceleration) -> Self {
        acceleration.0
    }
}

impl Deref for RelativeAngularAcceleration {
    type Target = Quat;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for RelativeAngularAcceleration {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
