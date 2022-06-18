use crate::{RelativeWorldUnit, WorldUnit};
use modor_math::{Quaternion, Vec3D};
use std::ops::{Deref, DerefMut};

#[derive(Default, Clone, Copy, Debug)]
pub struct Rotation(Quaternion);

impl Rotation {
    pub const ZERO: Self = Self(Quaternion::from_components(0., 0., 0., 1.));

    // TODO: angle is in radians
    pub fn from_axis_angle(axis: Vec3D<WorldUnit>, angle: f32) -> Self {
        Self(Quaternion::from_axis_angle(axis, angle))
    }

    pub(crate) fn update_with_relative(
        &mut self,
        relative_position: RelativeRotation,
        parent_rotation: Rotation,
    ) {
        **self = parent_rotation.with_rotation(*relative_position);
    }
}

impl Deref for Rotation {
    type Target = Quaternion;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Rotation {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[derive(Default, Clone, Copy, Debug)]
pub struct RelativeRotation(Quaternion);

impl RelativeRotation {
    pub const ZERO: Self = Self(Quaternion::from_components(0., 0., 0., 1.));

    pub fn new(axis: Vec3D<RelativeWorldUnit>, angle: f32) -> Self {
        Self(Quaternion::from_axis_angle(axis, angle))
    }
}

impl Deref for RelativeRotation {
    type Target = Quaternion;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for RelativeRotation {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
