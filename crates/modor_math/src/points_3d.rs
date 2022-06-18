use crate::Vec3D;
use std::marker::PhantomData;
use std::ops::{Add, AddAssign, Sub, SubAssign};

/// A point in a 3D space with `U` as unit of distance.
#[derive(Default, Clone, Copy, Debug)]
pub struct Point3D<U> {
    /// The X-coordinate.
    pub x: f32,
    /// The Y-coordinate.
    pub y: f32,
    /// The Z-coordinate.
    pub z: f32,
    phantom: PhantomData<U>,
}

impl<U> Point3D<U> {
    /// Creates a new point.
    #[inline]
    pub const fn xyz(x: f32, y: f32, z: f32) -> Self {
        Self {
            x,
            y,
            z,
            phantom: PhantomData,
        }
    }

    /// Converts to a point with the same coordinates and a different unit of distance `U2`.
    #[inline]
    pub const fn with_unit<U2>(self) -> Point3D<U2> {
        Point3D::xyz(self.x, self.y, self.z)
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

impl<U> Add<Vec3D<U>> for Point3D<U> {
    type Output = Point3D<U>;

    fn add(self, rhs: Vec3D<U>) -> Self::Output {
        Self::xyz(self.x + rhs.x, self.y + rhs.y, self.z + rhs.z)
    }
}

impl<U> Sub<Vec3D<U>> for Point3D<U> {
    type Output = Point3D<U>;

    fn sub(self, rhs: Vec3D<U>) -> Self::Output {
        Self::xyz(self.x - rhs.x, self.y - rhs.y, self.z - rhs.z)
    }
}

impl<U> AddAssign<Vec3D<U>> for Point3D<U> {
    fn add_assign(&mut self, rhs: Vec3D<U>) {
        self.x += rhs.x;
        self.y += rhs.y;
        self.z += rhs.z;
    }
}

impl<U> SubAssign<Vec3D<U>> for Point3D<U> {
    fn sub_assign(&mut self, rhs: Vec3D<U>) {
        self.x -= rhs.x;
        self.y -= rhs.y;
        self.z -= rhs.z;
    }
}
