/// A trait for defining a point in a 2D space.
pub trait Point2D: Copy {
    /// Returns components of the point.
    fn components(self) -> (f32, f32);

    /// Returns the Euclidean distance with `other_point`.
    fn distance(self, other_point: Self) -> f32 {
        let (x, y) = self.components();
        let (other_x, other_y) = other_point.components();
        let x_diff = x - other_x;
        let y_diff = y - other_y;
        x_diff.mul_add(x_diff, y_diff.powi(2)).sqrt()
    }
}

/// A trait for defining a point in a 3D space.
pub trait Point3D: Copy {
    /// Returns components of the point.
    fn components(self) -> (f32, f32, f32);

    /// Returns the Euclidean distance with `other_point`.
    fn distance(self, other_point: Self) -> f32 {
        let (x, y, z) = self.components();
        let (other_x, other_y, other_z) = other_point.components();
        let x_diff = x - other_x;
        let y_diff = y - other_y;
        let z_diff = z - other_z;
        x_diff
            .mul_add(x_diff, y_diff.mul_add(y_diff, z_diff.powi(2)))
            .sqrt()
    }
}
