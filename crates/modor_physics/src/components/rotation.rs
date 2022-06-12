use crate::RelativePosition;
use modor_math::{Quaternion, Vector3D};

#[derive(Clone, Copy, Debug)]
pub struct Rotation {
    x: f32,
    y: f32,
    z: f32,
    w: f32,
}

impl Rotation {
    pub const ZERO: Self = Self {
        x: 0.,
        y: 0.,
        z: 0.,
        w: 1.,
    };

    // TODO: angle is in radians
    pub fn new(axis: RotationAxis, angle: f32) -> Self {
        Self::create_from_axis_angle(axis, angle)
    }

    pub(crate) fn update_with_relative(
        &mut self,
        relative_position: RelativeRotation,
        parent_rotation: Rotation,
    ) {
        *self = parent_rotation.with_rotation(relative_position);
    }
}

impl Quaternion for Rotation {
    type Axis = RotationAxis;

    fn create(x: f32, y: f32, z: f32, w: f32) -> Self {
        Self { x, y, z, w }
    }

    fn components(self) -> (f32, f32, f32, f32) {
        (self.x, self.y, self.z, self.w)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct RelativeRotation {
    x: f32,
    y: f32,
    z: f32,
    w: f32,
}

impl RelativeRotation {
    pub const ZERO: Self = Self {
        x: 0.,
        y: 0.,
        z: 0.,
        w: 1.,
    };

    pub fn new(axis: RotationAxis, angle: f32) -> Self {
        Self::create_from_axis_angle(axis, angle)
    }
}

impl Quaternion for RelativeRotation {
    type Axis = RotationAxis;

    fn create(x: f32, y: f32, z: f32, w: f32) -> Self {
        Self { x, y, z, w }
    }

    fn components(self) -> (f32, f32, f32, f32) {
        (self.x, self.y, self.z, self.w)
    }
}

#[derive(Clone, Copy, Debug, Add, Sub, AddAssign, SubAssign)]
pub struct RotationAxis {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl RotationAxis {
    pub const X: Self = Self::xyz(1., 0., 0.);
    pub const Y: Self = Self::xyz(0., 1., 0.);
    pub const Z: Self = Self::xyz(0., 0., 1.);

    pub const fn xyz(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }
}

impl Vector3D for RotationAxis {
    fn create(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }

    fn components(self) -> (f32, f32, f32) {
        (self.x, self.y, self.z)
    }
}
