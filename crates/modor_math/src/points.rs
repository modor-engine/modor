pub trait Point2D {
    fn components(&self) -> (f32, f32);

    fn distance<P>(&self, other_point: &P) -> f32
    where
        P: Point2D,
    {
        let (x, y) = self.components();
        let (other_x, other_y) = other_point.components();
        let x_diff = x - other_x;
        let y_diff = y - other_y;
        x_diff.mul_add(x_diff, y_diff.powi(2)).sqrt()
    }
}

pub trait Point3D {
    fn components(&self) -> (f32, f32, f32);

    fn distance<P>(&self, other_point: &P) -> f32
    where
        P: Point3D,
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
