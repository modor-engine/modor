use crate::Vec3;
use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Sub, SubAssign};

/// A vector in a 2D space with `U` as unit of distance.
#[derive(Default, Clone, Copy, Debug)]
pub struct Vec2 {
    /// The X-coordinate.
    pub x: f32,
    /// The Y-coordinate.
    pub y: f32,
}

impl Vec2 {
    /// A vector with all components equal to `0.0`.
    pub const ZERO: Self = Self::xy(0., 0.);

    /// A vector with all components equal to `1.0`.
    pub const ONE: Self = Self::xy(1., 1.);

    /// A vector with all components equal to `-1.0`.
    pub const NEG_ONE: Self = Self::xy(-1., -1.);

    /// A vector with X component equal to `1.0`.
    pub const X: Self = Self::xy(1., 0.);

    /// A vector with Y component equal to `1.0`.
    pub const Y: Self = Self::xy(0., 1.);

    /// A vector with X component equal to `-1.0`.
    pub const NEG_X: Self = Self::xy(-1., 0.);

    /// A vector with Y component equal to `-1.0`.
    pub const NEG_Y: Self = Self::xy(0., -1.);

    /// Creates a new vector.
    #[inline]
    #[must_use]
    pub const fn xy(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    /// Converts to a 3D vector with the same x and y coordinates, and a chosen `z` coordinate.
    #[must_use]
    pub const fn with_z(self, z: f32) -> Vec3 {
        Vec3::xyz(self.x, self.y, z)
    }

    /// Returned the vector rescaled using `scale`.
    ///
    /// The returned vector is the coordinate-wise multiplication of `self` and `scale`.
    #[must_use]
    pub fn with_scale(self, scale: Self) -> Self {
        Self::xy(self.x * scale.x, self.y * scale.y)
    }

    /// Returns the vector with the same direction and but a different `magnitude`.
    ///
    /// If all components of the vector are equal to `0.0`, `None` is returned.
    #[must_use]
    pub fn with_magnitude(self, magnitude: f32) -> Option<Self> {
        let (x, y) = (self.x, self.y);
        let factor = magnitude / self.magnitude();
        factor.is_finite().then(|| Self::xy(x * factor, y * factor))
    }

    /// Returns the magnitude of the vector.
    #[must_use]
    pub fn magnitude(self) -> f32 {
        self.x.mul_add(self.x, self.y.powi(2)).sqrt()
    }

    /// Returns the Euclidean distance with `other`.
    #[must_use]
    pub fn distance(self, other: Self) -> f32 {
        let x_diff = self.x - other.x;
        let y_diff = self.y - other.y;
        x_diff.mul_add(x_diff, y_diff.powi(2)).sqrt()
    }
}

impl Add<Self> for Vec2 {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self::xy(self.x + rhs.x, self.y + rhs.y)
    }
}

impl Sub<Self> for Vec2 {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self::xy(self.x - rhs.x, self.y - rhs.y)
    }
}

impl Mul<f32> for Vec2 {
    type Output = Self;

    fn mul(self, rhs: f32) -> Self::Output {
        Self::xy(self.x * rhs, self.y * rhs)
    }
}

impl Div<f32> for Vec2 {
    type Output = Self;

    fn div(self, rhs: f32) -> Self::Output {
        Self::xy(self.x / rhs, self.y / rhs)
    }
}

impl AddAssign<Self> for Vec2 {
    fn add_assign(&mut self, rhs: Self) {
        self.x += rhs.x;
        self.y += rhs.y;
    }
}

impl SubAssign<Self> for Vec2 {
    fn sub_assign(&mut self, rhs: Self) {
        self.x -= rhs.x;
        self.y -= rhs.y;
    }
}

impl MulAssign<f32> for Vec2 {
    fn mul_assign(&mut self, rhs: f32) {
        self.x *= rhs;
        self.y *= rhs;
    }
}

impl DivAssign<f32> for Vec2 {
    fn div_assign(&mut self, rhs: f32) {
        self.x /= rhs;
        self.y /= rhs;
    }
}
