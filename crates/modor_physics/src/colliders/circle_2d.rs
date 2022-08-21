use crate::Transform;
use modor_math::Vec2;

#[derive(Default)]
pub(crate) struct Circle2DCollider {
    pub(crate) center: Vec2,
    pub(crate) radius: f32,
}

impl Circle2DCollider {
    pub(crate) fn update_outside_collider(&mut self, transform: &Transform) {
        self.center = transform.position.xy();
        self.radius = transform.size.xy().magnitude() / 2.;
    }

    pub(crate) fn update_inside_collider(&mut self, transform: &Transform) {
        self.center = transform.position.xy();
        self.radius = transform.size.x.abs().min(transform.size.y.abs()) / 2.;
    }
}
