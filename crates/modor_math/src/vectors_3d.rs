use crate::Quat;
use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Sub, SubAssign};

/// A vector in a 3D space.
#[derive(Default, Clone, Copy, Debug)]
pub struct Vec3 {
    /// The X-coordinate.
    pub x: f32,
    /// The Y-coordinate.
    pub y: f32,
    /// The Z-coordinate.
    pub z: f32,
}

impl Vec3 {
    /// A vector with all components equal to `0.0`.
    pub const ZERO: Self = Self::xyz(0., 0., 0.);

    /// A vector with all components equal to `1.0`.
    pub const ONE: Self = Self::xyz(1., 1., 1.);

    /// A vector with X and Y components equal to `1.0`.
    pub const XY: Self = Self::xyz(1., 1., 0.);

    /// A vector with X and Z components equal to `1.0`.
    pub const XZ: Self = Self::xyz(1., 0., 1.);

    /// A vector with Y and Z components equal to `1.0`.
    pub const YZ: Self = Self::xyz(0., 1., 1.);

    /// A vector with X and Y components equal to `-1.0`.
    pub const NEG_XY: Self = Self::xyz(1., 1., 0.);

    /// A vector with X and Z components equal to `-1.0`.
    pub const NEG_XZ: Self = Self::xyz(1., 0., 1.);

    /// A vector with Y and Z components equal to `-1.0`.
    pub const NEG_YZ: Self = Self::xyz(0., 1., 1.);

    /// A vector with X component equal to `1.0`.
    pub const X: Self = Self::xyz(1., 0., 0.);

    /// A vector with Y component equal to `1.0`.
    pub const Y: Self = Self::xyz(0., 1., 0.);

    /// A vector with Z component equal to `1.0`.
    pub const Z: Self = Self::xyz(0., 0., 1.);

    /// A vector with X component equal to `-1.0`.
    pub const NEG_X: Self = Self::xyz(-1., 0., 0.);

    /// A vector with Y component equal to `-1.0`.
    pub const NEG_Y: Self = Self::xyz(0., -1., 0.);

    /// A vector with Z component equal to `-1.0`.
    pub const NEG_Z: Self = Self::xyz(0., 0., -1.);

    /// Creates a new vector.
    #[inline]
    #[must_use]
    pub const fn xyz(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }

    /// Creates a new vector from 2D coordinates.
    ///
    /// Z-coordinate is initialized to `0.0`.
    #[inline]
    #[must_use]
    pub const fn xy(x: f32, y: f32) -> Self {
        Self::xyz(x, y, 0.)
    }

    /// Returned the vector rescaled using `scale`.
    ///
    /// The returned vector is the coordinate-wise multiplication of `self` and `scale`.
    #[must_use]
    pub fn with_scale(self, scale: Self) -> Self {
        Self::xyz(self.x * scale.x, self.y * scale.y, self.z * scale.z)
    }

    /// Returns the vector with the same direction and but a different `magnitude`.
    ///
    /// If all components of the vector are equal to `0.0`, `None` is returned.
    #[must_use]
    pub fn with_magnitude(self, magnitude: f32) -> Option<Self> {
        let (x, y, z) = (self.x, self.y, self.z);
        let factor = magnitude / self.magnitude();
        factor
            .is_finite()
            .then(|| Self::xyz(x * factor, y * factor, z * factor))
    }

    /// Returns the magnitude of the vector.
    #[must_use]
    pub fn magnitude(self) -> f32 {
        self.x
            .mul_add(self.x, self.y.mul_add(self.y, self.z.powi(2)))
            .sqrt()
    }

    /// Returns the Euclidean distance with `other`.
    #[must_use]
    pub fn distance(self, other: Self) -> f32 {
        (self - other).magnitude()
    }

    // TODO: add below methods also for Vec2

    #[must_use]
    pub fn rotation(self, other: Self) -> Quat {
        let cross = self.cross(other);
        let w = self.magnitude() * other.magnitude() + self.dot(other);
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

    #[must_use]
    pub fn dot(self, other: Self) -> f32 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }

    #[must_use]
    pub fn cross(self, other: Self) -> Vec3 {
        Vec3::xyz(
            self.y * other.z - self.z * other.y,
            self.z * other.x - self.x * other.z,
            self.x * other.y - self.y * other.x,
        )
    }

    #[must_use]
    pub fn mirror(self, axis_direction: Self) -> Vec3 {
        let axis = axis_direction.with_magnitude(1.).unwrap_or(Vec3::ZERO);
        self - axis * self.dot(axis) * 2.
    }
}

impl Add<Self> for Vec3 {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self::xyz(self.x + rhs.x, self.y + rhs.y, self.z + rhs.z)
    }
}

impl Sub<Self> for Vec3 {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self::xyz(self.x - rhs.x, self.y - rhs.y, self.z - rhs.z)
    }
}

impl Mul<f32> for Vec3 {
    type Output = Self;

    fn mul(self, rhs: f32) -> Self::Output {
        Self::xyz(self.x * rhs, self.y * rhs, self.z * rhs)
    }
}

impl Div<f32> for Vec3 {
    type Output = Self;

    fn div(self, rhs: f32) -> Self::Output {
        Self::xyz(self.x / rhs, self.y / rhs, self.z / rhs)
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
