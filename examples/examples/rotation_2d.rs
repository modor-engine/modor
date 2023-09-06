#![allow(missing_docs)]

use modor::{App, BuiltEntity};
use modor_graphics::{
    model_2d, window_target, Color, Material, Model2DMaterial, ZIndex2D, WINDOW_CAMERA_2D,
};
use modor_math::Vec2;
use modor_physics::{Dynamics2D, PhysicsModule, RelativeTransform2D, Transform2D};
use std::f32::consts::FRAC_PI_2;

#[cfg_attr(target_os = "android", ndk_glue::main(backtrace = "on"))]
pub fn main() {
    App::new()
        .with_entity(PhysicsModule::build())
        .with_entity(modor_text::module())
        .with_entity(window_target())
        .with_entity(object())
        .run(modor_graphics::runner);
}

fn object() -> impl BuiltEntity {
    model_2d(WINDOW_CAMERA_2D, Model2DMaterial::Rectangle)
        .updated(|t: &mut Transform2D| *t.position = Vec2::new(0.15, 0.15))
        .updated(|t: &mut Transform2D| *t.size = Vec2::new(0.25, 0.5))
        .updated(|m: &mut Material| m.color = Color::YELLOW)
        .component(Dynamics2D::new())
        .with(|d| *d.angular_velocity = FRAC_PI_2)
        .child_entity(child())
}

fn child() -> impl BuiltEntity {
    model_2d(WINDOW_CAMERA_2D, Model2DMaterial::Rectangle)
        .updated(|t: &mut Transform2D| *t.size = Vec2::new(0.1, 0.2))
        .updated(|m: &mut Material| m.color = Color::MAGENTA)
        .component(RelativeTransform2D::new())
        .with(|t| t.position = Some(Vec2::ONE * 0.5))
        .with(|t| t.rotation = Some(0.))
        .component(ZIndex2D::from(1))
}
