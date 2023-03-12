use crate::{Quat, Vec2};
use approx::{AbsDiffEq, RelativeEq, UlpsEq};
use std::iter::Sum;
use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign};

/// A vector in a 3D space.
#[derive(Default, Clone, Copy, Debug, PartialEq)]
pub struct Vec3 {
    /// X-coordinate.
    pub x: f32,
    /// Y-coordinate.
    pub y: f32,
    /// Z-coordinate.
    pub z: f32,
}

impl Vec3 {
    /// A vector with all components equal to `0.0`.
    pub const ZERO: Self = Self::new(0., 0., 0.);

    /// A vector with all components equal to `1.0`.
    pub const ONE: Self = Self::new(1., 1., 1.);

    /// A vector with X and Y components equal to `1.0`.
    pub const XY: Self = Self::new(1., 1., 0.);

    /// A vector with X and Z components equal to `1.0`.
    pub const XZ: Self = Self::new(1., 0., 1.);

    /// A vector with Y and Z components equal to `1.0`.
    pub const YZ: Self = Self::new(0., 1., 1.);

    /// A vector with X and Y components equal to `-1.0`.
    pub const NEG_XY: Self = Self::new(1., 1., 0.);

    /// A vector with X and Z components equal to `-1.0`.
    pub const NEG_XZ: Self = Self::new(1., 0., 1.);

    /// A vector with Y and Z components equal to `-1.0`.
    pub const NEG_YZ: Self = Self::new(0., 1., 1.);

    /// A vector with X component equal to `1.0`.
    pub const X: Self = Self::new(1., 0., 0.);

    /// A vector with Y component equal to `1.0`.
    pub const Y: Self = Self::new(0., 1., 0.);

    /// A vector with Z component equal to `1.0`.
    pub const Z: Self = Self::new(0., 0., 1.);

    /// A vector with X component equal to `-1.0`.
    pub const NEG_X: Self = Self::new(-1., 0., 0.);

    /// A vector with Y component equal to `-1.0`.
    pub const NEG_Y: Self = Self::new(0., -1., 0.);

    /// A vector with Z component equal to `-1.0`.
    pub const NEG_Z: Self = Self::new(0., 0., -1.);

    /// Creates a new vector.
    #[inline]
    pub const fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }

    /// Creates a new vector from 2D coordinates.
    ///
    /// Z-coordinate is initialized to `0.0`.
    #[inline]
    pub const fn from_xy(x: f32, y: f32) -> Self {
        Self::new(x, y, 0.)
    }

    /// Returned the vector rescaled using `scale`.
    ///
    /// The returned vector is the coordinate-wise multiplication of `self` and `scale`.
    pub fn with_scale(self, scale: Self) -> Self {
        Self::new(self.x * scale.x, self.y * scale.y, self.z * scale.z)
    }

    /// Returns the vector with the same direction and but a different `magnitude`.
    ///
    /// If all components of the vector are equal to `0.0`, `None` is returned.
    pub fn with_magnitude(self, magnitude: f32) -> Option<Self> {
        let (x, y, z) = (self.x, self.y, self.z);
        let factor = magnitude / self.magnitude();
        factor
            .is_finite()
            .then(|| Self::new(x * factor, y * factor, z * factor))
    }

    /// Returns a [`Vec2`](crate::Vec2) containing X and Y coordinates of the vector.
    #[inline]
    pub fn xy(self) -> Vec2 {
        Vec2::new(self.x, self.y)
    }

    /// Returns the magnitude of the vector.
    pub fn magnitude(self) -> f32 {
        self.x
            .mul_add(self.x, self.y.mul_add(self.y, self.z.powi(2)))
            .sqrt()
    }

    /// Returns the Euclidean distance with `other`.
    pub fn distance(self, other: Self) -> f32 {
        (self - other).magnitude()
    }

    /// Returns the rotation between the vector and `other`.
    pub fn rotation(self, other: Self) -> Quat {
        let cross = self.cross(other);
        let w = self.magnitude().mul_add(other.magnitude(), self.dot(other));
        let magnitude = cross
            .x
            .mul_add(
                cross.x,
                cross
                    .y
                    .mul_add(cross.y, cross.z.mul_add(cross.z, w.powi(2))),
            )
            .sqrt();
        Quat {
            x: cross.x / magnitude,
            y: cross.y / magnitude,
            z: cross.z / magnitude,
            w: w / magnitude,
        }
    }

    /// Returns the dot product between the vector and `other`.
    pub fn dot(self, other: Self) -> f32 {
        self.x
            .mul_add(other.x, self.y.mul_add(other.y, self.z * other.z))
    }

    /// Returns the cross product between the vector and `other`.
    pub fn cross(self, other: Self) -> Self {
        Self::new(
            self.y.mul_add(other.z, -self.z * other.y),
            self.z.mul_add(other.x, -self.x * other.z),
            self.x.mul_add(other.y, -self.y * other.x),
        )
    }

    /// Returns the mirror vector with a line of direction `axis_direction`.
    ///
    /// `axis_direction` sense has no impact on the resulting vector.
    pub fn mirror(self, axis_direction: Self) -> Self {
        let axis = axis_direction.with_magnitude(1.).unwrap_or(Self::ZERO);
        axis * self.dot(axis) * 2. - self
    }
}

