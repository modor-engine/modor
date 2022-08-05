use crate::{Quat, Vec3};
use approx::{AbsDiffEq, RelativeEq, UlpsEq};
use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign};

/// A vector in a 2D space with `U` as unit of distance.
#[derive(Default, Clone, Copy, Debug, PartialEq)]
pub struct Vec2 {
    /// The X-coordinate.
    pub x: f32,
    /// The Y-coordinate.
    pub y: f32,
}

impl Vec2 {
    /// A vector with all components equal to `0.0`.
    pub const ZERO: Self = Self::new(0., 0.);

    /// A vector with all components equal to `1.0`.
    pub const ONE: Self = Self::new(1., 1.);

    /// A vector with all components equal to `-1.0`.
    pub const NEG_ONE: Self = Self::new(-1., -1.);

    /// A vector with X component equal to `1.0`.
    pub const X: Self = Self::new(1., 0.);

    /// A vector with Y component equal to `1.0`.
    pub const Y: Self = Self::new(0., 1.);

    /// A vector with X component equal to `-1.0`.
    pub const NEG_X: Self = Self::new(-1., 0.);

    /// A vector with Y component equal to `-1.0`.
    pub const NEG_Y: Self = Self::new(0., -1.);

    /// Creates a new vector.
    #[inline]
    #[must_use]
    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    /// Converts to a 3D vector with the same x and y coordinates, and a chosen `z` coordinate.
    #[inline]
    #[must_use]
    pub const fn with_z(self, z: f32) -> Vec3 {
        Vec3::new(self.x, self.y, z)
    }

    /// Returned the vector rescaled using `scale`.
    ///
    /// The returned vector is the coordinate-wise multiplication of `self` and `scale`.
    #[must_use]
    pub fn with_scale(self, scale: Self) -> Self {
        Self::new(self.x * scale.x, self.y * scale.y)
    }

    /// Returns the vector with the same direction and but a different `magnitude`.
    ///
    /// If all components of the vector are equal to `0.0`, `None` is returned.
    #[must_use]
    pub fn with_magnitude(self, magnitude: f32) -> Option<Self> {
        let (x, y) = (self.x, self.y);
        let factor = magnitude / self.magnitude();
        factor
            .is_finite()
            .then(|| Self::new(x * factor, y * factor))
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

    /// Returns the rotation between the vector and `other`.
    #[must_use]
    pub fn rotation(self, other: Self) -> Quat {
        self.with_z(0.).rotation(other.with_z(0.))
    }

    /// Returns the dot product between the vector and `other`.
    #[must_use]
    pub fn dot(self, other: Self) -> f32 {
        self.x * other.x + self.y * other.y
    }

    /// Returns the cross product between the vector and `other`.
    #[must_use]
    pub fn mirror(self, axis_direction: Self) -> Self {
        let axis = axis_direction.with_magnitude(1.).unwrap_or(Vec2::ZERO);
        axis * self.dot(axis) * 2. - self
    }

    // TODO: test
    /// Returns the perpendicular vector clockwise.
    #[must_use]
    pub fn perpendicular_cw(self) -> Self {
        Self::new(self.y, -self.x)
    }

    /// Returns the perpendicular vector counter clockwise.
    // TODO: test
    #[must_use]
    pub fn perpendicular_ccw(self) -> Self {
        Self::new(-self.y, self.x)
    }
}

impl Add<Self> for Vec2 {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self::new(self.x + rhs.x, self.y + rhs.y)
    }
}

impl Sub<Self> for Vec2 {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self::new(self.x - rhs.x, self.y - rhs.y)
    }
}

impl Mul<f32> for Vec2 {
    type Output = Self;

    fn mul(self, rhs: f32) -> Self::Output {
        Self::new(self.x * rhs, self.y * rhs)
    }
}

impl Div<f32> for Vec2 {
    type Output = Self;

    fn div(self, rhs: f32) -> Self::Output {
        Self::new(self.x / rhs, self.y / rhs)
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

impl Mul<Vec2> for f32 {
    type Output = Vec2;

    fn mul(self, rhs: Vec2) -> Self::Output {
        rhs * self
    }
}

impl Neg for Vec2 {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self::new(-self.x, -self.y)
    }
}

impl AbsDiffEq for Vec2 {
    type Epsilon = <f32 as AbsDiffEq>::Epsilon;

    fn default_epsilon() -> Self::Epsilon {
        f32::default_epsilon()
    }

    fn abs_diff_eq(&self, other: &Self, epsilon: Self::Epsilon) -> bool {
        self.x.abs_diff_eq(&other.x, epsilon) && self.y.abs_diff_eq(&other.y, epsilon)
    }
}

impl RelativeEq for Vec2 {
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
    }
}

impl UlpsEq for Vec2 {
    fn default_max_ulps() -> u32 {
        f32::default_max_ulps()
    }

    fn ulps_eq(&self, other: &Self, epsilon: Self::Epsilon, max_ulps: u32) -> bool {
        self.x.ulps_eq(&other.x, epsilon, max_ulps) && self.y.ulps_eq(&other.y, epsilon, max_ulps)
    }
}
