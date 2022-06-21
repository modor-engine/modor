use crate::{Mat4, Vec3};
use num_traits::One;
use std::f32::consts::PI;

/// A quaternion used to store a rotation.
#[derive(Clone, Copy, Debug)]
pub struct Quat {
    x: f32,
    y: f32,
    z: f32,
    w: f32,
}

impl Default for Quat {
    fn default() -> Self {
        Self::ZERO
    }
}

impl Quat {
    /// A quaternion corresponding to no rotation.
    pub const ZERO: Self = Self {
        x: 0.,
        y: 0.,
        z: 0.,
        w: 1.,
    };

    /// Creates a new quaternion from an `axis` and an `angle` in radians.
    #[must_use]
    pub fn from_axis_angle(axis: Vec3, angle: f32) -> Self {
        let axis = axis.with_magnitude(1.).unwrap_or(Vec3::ZERO);
        let angle = Self::normalize_angle(angle);
        Self {
            x: axis.x * (angle / 2.).sin(),
            y: axis.y * (angle / 2.).sin(),
            z: axis.z * (angle / 2.).sin(),
            w: (angle / 2.).cos(),
        }
    }

    /// Creates a new quaternion from an `angle` in radians around the X axis.
    #[inline]
    #[must_use]
    pub fn from_x(angle: f32) -> Self {
        Self::from_axis_angle(Vec3::X, angle)
    }

    /// Creates a new quaternion from an `angle` in radians around the Y axis.
    #[inline]
    #[must_use]
    pub fn from_y(angle: f32) -> Self {
        Self::from_axis_angle(Vec3::Y, angle)
    }

    /// Creates a new quaternion from an `angle` in radians around the Z axis.
    #[inline]
    #[must_use]
    pub fn from_z(angle: f32) -> Self {
        Self::from_axis_angle(Vec3::Z, angle)
    }

    /// Returns the normalized axis, or `None` if the angle is `0.0`.
    #[must_use]
    pub fn axis(self) -> Option<Vec3> {
        (!self.w.powi(2).is_one()).then(|| {
            Vec3::xyz(
                self.x / (1. - self.w.powi(2)).sqrt(),
                self.y / (1. - self.w.powi(2)).sqrt(),
                self.z / (1. - self.w.powi(2)).sqrt(),
            )
        })
    }

    /// Returns the angle in radians normalized between `0` and `2*π`.
    #[inline]
    #[must_use]
    pub fn angle(self) -> f32 {
        2. * self.w.acos()
    }

    /// Returns the rotation matrix.
    #[must_use]
    pub fn matrix(self) -> Mat4 {
        Mat4::from_array([
            [
                1. - (2. * self.y).mul_add(self.y, 2. * self.z * self.z),
                (2. * self.x).mul_add(self.y, -2. * self.w * self.z),
                (2. * self.x).mul_add(self.z, 2. * self.w * self.y),
                0.,
            ],
            [
                (2. * self.x).mul_add(self.y, 2. * self.w * self.z),
                1. - (2. * self.x).mul_add(self.x, 2. * self.z * self.z),
                (2. * self.y).mul_add(self.z, -2. * self.w * self.x),
                0.,
            ],
            [
                (2. * self.x).mul_add(self.z, -2. * self.w * self.y),
                (2. * self.y).mul_add(self.z, 2. * self.w * self.x),
                1. - (2. * self.x).mul_add(self.x, 2. * self.y * self.y),
                0.,
            ],
            [0., 0., 0., 1.],
        ])
    }

    /// Returns the quaternion rotated with `other`.
    #[must_use]
    pub fn with_rotation(self, other: Self) -> Self {
        Self {
            x: self.y.mul_add(
                other.z,
                self.w
                    .mul_add(other.x, self.x.mul_add(other.w, -self.z * other.y)),
            ),
            y: self.z.mul_add(
                other.x,
                self.y
                    .mul_add(other.w, self.w.mul_add(other.y, -self.x * other.z)),
            ),
            z: self
                .z
                .mul_add(other.w, self.w.mul_add(other.z, self.x * other.y)),
            w: self.w.mul_add(
                other.w,
                -self
                    .x
                    .mul_add(other.x, self.y.mul_add(other.y, self.z * other.z)),
            ),
        }
    }

    fn normalize_angle(mut angle: f32) -> f32 {
        while angle > 2. * PI {
            angle -= 2. * PI;
        }
        while angle < 0. {
            angle += 2. * PI;
        }
        angle
    }
}
