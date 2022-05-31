/// A trait for defining a point in a 2D space.
pub trait Point2D: Copy {
    /// Unit of the components.
    type Unit;

    /// Returns components of the point.
    fn components(self) -> (f32, f32);

    /// Returns the Euclidean distance with `other_point`.
    fn distance<P>(self, other_point: P) -> f32
    where
        P: Point2D<Unit = Self::Unit>,
    {
        let (x, y) = self.components();
        let (other_x, other_y) = other_point.components();
        let x_diff = x - other_x;
        let y_diff = y - other_y;
        x_diff.mul_add(x_diff, y_diff.powi(2)).sqrt()
    }
}

/// A trait for defining a point in a 3D space.
pub trait Point3D: Copy {
    /// Unit of the components.
    type Unit;

    /// Returns components of the point.
    fn components(self) -> (f32, f32, f32);

    /// Returns the Euclidean distance with `other_point`.
    fn distance<P>(self, other_point: P) -> f32
    where
        P: Point3D<Unit = Self::Unit>,
    {
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

// TODO: distance between rel and abs position cannot be calculated with Point2D/Point3D
//  -> define a `type Unit;` in the traits.
