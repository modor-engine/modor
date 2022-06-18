use std::marker::PhantomData;
use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Sub, SubAssign};

/// A vector in a 3D space with `U` as unit of distance.
#[derive(Default, Clone, Copy, Debug)]
pub struct Vec3D<U> {
    /// The X-coordinate.
    pub x: f32,
    /// The Y-coordinate.
    pub y: f32,
    /// The Z-coordinate.
    pub z: f32,
    phantom: PhantomData<U>,
}

impl<U> Vec3D<U> {
    /// A vector with all components equal to `0.0`.
    pub const ZERO: Self = Self::xyz(0., 0., 0.);

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
        Self {
            x,
            y,
            z,
            phantom: PhantomData,
        }
    }

    /// Creates a new vector from 2D coordinates.
    ///
    /// Z-coordinate is initialized to `0.0`.
    #[inline]
    pub const fn xy(x: f32, y: f32) -> Self {
        Self::xyz(x, y, 0.)
    }

    /// Converts to a vector with the same coordinates and a different unit of distance `U2`.
    #[inline]
    pub const fn with_unit<U2>(self) -> Vec3D<U2> {
        Vec3D::xyz(self.x, self.y, self.z)
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
}

impl<U> Add<Vec3D<U>> for Vec3D<U> {
    type Output = Vec3D<U>;

    fn add(self, rhs: Vec3D<U>) -> Self::Output {
        Self::xyz(self.x + rhs.x, self.y + rhs.y, self.z + rhs.z)
    }
}

impl<U> Sub<Vec3D<U>> for Vec3D<U> {
    type Output = Vec3D<U>;

    fn sub(self, rhs: Vec3D<U>) -> Self::Output {
        Self::xyz(self.x - rhs.x, self.y - rhs.y, self.z - rhs.z)
    }
}

impl<U> Mul<f32> for Vec3D<U> {
    type Output = Vec3D<U>;

    fn mul(self, rhs: f32) -> Self::Output {
        Self::xyz(self.x * rhs, self.y * rhs, self.z * rhs)
    }
}

impl<U> Div<f32> for Vec3D<U> {
    type Output = Vec3D<U>;

    fn div(self, rhs: f32) -> Self::Output {
        Self::xyz(self.x / rhs, self.y / rhs, self.z / rhs)
    }
}

impl<U> AddAssign<Vec3D<U>> for Vec3D<U> {
    fn add_assign(&mut self, rhs: Vec3D<U>) {
        self.x += rhs.x;
        self.y += rhs.y;
        self.z += rhs.z;
    }
}

impl<U> SubAssign<Vec3D<U>> for Vec3D<U> {
    fn sub_assign(&mut self, rhs: Vec3D<U>) {
        self.x -= rhs.x;
        self.y -= rhs.y;
        self.z -= rhs.z;
    }
}

impl<U> MulAssign<f32> for Vec3D<U> {
    fn mul_assign(&mut self, rhs: f32) {
        self.x *= rhs;
        self.y *= rhs;
        self.z *= rhs;
    }
}

impl<U> DivAssign<f32> for Vec3D<U> {
    fn div_assign(&mut self, rhs: f32) {
        self.x /= rhs;
        self.y /= rhs;
        self.z /= rhs;
    }
}
