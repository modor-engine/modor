use crate::{Mat4, Vec3};
use std::f32::consts::PI;
use std::ops::{Mul, MulAssign};

/// A quaternion used to store a rotation.
#[derive(Clone, Copy, Debug)]
pub struct Quat {
    pub(crate) x: f32,
    pub(crate) y: f32,
    pub(crate) z: f32,
    pub(crate) w: f32,
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
    pub fn from_x(angle: f32) -> Self {
        Self::from_axis_angle(Vec3::X, angle)
    }

    /// Creates a new quaternion from an `angle` in radians around the Y axis.
    #[inline]
    pub fn from_y(angle: f32) -> Self {
        Self::from_axis_angle(Vec3::Y, angle)
    }

    /// Creates a new quaternion from an `angle` in radians around the Z axis.
    #[inline]
    pub fn from_z(angle: f32) -> Self {
        Self::from_axis_angle(Vec3::Z, angle)
    }

    /// Returns the normalized axis, or `None` if the angle is `0.0`.
    #[allow(clippy::float_cmp)]
    pub fn axis(self) -> Option<Vec3> {
        (self.w.powi(2) != 1.).then(|| {
            Vec3::new(
                self.x / self.w.mul_add(-self.w, 1.).sqrt(),
                self.y / self.w.mul_add(-self.w, 1.).sqrt(),
                self.z / self.w.mul_add(-self.w, 1.).sqrt(),
            )
        })
    }

    /// Returns the angle in radians normalized between `0` and `2*π`.
    #[inline]
    pub fn angle(self) -> f32 {
        2. * self.w.acos()
    }

    /// Returns the rotation matrix.
    pub fn matrix(self) -> Mat4 {
        Mat4::from_array([
            [
                1. - (2. * self.y).mul_add(self.y, 2. * self.z * self.z),
                (2. * self.x).mul_add(self.y, 2. * self.w * self.z),
                (2. * self.x).mul_add(self.z, -2. * self.w * self.y),
                0.,
            ],
            [
                (2. * self.x).mul_add(self.y, -2. * self.w * self.z),
                1. - (2. * self.x).mul_add(self.x, 2. * self.z * self.z),
                (2. * self.y).mul_add(self.z, 2. * self.w * self.x),
                0.,
            ],
            [
                (2. * self.x).mul_add(self.z, 2. * self.w * self.y),
                (2. * self.y).mul_add(self.z, -2. * self.w * self.x),
                1. - (2. * self.x).mul_add(self.x, 2. * self.y * self.y),
                0.,
            ],
            [0., 0., 0., 1.],
        ])
    }

    /// Returns the quaternion with a scaled angle.
    ///
    /// Axis is unchanged.
    pub fn with_scale(self, scale: f32) -> Self {
        let axis = self.axis().unwrap_or(Vec3::ZERO);
        let angle = self.angle();
        Self::from_axis_angle(axis, angle * scale)
    }

    /// Returns the quaternion rotated with `other`.
    ///
    /// The same operation can be done using multiplication of both quaternions.
    pub fn with_rotation(self, other: Self) -> Self {
        self * other
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

impl Mul<Self> for Quat {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        Self {
            x: self.y.mul_add(
                rhs.z,
                self.w
                    .mul_add(rhs.x, self.x.mul_add(rhs.w, -self.z * rhs.y)),
            ),
            y: self.z.mul_add(
                rhs.x,
                self.y
                    .mul_add(rhs.w, self.w.mul_add(rhs.y, -self.x * rhs.z)),
            ),
            z: self.z.mul_add(rhs.w, self.w.mul_add(rhs.z, self.x * rhs.y)),
            w: self.w.mul_add(
                rhs.w,
                -self.x.mul_add(rhs.x, self.y.mul_add(rhs.y, self.z * rhs.z)),
            ),
        }
    }
}

impl MulAssign<Self> for Quat {
    fn mul_assign(&mut self, rhs: Self) {
        *self = *self * rhs;
    }
}
