use modor::{App, BuiltEntity};
use modor_graphics::{model_2d, window_target, Model2DMaterial, WINDOW_CAMERA_2D};
use modor_math::Vec2;
use modor_physics::{Dynamics2D, Transform2D};

pub fn main() {
    App::new()
        .with_entity(modor_graphics::module())
        .with_entity(window_target())
        .with_entity(object())
        .run(modor_graphics::runner);
}

fn object() -> impl BuiltEntity {
    model_2d(WINDOW_CAMERA_2D, Model2DMaterial::Ellipse)
        .updated(|t: &mut Transform2D| t.position = Vec2::new(-0.5, 0.))
        .updated(|t: &mut Transform2D| t.size = Vec2::ONE * 0.04)
        .component(Dynamics2D::new())
        .with(|d| d.velocity = Vec2::ONE * 0.3)
        .with(|d| d.force = -Vec2::Y * 0.2)
        .with(|d| d.mass = 1.)
}
