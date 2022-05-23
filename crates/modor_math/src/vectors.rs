pub trait Vector2D: Sized {
    fn create(&self, x: f32, y: f32) -> Self;

    fn components(&self) -> (f32, f32);

    fn set_components(&mut self, x: f32, y: f32);

    fn magnitude(&self) -> f32 {
        let (x, y) = self.components();
        x.mul_add(x, y.powi(2)).sqrt()
    }

    fn with_magnitude(&mut self, magnitude: f32) -> Self {
        let (x, y) = self.components();
        let factor = magnitude / self.magnitude();
        if factor.is_finite() {
            self.create(x * factor, y * factor)
        } else {
            self.create(0., 0.)
        }
    }
}

pub trait Vector3D: Sized {
    fn create(&self, x: f32, y: f32, z: f32) -> Self;

    fn components(&self) -> (f32, f32, f32);

    fn magnitude(&self) -> f32 {
        let (x, y, z) = self.components();
        x.mul_add(x, y.mul_add(y, z.powi(2))).sqrt()
    }

    fn with_magnitude(&self, magnitude: f32) -> Self {
        let (x, y, z) = self.components();
        let factor = magnitude / self.magnitude();
        if factor.is_finite() {
            self.create(x * factor, y * factor, z * factor)
        } else {
            self.create(0., 0., 0.)
        }
    }
}
