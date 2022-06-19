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
    pub const fn xyz(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }

    /// Creates a new vector from 2D coordinates.
    ///
    /// Z-coordinate is initialized to `0.0`.
    #[inline]
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
    pub fn with_magnitude(self, magnitude: f32) -> Option<Self> {
        let (x, y, z) = (self.x, self.y, self.z);
        let factor = magnitude / self.magnitude();
        factor
            .is_finite()
            .then(|| Self::xyz(x * factor, y * factor, z * factor))
    }

    /// Returns the magnitude of the vector.
    pub fn magnitude(self) -> f32 {
        self.x
            .mul_add(self.x, self.y.mul_add(self.y, self.z.powi(2)))
            .sqrt()
    }

    /// Returns the Euclidean distance with `other`.
    pub fn distance(self, other: Self) -> f32 {
        let x_diff = self.x - other.x;
        let y_diff = self.y - other.y;
        let z_diff = self.z - other.z;
        x_diff
            .mul_add(x_diff, y_diff.mul_add(y_diff, z_diff.powi(2)))
            .sqrt()
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