impl Add<Self> for Vec3 {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self::new(self.x + rhs.x, self.y + rhs.y, self.z + rhs.z)
    }
}

impl Sub<Self> for Vec3 {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self::new(self.x - rhs.x, self.y - rhs.y, self.z - rhs.z)
    }
}

impl Mul<f32> for Vec3 {
    type Output = Self;

    fn mul(self, rhs: f32) -> Self::Output {
        Self::new(self.x * rhs, self.y * rhs, self.z * rhs)
    }
}

impl Div<f32> for Vec3 {
    type Output = Self;

    fn div(self, rhs: f32) -> Self::Output {
        Self::new(self.x / rhs, self.y / rhs, self.z / rhs)
    }
}

impl AddAssign<Self> for Vec3 {
    fn add_assign(&mut self, rhs: Self) {
        self.x += rhs.x;
        self.y += rhs.y;
        self.z += rhs.z;
    }
}

impl SubAssign<Self> for Vec3 {
    fn sub_assign(&mut self, rhs: Self) {
        self.x -= rhs.x;
        self.y -= rhs.y;
        self.z -= rhs.z;
    }
}

impl MulAssign<f32> for Vec3 {
    fn mul_assign(&mut self, rhs: f32) {
        self.x *= rhs;
        self.y *= rhs;
        self.z *= rhs;
    }
}

impl DivAssign<f32> for Vec3 {
    fn div_assign(&mut self, rhs: f32) {
        self.x /= rhs;
        self.y /= rhs;
        self.z /= rhs;
    }
}

impl Mul<Vec3> for f32 {
    type Output = Vec3;

    fn mul(self, rhs: Vec3) -> Self::Output {
        rhs * self
    }
}

impl Neg for Vec3 {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self::new(-self.x, -self.y, -self.z)
    }
}

impl Sum for Vec3 {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.fold(Self::ZERO, |a, b| a + b)
    }
}

impl AbsDiffEq for Vec3 {
    type Epsilon = <f32 as AbsDiffEq>::Epsilon;

    fn default_epsilon() -> Self::Epsilon {
        f32::default_epsilon()
    }

    fn abs_diff_eq(&self, other: &Self, epsilon: Self::Epsilon) -> bool {
        self.x.abs_diff_eq(&other.x, epsilon)
            && self.y.abs_diff_eq(&other.y, epsilon)
            && self.z.abs_diff_eq(&other.z, epsilon)
    }
}

impl RelativeEq for Vec3 {
    fn default_max_relative() -> Self::Epsilon {
        f32::default_max_relative()
    }

    fn relative_eq(
        &self,
        other: &Self,
        epsilon: Self::Epsilon,
        max_relative: Self::Epsilon,
    ) -> bool {
        self.x.relative_eq(&other.x, epsilon, max_relative)
            && self.y.relative_eq(&other.y, epsilon, max_relative)
            && self.z.relative_eq(&other.z, epsilon, max_relative)
    }
}

impl UlpsEq for Vec3 {
    fn default_max_ulps() -> u32 {
        f32::default_max_ulps()
    }

    fn ulps_eq(&self, other: &Self, epsilon: Self::Epsilon, max_ulps: u32) -> bool {
        self.x.ulps_eq(&other.x, epsilon, max_ulps)
            && self.y.ulps_eq(&other.y, epsilon, max_ulps)
            && self.z.ulps_eq(&other.z, epsilon, max_ulps)
    }
}
