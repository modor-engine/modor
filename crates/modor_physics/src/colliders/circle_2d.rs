use modor_math::Vec2;

#[derive(Default)]
pub(crate) struct Circle2DCollider {
    pub(crate) center: Vec2,
    pub(crate) radius: f32,
}

impl Circle2DCollider {
    pub(crate) fn update(&mut self, center: Vec2, radius: f32) {
        self.center = center;
        self.radius = radius.abs();
    }
}
